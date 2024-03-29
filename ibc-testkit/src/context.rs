use core::fmt::Debug;
use core::time::Duration;

use basecoin_store::context::ProvableStore;
use basecoin_store::impls::{GrowingStore, InMemoryStore, RevertibleStore};
use ibc::core::channel::types::channel::ChannelEnd;
use ibc::core::channel::types::commitment::PacketCommitment;
use ibc::core::client::context::client_state::ClientStateValidation;
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
use ibc::primitives::prelude::*;
use ibc::primitives::Timestamp;

use super::testapp::ibc::core::types::{LightClientState, MockIbcStore};
use crate::fixtures::core::context::MockContextConfig;
use crate::hosts::{HostClientState, TestBlock, TestHeader, TestHost};
use crate::relayer::error::RelayerError;
use crate::testapp::ibc::clients::{AnyClientState, AnyConsensusState};
use crate::testapp::ibc::core::router::MockRouter;
use crate::testapp::ibc::core::types::DEFAULT_BLOCK_TIME_SECS;

/// A context implementing the dependencies necessary for testing any IBC module.
#[derive(Debug)]
pub struct MockGenericContext<S, H>
where
    S: ProvableStore + Debug,
    H: TestHost,
    HostClientState<H>: ClientStateValidation<MockIbcStore<S>>,
{
    /// The main store of the context.
    pub main_store: S,

    /// The type of host chain underlying this mock context.
    pub host: H,

    /// An object that stores all IBC related data.
    pub ibc_store: MockIbcStore<S>,

    /// A router that can route messages to the appropriate IBC application.
    pub ibc_router: MockRouter,
}

pub type MockStore = RevertibleStore<GrowingStore<InMemoryStore>>;
pub type MockContext<H> = MockGenericContext<MockStore, H>;

/// Returns a MockContext with bare minimum initialization: no clients, no connections and no channels are
/// present, and the chain has Height(5). This should be used sparingly, mostly for testing the
/// creation of new domain objects.
impl<S, H> Default for MockGenericContext<S, H>
where
    S: ProvableStore + Debug + Default,
    H: TestHost,
    HostClientState<H>: ClientStateValidation<MockIbcStore<S>>,
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
    HostClientState<H>: ClientStateValidation<MockIbcStore<S>>,
{
    pub fn ibc_store(&self) -> &MockIbcStore<S> {
        &self.ibc_store
    }

    pub fn ibc_store_mut(&mut self) -> &mut MockIbcStore<S> {
        &mut self.ibc_store
    }

    pub fn host_block(&self, target_height: &Height) -> Option<H::Block> {
        self.host.get_block(target_height)
    }

    pub fn query_latest_block(&self) -> Option<H::Block> {
        self.host.get_block(&self.latest_height())
    }

    pub fn advance_block_up_to(mut self, target_height: Height) -> Self {
        let latest_height = self.host.latest_height();
        if target_height.revision_number() != latest_height.revision_number() {
            panic!("Cannot advance history of the chain to a different revision number!")
        } else if target_height.revision_height() < latest_height.revision_height() {
            panic!("Cannot rewind history of the chain to a smaller revision height!")
        } else {
            // Repeatedly advance the host chain height till we hit the desired height
            while self.host.latest_height().revision_height() < target_height.revision_height() {
                self.advance_block()
            }
        }
        self
    }

    pub fn generate_genesis_block(&mut self, genesis_time: Timestamp, params: &H::BlockParams) {
        self.end_block();

        // commit main store
        let main_store_commitment = self.main_store.commit().expect("no error");

        // generate a genesis block
        let genesis_block =
            self.host
                .generate_block(main_store_commitment, 1, genesis_time, params);

        // push the genesis block to the host
        self.host.push_block(genesis_block);

        self.begin_block();
    }

    pub fn begin_block(&mut self) {
        let consensus_state = self
            .host
            .latest_block()
            .into_header()
            .into_consensus_state()
            .into();

        let ibc_commitment_proof = self
            .main_store
            .get_proof(
                self.host.latest_height().revision_height().into(),
                &self
                    .ibc_store
                    .commitment_prefix()
                    .as_bytes()
                    .try_into()
                    .expect("valid utf8 prefix"),
            )
            .expect("no error");

        self.ibc_store.begin_block(
            self.host.latest_height().revision_height(),
            consensus_state,
            ibc_commitment_proof,
        );
    }

    pub fn end_block(&mut self) {
        // commit ibc store
        let ibc_store_commitment = self.ibc_store.end_block().expect("no error");

        // commit ibc store commitment in main store
        self.main_store
            .set(
                self.ibc_store
                    .commitment_prefix()
                    .as_bytes()
                    .try_into()
                    .expect("valid utf8 prefix"),
                ibc_store_commitment,
            )
            .expect("no error");
    }

    pub fn produce_block(&mut self, block_time: Duration, params: &H::BlockParams) {
        // commit main store
        let main_store_commitment = self.main_store.commit().expect("no error");
        // generate a new block
        self.host
            .advance_block(main_store_commitment, block_time, params);
    }

    pub fn advance_with_block_params(&mut self, block_time: Duration, params: &H::BlockParams) {
        self.end_block();
        self.produce_block(block_time, params);
        self.begin_block();
    }

    pub fn advance_block(&mut self) {
        self.advance_with_block_params(
            Duration::from_secs(DEFAULT_BLOCK_TIME_SECS),
            &Default::default(),
        )
    }

    pub fn prune_block_till(&mut self, height: &Height) {
        self.host.prune_block_till(height);
        self.ibc_store.prune_host_consensus_states_till(height);
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
        self.host.latest_block().timestamp()
    }

    pub fn timestamp_at(&self, height: Height) -> Timestamp {
        self.host
            .get_block(&height)
            .expect("block exists")
            .timestamp()
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
        mut consensus_heights: Vec<Height>,
        client_params: &H::LightClientParams,
    ) -> LightClientState<H> {
        let client_height = if let Some(&height) = consensus_heights.last() {
            height
        } else {
            consensus_heights.push(self.latest_height());
            self.latest_height()
        };

        let client_state = self
            .host
            .generate_client_state(&client_height, client_params);

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
    pub fn deliver(&mut self, msg: MsgEnvelope) -> Result<(), RelayerError> {
        dispatch(&mut self.ibc_store, &mut self.ibc_router, msg)
            .map_err(RelayerError::TransactionFailed)?;
        // Create a new block.
        self.advance_block();
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
    use ibc::core::client::context::consensus_state::ConsensusState;

    use super::*;
    use crate::hosts::{HostConsensusState, MockHost, TendermintHost};
    use crate::testapp::ibc::core::types::DefaultIbcStore;

    #[test]
    fn test_mock_history_validation() {
        pub struct Test<H: TestHost>
        where
            H: TestHost,
            HostConsensusState<H>: ConsensusState,
            HostClientState<H>: ClientStateValidation<DefaultIbcStore>,
        {
            name: String,
            ctx: MockContext<H>,
        }

        fn run_tests<H>(sub_title: &str)
        where
            H: TestHost,
            HostConsensusState<H>: ConsensusState,
            HostClientState<H>: ClientStateValidation<DefaultIbcStore>,
        {
            let cv = 0; // The version to use for all chains.

            let tests: Vec<Test<H>> = vec![
                Test {
                    name: "Empty history, small pruning window".to_string(),
                    ctx: MockContextConfig::builder()
                        .latest_height(Height::new(cv, 1).expect("Never fails"))
                        .build(),
                },
                Test {
                    name: "Large pruning window".to_string(),
                    ctx: MockContextConfig::builder()
                        .latest_height(Height::new(cv, 2).expect("Never fails"))
                        .build(),
                },
                Test {
                    name: "Small pruning window".to_string(),
                    ctx: MockContextConfig::builder()
                        .latest_height(Height::new(cv, 30).expect("Never fails"))
                        .build(),
                },
                Test {
                    name: "Small pruning window, small starting height".to_string(),
                    ctx: MockContextConfig::builder()
                        .latest_height(Height::new(cv, 2).expect("Never fails"))
                        .build(),
                },
                // This is disabled, as now we generate all the blocks till latest_height
                // Generating 2000 Tendermint blocks is slow.
                // Test {
                //     name: "Large pruning window, large starting height".to_string(),
                //     ctx: MockContextConfig::builder()
                //         .latest_height(Height::new(cv, 2000).expect("Never fails"))
                //         .build(),
                // },
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
                test.ctx.advance_block();
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
