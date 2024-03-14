use core::fmt::Debug;
use core::ops::Sub;
use core::time::Duration;

use basecoin_store::context::ProvableStore;
use basecoin_store::impls::{GrowingStore, InMemoryStore, RevertibleStore};
use ibc::core::channel::types::channel::ChannelEnd;
use ibc::core::channel::types::commitment::PacketCommitment;
use ibc::core::client::context::ClientExecutionContext;
use ibc::core::client::types::Height;
use ibc::core::connection::types::ConnectionEnd;
use ibc::core::entrypoint::dispatch;
use ibc::core::handler::types::events::IbcEvent;
use ibc::core::handler::types::msgs::MsgEnvelope;
use ibc::core::host::types::identifiers::{ChannelId, ClientId, ConnectionId, PortId, Sequence};
use ibc::core::host::types::path::{
    ChannelEndPath, ClientConsensusStatePath, ClientStatePath, CommitmentPath, ConnectionPath,
    SeqAckPath, SeqRecvPath, SeqSendPath,
};
use ibc::core::host::{ExecutionContext, ValidationContext};
use ibc::core::router::router::Router;
use ibc::primitives::prelude::*;
use ibc::primitives::Timestamp;
use typed_builder::TypedBuilder;

use super::testapp::ibc::core::types::{LightClientState, MockIbcStore};
use crate::fixtures::core::context::MockContextConfig;
use crate::hosts::{TestBlock, TestHeader, TestHost};
use crate::relayer::error::RelayerError;
use crate::testapp::ibc::clients::{AnyClientState, AnyConsensusState};

/// A context implementing the dependencies necessary for testing any IBC module.
#[derive(Debug)]
pub struct MockGenericContext<S, H>
where
    S: ProvableStore + Debug,
    H: TestHost,
{
    /// The type of host chain underlying this mock context.
    pub host: H,

    /// An object that stores all IBC related data.
    pub ibc_store: MockIbcStore<S>,
}

pub type MockContext<H> = MockGenericContext<RevertibleStore<GrowingStore<InMemoryStore>>, H>;

#[derive(Debug, TypedBuilder)]
pub struct MockClientConfig {
    #[builder(default = Duration::from_secs(64000))]
    pub trusting_period: Duration,
    #[builder(default = Duration::from_millis(3000))]
    pub max_clock_drift: Duration,
    #[builder(default = Duration::from_secs(128_000))]
    pub unbonding_period: Duration,
}

impl Default for MockClientConfig {
    fn default() -> Self {
        Self::builder().build()
    }
}

/// Returns a MockContext with bare minimum initialization: no clients, no connections and no channels are
/// present, and the chain has Height(5). This should be used sparingly, mostly for testing the
/// creation of new domain objects.
impl<S, H> Default for MockGenericContext<S, H>
where
    S: ProvableStore + Debug + Default,
    H: TestHost,
{
    fn default() -> Self {
        MockContextConfig::builder().build()
    }
}

/// Implementation of internal interface for use in testing. The methods in this interface should
/// _not_ be accessible to any Ics handler.
impl<S, H> MockGenericContext<S, H>
where
    S: ProvableStore + Debug,
    H: TestHost,
{
    pub fn ibc_store(&self) -> &MockIbcStore<S> {
        &self.ibc_store
    }

    pub fn host_block(&self, target_height: &Height) -> Option<H::Block> {
        self.host.get_block(target_height)
    }

    pub fn query_latest_block(&self) -> Option<H::Block> {
        self.host.get_block(&self.latest_height())
    }

    pub fn advance_block_up_to(mut self, target_height: Height) -> Self {
        self.host.advance_block_up_to(target_height);
        self
    }

    pub fn latest_height(&self) -> Height {
        let latest_ibc_height = self.ibc_store.host_height().expect("Never fails");
        let latest_host_height = self.host.latest_height();
        assert_eq!(
            latest_ibc_height, latest_host_height,
            "The IBC store and the host chain must have the same height"
        );
        latest_ibc_height
    }

    pub fn latest_timestamp(&self) -> Timestamp {
        self.host_block(&self.latest_height())
            .expect("Never fails")
            .timestamp()
    }

    pub fn timestamp_at(&self, height: Height) -> Timestamp {
        let n_blocks = self
            .host
            .blocks_since(height)
            .expect("less or equal height");
        self.latest_timestamp()
            .sub(self.host.block_time() * (n_blocks as u32))
            .expect("Never fails")
    }

    pub fn with_client_state(mut self, client_id: &ClientId, client_state: AnyClientState) -> Self {
        let client_state_path = ClientStatePath::new(client_id.clone());
        self.ibc_store
            .store_client_state(client_state_path, client_state)
            .expect("error writing to store");
        self
    }

    pub fn with_consensus_state(
        mut self,
        client_id: &ClientId,
        height: Height,
        consensus_state: AnyConsensusState,
    ) -> Self {
        let consensus_state_path = ClientConsensusStatePath::new(
            client_id.clone(),
            height.revision_number(),
            height.revision_height(),
        );
        self.ibc_store
            .store_consensus_state(consensus_state_path, consensus_state)
            .expect("error writing to store");

        self
    }

    pub fn generate_light_client(
        &self,
        consensus_heights: Vec<Height>,
        client_params: &H::LightClientParams,
    ) -> LightClientState<H> {
        let client_height = if let Some(&height) = consensus_heights.last() {
            height
        } else {
            self.latest_height()
        };

        let client_state = self
            .host
            .generate_client_state(client_height, client_params);

        let consensus_states = consensus_heights
            .into_iter()
            .map(|height| {
                (
                    height,
                    self.host_block(&height)
                        .expect("block exists")
                        .into_header()
                        .into_consensus_state(),
                )
            })
            .collect();

        LightClientState {
            client_state,
            consensus_states,
        }
    }

    pub fn with_light_client<RH>(
        mut self,
        client_id: &ClientId,
        light_client: LightClientState<RH>,
    ) -> Self
    where
        RH: TestHost,
    {
        self = self.with_client_state(client_id, light_client.client_state.into());

        for (height, consensus_state) in light_client.consensus_states {
            self = self.with_consensus_state(client_id, height, consensus_state.into());

            self.ibc_store
                .store_update_meta(
                    client_id.clone(),
                    height,
                    self.latest_timestamp(),
                    self.latest_height(),
                )
                .expect("error writing to store");
        }

        self
    }

    /// Associates a connection to this context.
    pub fn with_connection(
        mut self,
        connection_id: ConnectionId,
        connection_end: ConnectionEnd,
    ) -> Self {
        let connection_path = ConnectionPath::new(&connection_id);
        self.ibc_store
            .store_connection(&connection_path, connection_end)
            .expect("error writing to store");
        self
    }

    /// Associates a channel (in an arbitrary state) to this context.
    pub fn with_channel(
        mut self,
        port_id: PortId,
        chan_id: ChannelId,
        channel_end: ChannelEnd,
    ) -> Self {
        let channel_end_path = ChannelEndPath::new(&port_id, &chan_id);
        self.ibc_store
            .store_channel(&channel_end_path, channel_end)
            .expect("error writing to store");
        self
    }

    pub fn with_send_sequence(
        mut self,
        port_id: PortId,
        chan_id: ChannelId,
        seq_number: Sequence,
    ) -> Self {
        let seq_send_path = SeqSendPath::new(&port_id, &chan_id);
        self.ibc_store
            .store_next_sequence_send(&seq_send_path, seq_number)
            .expect("error writing to store");
        self
    }

    pub fn with_recv_sequence(
        mut self,
        port_id: PortId,
        chan_id: ChannelId,
        seq_number: Sequence,
    ) -> Self {
        let seq_recv_path = SeqRecvPath::new(&port_id, &chan_id);
        self.ibc_store
            .store_next_sequence_recv(&seq_recv_path, seq_number)
            .expect("error writing to store");
        self
    }

    pub fn with_ack_sequence(
        mut self,
        port_id: PortId,
        chan_id: ChannelId,
        seq_number: Sequence,
    ) -> Self {
        let seq_ack_path = SeqAckPath::new(&port_id, &chan_id);
        self.ibc_store
            .store_next_sequence_ack(&seq_ack_path, seq_number)
            .expect("error writing to store");
        self
    }

    pub fn with_packet_commitment(
        mut self,
        port_id: PortId,
        chan_id: ChannelId,
        seq: Sequence,
        data: PacketCommitment,
    ) -> Self {
        let commitment_path = CommitmentPath::new(&port_id, &chan_id, seq);
        self.ibc_store
            .store_packet_commitment(&commitment_path, data)
            .expect("error writing to store");
        self
    }

    /// A datagram passes from the relayer to the IBC module (on host chain).
    /// Alternative method to `Ics18Context::send` that does not exercise any serialization.
    /// Used in testing the Ics18 algorithms, hence this may return a Ics18Error.
    pub fn deliver(
        &mut self,
        router: &mut impl Router,
        msg: MsgEnvelope,
    ) -> Result<(), RelayerError> {
        dispatch(&mut self.ibc_store, router, msg).map_err(RelayerError::TransactionFailed)?;
        // Create a new block.
        self.host.advance_block();
        Ok(())
    }

    pub fn get_events(&self) -> Vec<IbcEvent> {
        self.ibc_store.events.lock().clone()
    }

    pub fn get_logs(&self) -> Vec<String> {
        self.ibc_store.logs.lock().clone()
    }
}

#[cfg(test)]
mod tests {
    use ibc::core::host::types::identifiers::ChainId;

    use super::*;
    use crate::hosts::{MockHost, TendermintHost};

    #[test]
    fn test_history_manipulation_mock() {
        pub struct Test<H: TestHost> {
            name: String,
            ctx: MockContext<H>,
        }

        fn run_tests<H: TestHost>(sub_title: &str) {
            let cv = 1; // The version to use for all chains.
            let mock_chain_id = ChainId::new(&format!("mockgaia-{cv}")).unwrap();

            let tests: Vec<Test<H>> = vec![
                Test {
                    name: "Empty history, small pruning window".to_string(),
                    ctx: MockContextConfig::builder()
                        .host_id(mock_chain_id.clone())
                        .max_history_size(2)
                        .latest_height(Height::new(cv, 1).expect("Never fails"))
                        .build(),
                },
                Test {
                    name: "Large pruning window".to_string(),
                    ctx: MockContextConfig::builder()
                        .host_id(mock_chain_id.clone())
                        .max_history_size(30)
                        .latest_height(Height::new(cv, 2).expect("Never fails"))
                        .build(),
                },
                Test {
                    name: "Small pruning window".to_string(),
                    ctx: MockContextConfig::builder()
                        .host_id(mock_chain_id.clone())
                        .max_history_size(3)
                        .latest_height(Height::new(cv, 30).expect("Never fails"))
                        .build(),
                },
                Test {
                    name: "Small pruning window, small starting height".to_string(),
                    ctx: MockContextConfig::builder()
                        .host_id(mock_chain_id.clone())
                        .max_history_size(3)
                        .latest_height(Height::new(cv, 2).expect("Never fails"))
                        .build(),
                },
                Test {
                    name: "Large pruning window, large starting height".to_string(),
                    ctx: MockContextConfig::builder()
                        .host_id(mock_chain_id.clone())
                        .max_history_size(50)
                        .latest_height(Height::new(cv, 2000).expect("Never fails"))
                        .build(),
                },
            ];

            for mut test in tests {
                // All tests should yield a valid context after initialization.
                assert!(
                    test.ctx.host.validate().is_ok(),
                    "failed in test [{}] {} while validating context {:?}",
                    sub_title,
                    test.name,
                    test.ctx
                );

                let current_height = test.ctx.latest_height();

                // After advancing the chain's height, the context should still be valid.
                test.ctx.host.advance_block();
                assert!(
                    test.ctx.host.validate().is_ok(),
                    "failed in test [{}] {} while validating context {:?}",
                    sub_title,
                    test.name,
                    test.ctx
                );

                let next_height = current_height.increment();
                assert_eq!(
                    test.ctx.latest_height(),
                    next_height,
                    "failed while increasing height for context {:?}",
                    test.ctx
                );

                assert_eq!(
                    test.ctx
                        .host
                        .get_block(&current_height)
                        .expect("Never fails")
                        .height(),
                    current_height,
                    "failed while fetching height {:?} of context {:?}",
                    current_height,
                    test.ctx
                );
            }
        }

        run_tests::<MockHost>("Mock Host");
        run_tests::<TendermintHost>("Synthetic TM Host");
    }
}
