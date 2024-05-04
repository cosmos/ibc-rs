use core::fmt::Debug;
use core::time::Duration;

use basecoin_store::context::ProvableStore;
use basecoin_store::impls::InMemoryStore;
use ibc::core::channel::types::channel::ChannelEnd;
use ibc::core::channel::types::commitment::PacketCommitment;
use ibc::core::client::context::client_state::ClientStateValidation;
use ibc::core::client::context::{ClientExecutionContext, ClientValidationContext};
use ibc::core::client::types::Height;
use ibc::core::connection::types::ConnectionEnd;
use ibc::core::entrypoint::{dispatch, execute, validate};
use ibc::core::handler::types::error::ContextError;
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
use crate::fixtures::core::context::TestContextConfig;
use crate::hosts::{HostClientState, MockHost, TendermintHost, TestBlock, TestHeader, TestHost};
use crate::relayer::error::RelayerError;
use crate::testapp::ibc::clients::{AnyClientState, AnyConsensusState};
use crate::testapp::ibc::core::router::MockRouter;
use crate::testapp::ibc::core::types::DEFAULT_BLOCK_TIME_SECS;

/// A context implementing the dependencies necessary for testing any IBC module.
#[derive(Debug)]
pub struct StoreGenericTestContext<S, H>
where
    S: ProvableStore + Debug,
    H: TestHost,
    HostClientState<H>: ClientStateValidation<MockIbcStore<S>>,
{
    /// The multi store of the context.
    /// This is where the IBC store root is stored at IBC commitment prefix.
    pub multi_store: S,

    /// The type of host chain underlying this mock context.
    pub host: H,

    /// An object that stores all IBC related data.
    pub ibc_store: MockIbcStore<S>,

    /// A router that can route messages to the appropriate IBC application.
    pub ibc_router: MockRouter,
}

/// A mock store type using basecoin-storage implementations.
pub type MockStore = InMemoryStore;
/// A [`StoreGenericTestContext`] using [`MockStore`].
pub type TestContext<H> = StoreGenericTestContext<MockStore, H>;
/// A [`StoreGenericTestContext`] using [`MockStore`] and [`MockHost`].
pub type MockContext = TestContext<MockHost>;
/// A [`StoreGenericTestContext`] using [`MockStore`] and [`TendermintHost`].
pub type TendermintContext = TestContext<TendermintHost>;

/// Returns a [`StoreGenericTestContext`] with bare minimum initialization: no clients, no connections, and no channels are
/// present, and the chain has Height(5). This should be used sparingly, mostly for testing the
/// creation of new domain objects.
impl<S, H> Default for StoreGenericTestContext<S, H>
where
    S: ProvableStore + Debug + Default,
    H: TestHost,
    HostClientState<H>: ClientStateValidation<MockIbcStore<S>>,
{
    fn default() -> Self {
        TestContextConfig::builder().build()
    }
}

/// Implementation of internal interface for use in testing. The methods in this interface should
/// _not_ be accessible to any ICS handler.
impl<S, H> StoreGenericTestContext<S, H>
where
    S: ProvableStore + Debug,
    H: TestHost,
    HostClientState<H>: ClientStateValidation<MockIbcStore<S>>,
{
    /// Returns a immutable reference to the IBC store.
    pub fn ibc_store(&self) -> &MockIbcStore<S> {
        &self.ibc_store
    }

    /// Returns a mutable reference to the IBC store.
    pub fn ibc_store_mut(&mut self) -> &mut MockIbcStore<S> {
        &mut self.ibc_store
    }

    /// Returns a immutable reference to the IBC router.
    pub fn ibc_router(&self) -> &MockRouter {
        &self.ibc_router
    }

    /// Returns a mutable reference to the IBC router.
    pub fn ibc_router_mut(&mut self) -> &mut MockRouter {
        &mut self.ibc_router
    }

    /// Returns the block at the given height from the host chain, if exists.
    pub fn host_block(&self, target_height: &Height) -> Option<H::Block> {
        self.host.get_block(target_height)
    }

    /// Returns the latest block from the host chain.
    pub fn query_latest_block(&self) -> Option<H::Block> {
        self.host.get_block(&self.latest_height())
    }

    /// Returns the latest height of client state for the given [`ClientId`].
    pub fn light_client_latest_height(&self, client_id: &ClientId) -> Height {
        self.ibc_store
            .client_state(client_id)
            .expect("client state exists")
            .latest_height()
    }

    /// Advances the host chain height to the given target height.
    pub fn advance_block_up_to_height(mut self, target_height: Height) -> Self {
        let latest_height = self.host.latest_height();
        if target_height.revision_number() != latest_height.revision_number() {
            panic!("Cannot advance history of the chain to a different revision number!")
        } else if target_height.revision_height() < latest_height.revision_height() {
            panic!("Cannot rewind history of the chain to a smaller revision height!")
        } else {
            // Repeatedly advance the host chain height till we hit the desired height
            while self.host.latest_height().revision_height() < target_height.revision_height() {
                self.advance_block_height()
            }
        }
        self
    }

    /// Advance the first height of the host chain by generating a genesis block.
    ///
    /// This method is exactly the same as [`Self::advance_genesis_height`].
    /// But it bootstraps the genesis block by height 1 and `genesis_time`.
    ///
    /// The method starts and ends with [`Self::end_block`] and [`Self::begin_block`], just
    /// like the [`Self::advance_block_height_with_params`], so that it can advance to next height
    /// i.e. height 2 - just by calling [`Self::advance_block_height_with_params`].
    pub fn advance_genesis_height(&mut self, genesis_time: Timestamp, params: &H::BlockParams) {
        self.end_block();

        // commit multi store
        let multi_store_commitment = self.multi_store.commit().expect("no error");

        // generate a genesis block
        // this is basically self.host.produce_block() but with
        // block height 1 and block timestamp `genesis_time`.
        let genesis_block =
            self.host
                .generate_block(multi_store_commitment, 1, genesis_time, params);

        // push the genesis block to the host
        self.host.push_block(genesis_block);

        self.begin_block();
    }

    /// Begin a new block on the context.
    ///
    /// This method commits the required metadata from the last block generation
    /// and consensus, and prepares the context for the next block. This includes
    /// the latest consensus state and the latest IBC commitment proof.
    pub fn begin_block(&mut self) {
        let consensus_state = self
            .host
            .latest_block()
            .into_header()
            .into_consensus_state()
            .into();

        let ibc_commitment_proof = self
            .multi_store
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

    /// End the current block on the context.
    ///
    /// This method commits the state of the IBC store and the host's multi store.
    pub fn end_block(&mut self) {
        // commit ibc store
        let ibc_store_commitment = self.ibc_store.end_block().expect("no error");

        // commit ibc store commitment in multi store
        self.multi_store
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

    /// Commit store state to the current block of the host chain by:
    /// - Committing the state to the context's multi store.
    /// - Generating a new block with the commitment.
    /// - Adding the generated block to the host's block history.
    pub fn commit_state_to_host(&mut self, block_time: Duration, params: &H::BlockParams) {
        // commit the multi store
        let multi_store_commitment = self.multi_store.commit().expect("no error");
        // generate a new block and add it to the block history
        self.host
            .commit_block(multi_store_commitment, block_time, params);
    }

    /// Advances the host chain height by ending the current block, producing a new block, and
    /// beginning the next block.
    pub fn advance_block_height_with_params(
        &mut self,
        block_time: Duration,
        params: &H::BlockParams,
    ) {
        self.end_block();
        self.commit_state_to_host(block_time, params);
        self.begin_block();
    }

    /// Convenience method to advance the host chain height using default parameters.
    pub fn advance_block_height(&mut self) {
        self.advance_block_height_with_params(
            Duration::from_secs(DEFAULT_BLOCK_TIME_SECS),
            &Default::default(),
        )
    }

    /// Returns the latest height of the host chain.
    pub fn latest_height(&self) -> Height {
        let latest_ibc_height = self.ibc_store.host_height().expect("Never fails");
        let latest_host_height = self.host.latest_height();
        assert_eq!(
            latest_ibc_height, latest_host_height,
            "The IBC store and the host chain must have the same height"
        );
        latest_ibc_height
    }

    /// Returns the latest timestamp of the host chain.
    pub fn latest_timestamp(&self) -> Timestamp {
        self.host.latest_block().timestamp()
    }

    /// Returns the timestamp at the given height.
    pub fn timestamp_at(&self, height: Height) -> Timestamp {
        self.host
            .get_block(&height)
            .expect("block exists")
            .timestamp()
    }

    /// Bootstraps the context with a client state and its corresponding [`ClientId`].
    pub fn with_client_state(mut self, client_id: &ClientId, client_state: AnyClientState) -> Self {
        let client_state_path = ClientStatePath::new(client_id.clone());
        self.ibc_store
            .store_client_state(client_state_path, client_state)
            .expect("error writing to store");
        self
    }

    /// Bootstraps the context with a consensus state and its corresponding [`ClientId`] and [`Height`].
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

    /// Generates a light client for the host by generating a client
    /// state, as well as generating consensus states for each
    /// consensus height.
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

    /// Bootstrap a light client with ClientState and its ConsensusState(s) to this context.
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

    /// Bootstraps a IBC connection to this context.
    ///
    /// This does not bootstrap any light client.
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

    /// Bootstraps a IBC channel to this context.
    ///
    /// This does not bootstrap any corresponding IBC connection or light client.
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

    /// Bootstraps a send sequence to this context.
    ///
    /// This does not bootstrap any corresponding IBC channel, connection or light client.
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

    /// Bootstraps a receive sequence to this context.
    ///
    /// This does not bootstrap any corresponding IBC channel, connection or light client.
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

    /// Bootstraps a ack sequence to this context.
    ///
    /// This does not bootstrap any corresponding IBC channel, connection or light client.
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

    /// Bootstraps a packet commitment to this context.
    ///
    /// This does not bootstrap any corresponding IBC channel, connection or light client.
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

    /// Calls [`validate`] function on [`MsgEnvelope`] using the context's IBC store and router.
    pub fn validate(&mut self, msg: MsgEnvelope) -> Result<(), ContextError> {
        validate(&self.ibc_store, &self.ibc_router, msg)
    }

    /// Calls [`execute`] function on [`MsgEnvelope`] using the context's IBC store and router.
    pub fn execute(&mut self, msg: MsgEnvelope) -> Result<(), ContextError> {
        execute(&mut self.ibc_store, &mut self.ibc_router, msg)
    }

    /// Calls [`dispatch`] function on [`MsgEnvelope`] using the context's IBC store and router.
    pub fn dispatch(&mut self, msg: MsgEnvelope) -> Result<(), ContextError> {
        dispatch(&mut self.ibc_store, &mut self.ibc_router, msg)
    }

    /// A datagram passes from the relayer to the IBC module (on host chain).
    /// Alternative method to `Ics18Context::send` that does not exercise any serialization.
    /// Used in testing the Ics18 algorithms, hence this may return a Ics18Error.
    pub fn deliver(&mut self, msg: MsgEnvelope) -> Result<(), RelayerError> {
        self.dispatch(msg)
            .map_err(RelayerError::TransactionFailed)?;
        // Create a new block.
        self.advance_block_height();
        Ok(())
    }

    /// Returns all the events that have been emitted by the context's IBC store.
    pub fn get_events(&self) -> Vec<IbcEvent> {
        self.ibc_store.events.lock().clone()
    }

    /// Returns all the logs that have been emitted by the context's IBC store.
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
            ctx: TestContext<H>,
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
                    ctx: TestContextConfig::builder()
                        .latest_height(Height::new(cv, 1).expect("Never fails"))
                        .build(),
                },
                Test {
                    name: "Large pruning window".to_string(),
                    ctx: TestContextConfig::builder()
                        .latest_height(Height::new(cv, 2).expect("Never fails"))
                        .build(),
                },
                Test {
                    name: "Small pruning window".to_string(),
                    ctx: TestContextConfig::builder()
                        .latest_height(Height::new(cv, 30).expect("Never fails"))
                        .build(),
                },
                Test {
                    name: "Small pruning window, small starting height".to_string(),
                    ctx: TestContextConfig::builder()
                        .latest_height(Height::new(cv, 2).expect("Never fails"))
                        .build(),
                },
                // This is disabled, as now we generate all the blocks till latest_height
                // Generating 2000 Tendermint blocks is slow.
                // Test {
                //     name: "Large pruning window, large starting height".to_string(),
                //     ctx: TestContextConfig::builder()
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
                test.ctx.advance_block_height();
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
