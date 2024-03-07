//! Implementation of a global context mock. Used in testing handlers of all IBC modules.

use alloc::sync::Arc;
use core::fmt::Debug;
use core::ops::{Add, Sub};
use core::time::Duration;

use basecoin_store::context::ProvableStore;
use basecoin_store::impls::{GrowingStore, InMemoryStore, RevertibleStore, SharedStore};
use basecoin_store::types::{BinStore, JsonStore, ProtobufStore, TypedSet, TypedStore};
use ibc::core::channel::types::channel::ChannelEnd;
use ibc::core::channel::types::commitment::{AcknowledgementCommitment, PacketCommitment};
use ibc::core::client::context::ClientExecutionContext;
use ibc::core::client::types::Height;
use ibc::core::connection::types::ConnectionEnd;
use ibc::core::entrypoint::dispatch;
use ibc::core::handler::types::events::IbcEvent;
use ibc::core::handler::types::msgs::MsgEnvelope;
use ibc::core::host::types::identifiers::{ChannelId, ClientId, ConnectionId, PortId, Sequence};
use ibc::core::host::types::path::{
    AckPath, ChannelEndPath, ClientConnectionPath, ClientConsensusStatePath, ClientStatePath,
    ClientUpdateHeightPath, ClientUpdateTimePath, CommitmentPath, ConnectionPath,
    NextChannelSequencePath, NextClientSequencePath, NextConnectionSequencePath, ReceiptPath,
    SeqAckPath, SeqRecvPath, SeqSendPath,
};
use ibc::core::host::{ExecutionContext, ValidationContext};
use ibc::core::primitives::prelude::*;
use ibc::core::primitives::Timestamp;
use ibc::core::router::router::Router;
use ibc_proto::google::protobuf::Any;
use ibc_proto::ibc::core::channel::v1::Channel as RawChannelEnd;
use ibc_proto::ibc::core::client::v1::Height as RawHeight;
use ibc_proto::ibc::core::connection::v1::ConnectionEnd as RawConnectionEnd;
use parking_lot::Mutex;
use typed_builder::TypedBuilder;

use crate::fixtures::core::context::MockContextConfig;
use crate::hosts::{TestBlock, TestHeader, TestHost};
use crate::relayer::error::RelayerError;
use crate::testapp::ibc::clients::{AnyClientState, AnyConsensusState};
use crate::testapp::ibc::utils::blocks_since;
pub const DEFAULT_BLOCK_TIME_SECS: u64 = 3;

/// An object that stores all IBC related data.
#[derive(Debug)]
pub struct MockIbcStore<S>
where
    S: ProvableStore + Debug,
{
    /// Handle to store instance.
    /// The module is guaranteed exclusive access to all paths in the store key-space.
    pub store: SharedStore<S>,
    /// A typed-store for next client counter sequence
    pub client_counter: JsonStore<SharedStore<S>, NextClientSequencePath, u64>,
    /// A typed-store for next connection counter sequence
    pub conn_counter: JsonStore<SharedStore<S>, NextConnectionSequencePath, u64>,
    /// A typed-store for next channel counter sequence
    pub channel_counter: JsonStore<SharedStore<S>, NextChannelSequencePath, u64>,
    /// Tracks the processed time for client updates
    pub client_processed_times: JsonStore<SharedStore<S>, ClientUpdateTimePath, Timestamp>,
    /// A typed-store to track the processed height for client updates
    pub client_processed_heights:
        ProtobufStore<SharedStore<S>, ClientUpdateHeightPath, Height, RawHeight>,
    /// A typed-store for AnyClientState
    pub client_state_store: ProtobufStore<SharedStore<S>, ClientStatePath, AnyClientState, Any>,
    /// A typed-store for AnyConsensusState
    pub consensus_state_store:
        ProtobufStore<SharedStore<S>, ClientConsensusStatePath, AnyConsensusState, Any>,
    /// A typed-store for ConnectionEnd
    pub connection_end_store:
        ProtobufStore<SharedStore<S>, ConnectionPath, ConnectionEnd, RawConnectionEnd>,
    /// A typed-store for ConnectionIds
    pub connection_ids_store: JsonStore<SharedStore<S>, ClientConnectionPath, Vec<ConnectionId>>,
    /// A typed-store for ChannelEnd
    pub channel_end_store: ProtobufStore<SharedStore<S>, ChannelEndPath, ChannelEnd, RawChannelEnd>,
    /// A typed-store for send sequences
    pub send_sequence_store: JsonStore<SharedStore<S>, SeqSendPath, Sequence>,
    /// A typed-store for receive sequences
    pub recv_sequence_store: JsonStore<SharedStore<S>, SeqRecvPath, Sequence>,
    /// A typed-store for ack sequences
    pub ack_sequence_store: JsonStore<SharedStore<S>, SeqAckPath, Sequence>,
    /// A typed-store for packet commitments
    pub packet_commitment_store: BinStore<SharedStore<S>, CommitmentPath, PacketCommitment>,
    /// A typed-store for packet receipts
    pub packet_receipt_store: TypedSet<SharedStore<S>, ReceiptPath>,
    /// A typed-store for packet ack
    pub packet_ack_store: BinStore<SharedStore<S>, AckPath, AcknowledgementCommitment>,
    /// Map of host consensus states
    pub consensus_states: Arc<Mutex<BTreeMap<u64, AnyConsensusState>>>,
    /// IBC Events
    pub events: Arc<Mutex<Vec<IbcEvent>>>,
    /// message logs
    pub logs: Arc<Mutex<Vec<String>>>,
}

impl<S> MockIbcStore<S>
where
    S: ProvableStore + Debug,
{
    pub fn new(store: S) -> Self {
        let shared_store = SharedStore::new(store);

        let mut client_counter = TypedStore::new(shared_store.clone());
        let mut conn_counter = TypedStore::new(shared_store.clone());
        let mut channel_counter = TypedStore::new(shared_store.clone());

        client_counter
            .set(NextClientSequencePath, 0)
            .expect("no error");

        conn_counter
            .set(NextConnectionSequencePath, 0)
            .expect("no error");

        channel_counter
            .set(NextChannelSequencePath, 0)
            .expect("no error");

        Self {
            client_counter,
            conn_counter,
            channel_counter,
            client_processed_times: TypedStore::new(shared_store.clone()),
            client_processed_heights: TypedStore::new(shared_store.clone()),
            consensus_states: Arc::new(Mutex::new(Default::default())),
            client_state_store: TypedStore::new(shared_store.clone()),
            consensus_state_store: TypedStore::new(shared_store.clone()),
            connection_end_store: TypedStore::new(shared_store.clone()),
            connection_ids_store: TypedStore::new(shared_store.clone()),
            channel_end_store: TypedStore::new(shared_store.clone()),
            send_sequence_store: TypedStore::new(shared_store.clone()),
            recv_sequence_store: TypedStore::new(shared_store.clone()),
            ack_sequence_store: TypedStore::new(shared_store.clone()),
            packet_commitment_store: TypedStore::new(shared_store.clone()),
            packet_receipt_store: TypedStore::new(shared_store.clone()),
            packet_ack_store: TypedStore::new(shared_store.clone()),
            events: Arc::new(Mutex::new(Vec::new())),
            logs: Arc::new(Mutex::new(Vec::new())),
            store: shared_store,
        }
    }
}

impl<S> Default for MockIbcStore<S>
where
    S: ProvableStore + Debug + Default,
{
    fn default() -> Self {
        Self::new(S::default())
    }
}

/// A context implementing the dependencies necessary for testing any IBC module.
#[derive(Debug)]
pub struct MockGenericContext<S, H>
where
    S: ProvableStore + Debug,
    H: TestHost,
{
    /// The type of host chain underlying this mock context.
    pub host: H,

    /// Maximum size for the history of the host chain. Any block older than this is pruned.
    pub max_history_size: u64,

    /// The chain of blocks underlying this context. A vector of size up to `max_history_size`
    /// blocks, ascending order by their height (latest block is on the last position).
    pub history: Vec<H::Block>,

    /// Average time duration between blocks
    pub block_time: Duration,

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
    #[builder(default = Duration::from_secs(128000))]
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

pub struct LightClientState<H: TestHost> {
    pub client_state: H::ClientState,
    pub consensus_states:
        BTreeMap<Height, <<H::Block as TestBlock>::Header as TestHeader>::ConsensusState>,
}

impl<H> Default for LightClientState<H>
where
    H: TestHost,
{
    fn default() -> Self {
        let context = MockContext::<H>::default();
        LightClientBuilder::init().context(&context).build()
    }
}

impl<H> LightClientState<H>
where
    H: TestHost,
{
    pub fn with_latest_height(height: Height) -> Self {
        let context = MockContextConfig::builder()
            .latest_height(height)
            .build::<MockContext<_>>();
        LightClientBuilder::init().context(&context).build()
    }
}

#[derive(TypedBuilder)]
#[builder(builder_method(name = init), build_method(into))]
pub struct LightClientBuilder<'a, H: TestHost> {
    context: &'a MockContext<H>,
    #[builder(default, setter(into))]
    consensus_heights: Vec<Height>,
    #[builder(default)]
    params: H::LightClientParams,
}

impl<'a, H> From<LightClientBuilder<'a, H>> for LightClientState<H>
where
    H: TestHost,
{
    fn from(builder: LightClientBuilder<'a, H>) -> Self {
        let LightClientBuilder {
            context,
            consensus_heights,
            params,
        } = builder;

        context.generate_light_client(consensus_heights, &params)
    }
}

/// Implementation of internal interface for use in testing. The methods in this interface should
/// _not_ be accessible to any Ics handler.
impl<S, H> MockGenericContext<S, H>
where
    S: ProvableStore + Debug,
    H: TestHost,
{
    pub fn with_client_state(mut self, client_id: &ClientId, client_state: AnyClientState) -> Self {
        let client_state_path = ClientStatePath::new(client_id.clone());
        self.store_client_state(client_state_path, client_state)
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
        self.store_consensus_state(consensus_state_path, consensus_state)
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

        let client_state = self.host.generate_client_state(
            self.host_block(&client_height)
                .expect("latest block exists"),
            client_params,
        );

        let consensus_states = consensus_heights
            .into_iter()
            .map(|height| {
                (
                    height,
                    self.host_block(&height)
                        .expect("block exists")
                        .clone()
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
        self.store_connection(&connection_path, connection_end)
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
        self.store_channel(&channel_end_path, channel_end)
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
        self.store_next_sequence_send(&seq_send_path, seq_number)
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
        self.store_next_sequence_recv(&seq_recv_path, seq_number)
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
        self.store_next_sequence_ack(&seq_ack_path, seq_number)
            .expect("error writing to store");
        self
    }

    pub fn with_height(self, target_height: Height) -> Self {
        let latest_height = self.latest_height();
        if target_height.revision_number() > latest_height.revision_number() {
            unimplemented!()
        } else if target_height.revision_number() < latest_height.revision_number() {
            panic!("Cannot rewind history of the chain to a smaller revision number!")
        } else if target_height.revision_height() < latest_height.revision_height() {
            panic!("Cannot rewind history of the chain to a smaller revision height!")
        } else if target_height.revision_height() > latest_height.revision_height() {
            // Repeatedly advance the host chain height till we hit the desired height
            let mut ctx = self;
            while ctx.latest_height().revision_height() < target_height.revision_height() {
                ctx.advance_host_chain_height()
            }
            ctx
        } else {
            // Both the revision number and height match
            self
        }
    }

    pub fn with_packet_commitment(
        mut self,
        port_id: PortId,
        chan_id: ChannelId,
        seq: Sequence,
        data: PacketCommitment,
    ) -> Self {
        let commitment_path = CommitmentPath::new(&port_id, &chan_id, seq);
        self.store_packet_commitment(&commitment_path, data)
            .expect("error writing to store");
        self
    }

    /// Accessor for a block of the local (host) chain from this context.
    /// Returns `None` if the block at the requested height does not exist.
    pub fn host_block(&self, target_height: &Height) -> Option<&H::Block> {
        let target = target_height.revision_height();
        let latest = self.latest_height().revision_height();

        // Check that the block is not too advanced, nor has it been pruned.
        if (target > latest) || (target <= latest - self.history.len() as u64) {
            None // Block for requested height does not exist in history.
        } else {
            Some(&self.history[self.history.len() + target as usize - latest as usize - 1])
        }
    }

    /// Triggers the advancing of the host chain, by extending the history of blocks (or headers).
    pub fn advance_host_chain_height(&mut self) {
        let latest_block = self.history.last().expect("history cannot be empty");

        let new_block = self.host.generate_block(
            latest_block.height().increment().revision_height(),
            latest_block
                .timestamp()
                .add(self.block_time)
                .expect("Never fails"),
            &H::BlockParams::default(),
        );

        // Append the new header at the tip of the history.
        if self.history.len() as u64 >= self.max_history_size {
            // History is full, we rotate and replace the tip with the new header.
            self.history.rotate_left(1);
            self.history[self.max_history_size as usize - 1] = new_block;
        } else {
            // History is not full yet.
            self.history.push(new_block);
        }
    }

    /// A datagram passes from the relayer to the IBC module (on host chain).
    /// Alternative method to `Ics18Context::send` that does not exercise any serialization.
    /// Used in testing the Ics18 algorithms, hence this may return a Ics18Error.
    pub fn deliver(
        &mut self,
        router: &mut impl Router,
        msg: MsgEnvelope,
    ) -> Result<(), RelayerError> {
        dispatch(self, router, msg).map_err(RelayerError::TransactionFailed)?;
        // Create a new block.
        self.advance_host_chain_height();
        Ok(())
    }

    /// Validates this context. Should be called after the context is mutated by a test.
    pub fn validate(&self) -> Result<(), String> {
        // Check that the number of entries is not higher than window size.
        if self.history.len() as u64 > self.max_history_size {
            return Err("too many entries".to_string());
        }

        // Check the content of the history.
        if !self.history.is_empty() {
            // Get the highest block.
            let lh = &self.history[self.history.len() - 1];
            // Check latest is properly updated with highest header height.
            if lh.height() != self.latest_height() {
                return Err("latest height is not updated".to_string());
            }
        }

        // Check that headers in the history are in sequential order.
        for i in 1..self.history.len() {
            let ph = &self.history[i - 1];
            let h = &self.history[i];
            if ph.height().increment() != h.height() {
                return Err("headers in history not sequential".to_string());
            }
        }
        Ok(())
    }

    pub fn latest_client_states(&self, client_id: &ClientId) -> AnyClientState {
        self.client_state(client_id)
            .expect("error reading from store")
    }

    pub fn latest_consensus_states(
        &self,
        client_id: &ClientId,
        height: &Height,
    ) -> AnyConsensusState {
        self.consensus_state(&ClientConsensusStatePath::new(
            client_id.clone(),
            height.revision_number(),
            height.revision_height(),
        ))
        .expect("error reading from store")
    }

    pub fn latest_height(&self) -> Height {
        self.host_height().expect("Never fails")
    }

    pub fn latest_timestamp(&self) -> Timestamp {
        self.host_block(&self.latest_height())
            .expect("Never fails")
            .timestamp()
    }

    pub fn timestamp_at(&self, height: Height) -> Timestamp {
        let n_blocks = blocks_since(self.latest_height(), height).expect("less or equal height");
        self.latest_timestamp()
            .sub(self.block_time * (n_blocks as u32))
            .expect("Never fails")
    }

    pub fn query_latest_header(&self) -> Option<&H::Block> {
        self.host_block(&self.latest_height())
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
    use ibc::core::channel::types::acknowledgement::Acknowledgement;
    use ibc::core::channel::types::channel::{Counterparty, Order};
    use ibc::core::channel::types::error::{ChannelError, PacketError};
    use ibc::core::channel::types::packet::Packet;
    use ibc::core::channel::types::Version;
    use ibc::core::host::types::identifiers::ChainId;
    use ibc::core::primitives::Signer;
    use ibc::core::router::module::Module;
    use ibc::core::router::types::module::{ModuleExtras, ModuleId};

    use super::*;
    use crate::fixtures::core::channel::PacketConfig;
    use crate::fixtures::core::signer::dummy_bech32_account;
    use crate::hosts::{MockHost, TendermintHost};
    use crate::testapp::ibc::core::router::MockRouter;

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
                    test.ctx.validate().is_ok(),
                    "failed in test [{}] {} while validating context {:?}",
                    sub_title,
                    test.name,
                    test.ctx
                );

                let current_height = test.ctx.latest_height();

                // After advancing the chain's height, the context should still be valid.
                test.ctx.advance_host_chain_height();
                assert!(
                    test.ctx.validate().is_ok(),
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
                        .host_block(&current_height)
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

    #[test]
    fn test_router() {
        #[derive(Debug, Default)]
        struct FooModule {
            counter: u64,
        }

        impl Module for FooModule {
            fn on_chan_open_init_validate(
                &self,
                _order: Order,
                _connection_hops: &[ConnectionId],
                _port_id: &PortId,
                _channel_id: &ChannelId,
                _counterparty: &Counterparty,
                version: &Version,
            ) -> Result<Version, ChannelError> {
                Ok(version.clone())
            }

            fn on_chan_open_init_execute(
                &mut self,
                _order: Order,
                _connection_hops: &[ConnectionId],
                _port_id: &PortId,
                _channel_id: &ChannelId,
                _counterparty: &Counterparty,
                version: &Version,
            ) -> Result<(ModuleExtras, Version), ChannelError> {
                Ok((ModuleExtras::empty(), version.clone()))
            }

            fn on_chan_open_try_validate(
                &self,
                _order: Order,
                _connection_hops: &[ConnectionId],
                _port_id: &PortId,
                _channel_id: &ChannelId,
                _counterparty: &Counterparty,
                counterparty_version: &Version,
            ) -> Result<Version, ChannelError> {
                Ok(counterparty_version.clone())
            }

            fn on_chan_open_try_execute(
                &mut self,
                _order: Order,
                _connection_hops: &[ConnectionId],
                _port_id: &PortId,
                _channel_id: &ChannelId,
                _counterparty: &Counterparty,
                counterparty_version: &Version,
            ) -> Result<(ModuleExtras, Version), ChannelError> {
                Ok((ModuleExtras::empty(), counterparty_version.clone()))
            }

            fn on_recv_packet_execute(
                &mut self,
                _packet: &Packet,
                _relayer: &Signer,
            ) -> (ModuleExtras, Acknowledgement) {
                self.counter += 1;

                (
                    ModuleExtras::empty(),
                    Acknowledgement::try_from(vec![1u8]).expect("Never fails"),
                )
            }

            fn on_timeout_packet_validate(
                &self,
                _packet: &Packet,
                _relayer: &Signer,
            ) -> Result<(), PacketError> {
                Ok(())
            }

            fn on_timeout_packet_execute(
                &mut self,
                _packet: &Packet,
                _relayer: &Signer,
            ) -> (ModuleExtras, Result<(), PacketError>) {
                (ModuleExtras::empty(), Ok(()))
            }

            fn on_acknowledgement_packet_validate(
                &self,
                _packet: &Packet,
                _acknowledgement: &Acknowledgement,
                _relayer: &Signer,
            ) -> Result<(), PacketError> {
                Ok(())
            }

            fn on_acknowledgement_packet_execute(
                &mut self,
                _packet: &Packet,
                _acknowledgement: &Acknowledgement,
                _relayer: &Signer,
            ) -> (ModuleExtras, Result<(), PacketError>) {
                (ModuleExtras::empty(), Ok(()))
            }
        }

        #[derive(Debug, Default)]
        struct BarModule;

        impl Module for BarModule {
            fn on_chan_open_init_validate(
                &self,
                _order: Order,
                _connection_hops: &[ConnectionId],
                _port_id: &PortId,
                _channel_id: &ChannelId,
                _counterparty: &Counterparty,
                version: &Version,
            ) -> Result<Version, ChannelError> {
                Ok(version.clone())
            }

            fn on_chan_open_init_execute(
                &mut self,
                _order: Order,
                _connection_hops: &[ConnectionId],
                _port_id: &PortId,
                _channel_id: &ChannelId,
                _counterparty: &Counterparty,
                version: &Version,
            ) -> Result<(ModuleExtras, Version), ChannelError> {
                Ok((ModuleExtras::empty(), version.clone()))
            }

            fn on_chan_open_try_validate(
                &self,
                _order: Order,
                _connection_hops: &[ConnectionId],
                _port_id: &PortId,
                _channel_id: &ChannelId,
                _counterparty: &Counterparty,
                counterparty_version: &Version,
            ) -> Result<Version, ChannelError> {
                Ok(counterparty_version.clone())
            }

            fn on_chan_open_try_execute(
                &mut self,
                _order: Order,
                _connection_hops: &[ConnectionId],
                _port_id: &PortId,
                _channel_id: &ChannelId,
                _counterparty: &Counterparty,
                counterparty_version: &Version,
            ) -> Result<(ModuleExtras, Version), ChannelError> {
                Ok((ModuleExtras::empty(), counterparty_version.clone()))
            }

            fn on_recv_packet_execute(
                &mut self,
                _packet: &Packet,
                _relayer: &Signer,
            ) -> (ModuleExtras, Acknowledgement) {
                (
                    ModuleExtras::empty(),
                    Acknowledgement::try_from(vec![1u8]).expect("Never fails"),
                )
            }

            fn on_timeout_packet_validate(
                &self,
                _packet: &Packet,
                _relayer: &Signer,
            ) -> Result<(), PacketError> {
                Ok(())
            }

            fn on_timeout_packet_execute(
                &mut self,
                _packet: &Packet,
                _relayer: &Signer,
            ) -> (ModuleExtras, Result<(), PacketError>) {
                (ModuleExtras::empty(), Ok(()))
            }

            fn on_acknowledgement_packet_validate(
                &self,
                _packet: &Packet,
                _acknowledgement: &Acknowledgement,
                _relayer: &Signer,
            ) -> Result<(), PacketError> {
                Ok(())
            }

            fn on_acknowledgement_packet_execute(
                &mut self,
                _packet: &Packet,
                _acknowledgement: &Acknowledgement,
                _relayer: &Signer,
            ) -> (ModuleExtras, Result<(), PacketError>) {
                (ModuleExtras::empty(), Ok(()))
            }
        }

        let mut router = MockRouter::default();
        router
            .add_route(ModuleId::new("foomodule".to_string()), FooModule::default())
            .expect("Never fails");
        router
            .add_route(ModuleId::new("barmodule".to_string()), BarModule)
            .expect("Never fails");

        let mut on_recv_packet_result = |module_id: &'static str| {
            let module_id = ModuleId::new(module_id.to_string());
            let m = router.get_route_mut(&module_id).expect("Never fails");

            let packet = PacketConfig::builder().build();

            let result = m.on_recv_packet_execute(&packet, &dummy_bech32_account().into());
            (module_id, result)
        };

        let _results = [
            on_recv_packet_result("foomodule"),
            on_recv_packet_result("barmodule"),
        ];
    }
}
