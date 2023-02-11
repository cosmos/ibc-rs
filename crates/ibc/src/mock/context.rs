//! Implementation of a global context mock. Used in testing handlers of all IBC modules.

use crate::clients::ics07_tendermint::TENDERMINT_CLIENT_TYPE;
use crate::core::ics24_host::path::{
    AcksPath, ChannelEndsPath, ClientConsensusStatePath, CommitmentsPath, ReceiptsPath,
    SeqAcksPath, SeqRecvsPath, SeqSendsPath,
};
use crate::prelude::*;

use alloc::collections::btree_map::BTreeMap;
use alloc::sync::Arc;
use core::borrow::Borrow;
use core::cmp::min;
use core::fmt::{Debug, Formatter};
use core::ops::{Add, Sub};
use core::time::Duration;
use parking_lot::Mutex;

use ibc_proto::google::protobuf::Any;
use sha2::Digest;
use tracing::debug;

use crate::clients::ics07_tendermint::client_state::test_util::get_dummy_tendermint_client_state;
use crate::clients::ics07_tendermint::client_state::ClientState as TmClientState;
use crate::core::context::ContextError;
use crate::core::context::Router as NewRouter;
use crate::core::ics02_client::client_state::ClientState;
use crate::core::ics02_client::client_type::ClientType;
use crate::core::ics02_client::consensus_state::ConsensusState;
use crate::core::ics02_client::context::{ClientKeeper, ClientReader};
use crate::core::ics02_client::error::ClientError;
use crate::core::ics02_client::header::Header;
use crate::core::ics03_connection::connection::ConnectionEnd;
use crate::core::ics03_connection::context::{ConnectionKeeper, ConnectionReader};
use crate::core::ics03_connection::error::ConnectionError;
use crate::core::ics04_channel::channel::ChannelEnd;
use crate::core::ics04_channel::commitment::{AcknowledgementCommitment, PacketCommitment};
use crate::core::ics04_channel::context::{ChannelKeeper, ChannelReader};
use crate::core::ics04_channel::error::{ChannelError, PacketError};
use crate::core::ics04_channel::packet::{Receipt, Sequence};
use crate::core::ics05_port::context::PortReader;
use crate::core::ics05_port::error::PortError;
use crate::core::ics23_commitment::commitment::CommitmentPrefix;
use crate::core::ics24_host::identifier::{ChainId, ChannelId, ClientId, ConnectionId, PortId};
use crate::core::ics26_routing::context::{Module, ModuleId, Router, RouterBuilder, RouterContext};
use crate::core::ics26_routing::handler::{deliver, dispatch, MsgReceipt};
use crate::core::ics26_routing::msgs::MsgEnvelope;
use crate::core::ValidationContext;
use crate::events::IbcEvent;
use crate::mock::client_state::{
    client_type as mock_client_type, MockClientRecord, MockClientState,
};
use crate::mock::consensus_state::MockConsensusState;
use crate::mock::header::MockHeader;
use crate::mock::host::{HostBlock, HostType};
use crate::mock::ics18_relayer::context::RelayerContext;
use crate::mock::ics18_relayer::error::RelayerError;
use crate::signer::Signer;
use crate::timestamp::Timestamp;
use crate::Height;

use super::client_state::MOCK_CLIENT_TYPE;

pub const DEFAULT_BLOCK_TIME_SECS: u64 = 3;

/// A context implementing the dependencies necessary for testing any IBC module.
#[derive(Debug)]
pub struct MockContext {
    /// The type of host chain underlying this mock context.
    host_chain_type: HostType,

    /// Host chain identifier.
    host_chain_id: ChainId,

    /// Maximum size for the history of the host chain. Any block older than this is pruned.
    max_history_size: usize,

    /// The chain of blocks underlying this context. A vector of size up to `max_history_size`
    /// blocks, ascending order by their height (latest block is on the last position).
    history: Vec<HostBlock>,

    /// Average time duration between blocks
    block_time: Duration,

    /// An object that stores all IBC related data.
    pub ibc_store: Arc<Mutex<MockIbcStore>>,

    /// ICS26 router impl
    router: MockRouter,

    /// To implement ValidationContext Router
    new_router: BTreeMap<ModuleId, Arc<dyn Module>>,
}

/// Returns a MockContext with bare minimum initialization: no clients, no connections and no channels are
/// present, and the chain has Height(5). This should be used sparingly, mostly for testing the
/// creation of new domain objects.
impl Default for MockContext {
    fn default() -> Self {
        Self::new(
            ChainId::new("mockgaia".to_string(), 0),
            HostType::Mock,
            5,
            Height::new(0, 5).unwrap(),
        )
    }
}

/// A manual clone impl is provided because the tests are oblivious to the fact that the `ibc_store`
/// is a shared ptr.
impl Clone for MockContext {
    fn clone(&self) -> Self {
        let ibc_store = {
            let ibc_store = self.ibc_store.lock().clone();
            Arc::new(Mutex::new(ibc_store))
        };

        Self {
            host_chain_type: self.host_chain_type,
            host_chain_id: self.host_chain_id.clone(),
            max_history_size: self.max_history_size,
            history: self.history.clone(),
            block_time: self.block_time,
            ibc_store,
            router: self.router.clone(),
            new_router: self.new_router.clone(),
        }
    }
}

/// Implementation of internal interface for use in testing. The methods in this interface should
/// _not_ be accessible to any Ics handler.
impl MockContext {
    /// Creates a mock context. Parameter `max_history_size` determines how many blocks will
    /// the chain maintain in its history, which also determines the pruning window. Parameter
    /// `latest_height` determines the current height of the chain. This context
    /// has support to emulate two type of underlying chains: Mock or SyntheticTendermint.
    pub fn new(
        host_id: ChainId,
        host_type: HostType,
        max_history_size: usize,
        latest_height: Height,
    ) -> Self {
        assert_ne!(
            max_history_size, 0,
            "The chain must have a non-zero max_history_size"
        );

        assert_ne!(
            latest_height.revision_height(),
            0,
            "The chain must have a non-zero revision_height"
        );

        // Compute the number of blocks to store.
        let n = min(max_history_size as u64, latest_height.revision_height());

        assert_eq!(
            host_id.version(),
            latest_height.revision_number(),
            "The version in the chain identifier must match the version in the latest height"
        );

        let block_time = Duration::from_secs(DEFAULT_BLOCK_TIME_SECS);
        let next_block_timestamp = Timestamp::now().add(block_time).unwrap();
        MockContext {
            host_chain_type: host_type,
            host_chain_id: host_id.clone(),
            max_history_size,
            history: (0..n)
                .rev()
                .map(|i| {
                    // generate blocks with timestamps -> N, N - BT, N - 2BT, ...
                    // where N = now(), BT = block_time
                    HostBlock::generate_block(
                        host_id.clone(),
                        host_type,
                        latest_height.sub(i).unwrap().revision_height(),
                        next_block_timestamp
                            .sub(Duration::from_secs(DEFAULT_BLOCK_TIME_SECS * (i + 1)))
                            .unwrap(),
                    )
                })
                .collect(),
            block_time,
            ibc_store: Arc::new(Mutex::new(MockIbcStore::default())),
            router: Default::default(),
            new_router: BTreeMap::new(),
        }
    }

    /// Associates a client record to this context.
    /// Given a client id and a height, registers a new client in the context and also associates
    /// to this client a mock client state and a mock consensus state for height `height`. The type
    /// of this client is implicitly assumed to be Mock.
    pub fn with_client(self, client_id: &ClientId, height: Height) -> Self {
        self.with_client_parametrized(client_id, height, Some(mock_client_type()), Some(height))
    }

    /// Similar to `with_client`, this function associates a client record to this context, but
    /// additionally permits to parametrize two details of the client. If `client_type` is None,
    /// then the client will have type Mock, otherwise the specified type. If
    /// `consensus_state_height` is None, then the client will be initialized with a consensus
    /// state matching the same height as the client state (`client_state_height`).
    pub fn with_client_parametrized(
        self,
        client_id: &ClientId,
        client_state_height: Height,
        client_type: Option<ClientType>,
        consensus_state_height: Option<Height>,
    ) -> Self {
        let client_chain_id = self.host_chain_id.clone();
        self.with_client_parametrized_with_chain_id(
            client_chain_id,
            client_id,
            client_state_height,
            client_type,
            consensus_state_height,
        )
    }

    pub fn with_client_parametrized_with_chain_id(
        self,
        client_chain_id: ChainId,
        client_id: &ClientId,
        client_state_height: Height,
        client_type: Option<ClientType>,
        consensus_state_height: Option<Height>,
    ) -> Self {
        let cs_height = consensus_state_height.unwrap_or(client_state_height);

        let client_type = client_type.unwrap_or_else(mock_client_type);
        let (client_state, consensus_state) = if client_type.as_str() == MOCK_CLIENT_TYPE {
            (
                Some(MockClientState::new(MockHeader::new(client_state_height)).into_box()),
                MockConsensusState::new(MockHeader::new(cs_height)).into_box(),
            )
        } else if client_type.as_str() == TENDERMINT_CLIENT_TYPE {
            let light_block = HostBlock::generate_tm_block(
                client_chain_id,
                cs_height.revision_height(),
                Timestamp::now(),
            );

            let client_state =
                get_dummy_tendermint_client_state(light_block.header().clone()).into_box();

            // Return the tuple.
            (Some(client_state), light_block.into())
        } else {
            panic!("unknown client type")
        };
        // If it's a mock client, create the corresponding mock states.
        let consensus_states = vec![(cs_height, consensus_state)].into_iter().collect();

        debug!("consensus states: {:?}", consensus_states);

        let client_record = MockClientRecord {
            client_type,
            client_state,
            consensus_states,
        };
        self.ibc_store
            .lock()
            .clients
            .insert(client_id.clone(), client_record);
        self
    }

    pub fn with_client_parametrized_history(
        self,
        client_id: &ClientId,
        client_state_height: Height,
        client_type: Option<ClientType>,
        consensus_state_height: Option<Height>,
    ) -> Self {
        let client_chain_id = self.host_chain_id.clone();
        self.with_client_parametrized_history_with_chain_id(
            client_chain_id,
            client_id,
            client_state_height,
            client_type,
            consensus_state_height,
        )
    }

    pub(crate) fn with_client_parametrized_history_with_chain_id(
        self,
        client_chain_id: ChainId,
        client_id: &ClientId,
        client_state_height: Height,
        client_type: Option<ClientType>,
        consensus_state_height: Option<Height>,
    ) -> Self {
        let cs_height = consensus_state_height.unwrap_or(client_state_height);
        let prev_cs_height = cs_height.clone().sub(1).unwrap_or(client_state_height);

        let client_type = client_type.unwrap_or_else(mock_client_type);
        let now = Timestamp::now();

        let (client_state, consensus_state) = if client_type.as_str() == MOCK_CLIENT_TYPE {
            // If it's a mock client, create the corresponding mock states.
            (
                Some(MockClientState::new(MockHeader::new(client_state_height)).into_box()),
                MockConsensusState::new(MockHeader::new(cs_height)).into_box(),
            )
        } else if client_type.as_str() == TENDERMINT_CLIENT_TYPE {
            // If it's a Tendermint client, we need TM states.
            let light_block =
                HostBlock::generate_tm_block(client_chain_id, cs_height.revision_height(), now);

            let client_state =
                get_dummy_tendermint_client_state(light_block.header().clone()).into_box();

            // Return the tuple.
            (Some(client_state), light_block.into())
        } else {
            panic!("Unknown client type")
        };

        let prev_consensus_state = if client_type.as_str() == MOCK_CLIENT_TYPE {
            MockConsensusState::new(MockHeader::new(prev_cs_height)).into_box()
        } else if client_type.as_str() == TENDERMINT_CLIENT_TYPE {
            let light_block = HostBlock::generate_tm_block(
                self.host_chain_id.clone(),
                prev_cs_height.revision_height(),
                now.sub(self.block_time).unwrap(),
            );
            light_block.into()
        } else {
            panic!("Unknown client type")
        };

        let consensus_states = vec![
            (prev_cs_height, prev_consensus_state),
            (cs_height, consensus_state),
        ]
        .into_iter()
        .collect();

        debug!("consensus states: {:?}", consensus_states);

        let client_record = MockClientRecord {
            client_type,
            client_state,
            consensus_states,
        };

        self.ibc_store
            .lock()
            .clients
            .insert(client_id.clone(), client_record);
        self
    }

    /// Associates a connection to this context.
    pub fn with_connection(
        self,
        connection_id: ConnectionId,
        connection_end: ConnectionEnd,
    ) -> Self {
        self.ibc_store
            .lock()
            .connections
            .insert(connection_id, connection_end);
        self
    }

    /// Associates a channel (in an arbitrary state) to this context.
    pub fn with_channel(
        self,
        port_id: PortId,
        chan_id: ChannelId,
        channel_end: ChannelEnd,
    ) -> Self {
        let mut channels = self.ibc_store.lock().channels.clone();
        channels
            .entry(port_id)
            .or_default()
            .insert(chan_id, channel_end);
        self.ibc_store.lock().channels = channels;
        self
    }

    pub fn with_send_sequence(
        self,
        port_id: PortId,
        chan_id: ChannelId,
        seq_number: Sequence,
    ) -> Self {
        let mut next_sequence_send = self.ibc_store.lock().next_sequence_send.clone();
        next_sequence_send
            .entry(port_id)
            .or_default()
            .insert(chan_id, seq_number);
        self.ibc_store.lock().next_sequence_send = next_sequence_send;
        self
    }

    pub fn with_recv_sequence(
        self,
        port_id: PortId,
        chan_id: ChannelId,
        seq_number: Sequence,
    ) -> Self {
        let mut next_sequence_recv = self.ibc_store.lock().next_sequence_recv.clone();
        next_sequence_recv
            .entry(port_id)
            .or_default()
            .insert(chan_id, seq_number);
        self.ibc_store.lock().next_sequence_recv = next_sequence_recv;
        self
    }

    pub fn with_ack_sequence(
        self,
        port_id: PortId,
        chan_id: ChannelId,
        seq_number: Sequence,
    ) -> Self {
        let mut next_sequence_ack = self.ibc_store.lock().next_sequence_send.clone();
        next_sequence_ack
            .entry(port_id)
            .or_default()
            .insert(chan_id, seq_number);
        self.ibc_store.lock().next_sequence_ack = next_sequence_ack;
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
            let mut ctx = MockContext { ..self };
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
        self,
        port_id: PortId,
        chan_id: ChannelId,
        seq: Sequence,
        data: PacketCommitment,
    ) -> Self {
        let mut packet_commitment = self.ibc_store.lock().packet_commitment.clone();
        packet_commitment
            .entry(port_id)
            .or_default()
            .entry(chan_id)
            .or_default()
            .insert(seq, data);
        self.ibc_store.lock().packet_commitment = packet_commitment;
        self
    }

    pub fn with_router(self, router: MockRouter) -> Self {
        Self { router, ..self }
    }

    /// Accessor for a block of the local (host) chain from this context.
    /// Returns `None` if the block at the requested height does not exist.
    pub fn host_block(&self, target_height: &Height) -> Option<&HostBlock> {
        let target = target_height.revision_height() as usize;
        let latest = self.latest_height().revision_height() as usize;

        // Check that the block is not too advanced, nor has it been pruned.
        if (target > latest) || (target <= latest - self.history.len()) {
            None // Block for requested height does not exist in history.
        } else {
            Some(&self.history[self.history.len() + target - latest - 1])
        }
    }

    /// Triggers the advancing of the host chain, by extending the history of blocks (or headers).
    pub fn advance_host_chain_height(&mut self) {
        let latest_block = self.history.last().expect("history cannot be empty");
        let new_block = HostBlock::generate_block(
            self.host_chain_id.clone(),
            self.host_chain_type,
            latest_block.height().increment().revision_height(),
            latest_block.timestamp().add(self.block_time).unwrap(),
        );

        // Append the new header at the tip of the history.
        if self.history.len() >= self.max_history_size {
            // History is full, we rotate and replace the tip with the new header.
            self.history.rotate_left(1);
            self.history[self.max_history_size - 1] = new_block;
        } else {
            // History is not full yet.
            self.history.push(new_block);
        }
    }

    /// A datagram passes from the relayer to the IBC module (on host chain).
    /// Alternative method to `Ics18Context::send` that does not exercise any serialization.
    /// Used in testing the Ics18 algorithms, hence this may return a Ics18Error.
    pub fn deliver(&mut self, msg: MsgEnvelope) -> Result<(), RelayerError> {
        dispatch(self, msg).map_err(RelayerError::TransactionFailed)?;
        // Create a new block.
        self.advance_host_chain_height();
        Ok(())
    }

    /// Validates this context. Should be called after the context is mutated by a test.
    pub fn validate(&self) -> Result<(), String> {
        // Check that the number of entries is not higher than window size.
        if self.history.len() > self.max_history_size {
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

    pub fn add_port(&mut self, port_id: PortId) {
        let module_id = ModuleId::new(format!("module{port_id}").into()).unwrap();
        self.ibc_store
            .lock()
            .port_to_module
            .insert(port_id, module_id);
    }

    pub fn scope_port_to_module(&mut self, port_id: PortId, module_id: ModuleId) {
        self.ibc_store
            .lock()
            .port_to_module
            .insert(port_id, module_id);
    }

    pub fn latest_client_states(&self, client_id: &ClientId) -> Box<dyn ClientState> {
        self.ibc_store.lock().clients[client_id]
            .client_state
            .as_ref()
            .unwrap()
            .clone()
    }

    pub fn latest_consensus_states(
        &self,
        client_id: &ClientId,
        height: &Height,
    ) -> Box<dyn ConsensusState> {
        dyn_clone::clone_box(
            self.ibc_store.lock().clients[client_id]
                .consensus_states
                .get(height)
                .unwrap()
                .as_ref(),
        )
    }

    #[inline]
    fn latest_height(&self) -> Height {
        self.history
            .last()
            .expect("history cannot be empty")
            .height()
    }

    pub fn ibc_store_share(&self) -> Arc<Mutex<MockIbcStore>> {
        self.ibc_store.clone()
    }
}

type PortChannelIdMap<V> = BTreeMap<PortId, BTreeMap<ChannelId, V>>;

/// An object that stores all IBC related data.
#[derive(Clone, Debug, Default)]
pub struct MockIbcStore {
    /// The set of all clients, indexed by their id.
    pub clients: BTreeMap<ClientId, MockClientRecord>,

    /// Tracks the processed time for clients header updates
    pub client_processed_times: BTreeMap<(ClientId, Height), Timestamp>,

    /// Tracks the processed height for the clients
    pub client_processed_heights: BTreeMap<(ClientId, Height), Height>,

    /// Counter for the client identifiers, necessary for `increase_client_counter` and the
    /// `client_counter` methods.
    pub client_ids_counter: u64,

    /// Association between client ids and connection ids.
    pub client_connections: BTreeMap<ClientId, ConnectionId>,

    /// All the connections in the store.
    pub connections: BTreeMap<ConnectionId, ConnectionEnd>,

    /// Counter for connection identifiers (see `increase_connection_counter`).
    pub connection_ids_counter: u64,

    /// Association between connection ids and channel ids.
    pub connection_channels: BTreeMap<ConnectionId, Vec<(PortId, ChannelId)>>,

    /// Counter for channel identifiers (see `increase_channel_counter`).
    pub channel_ids_counter: u64,

    /// All the channels in the store. TODO Make new key PortId X ChanneId
    pub channels: PortChannelIdMap<ChannelEnd>,

    /// Tracks the sequence number for the next packet to be sent.
    pub next_sequence_send: PortChannelIdMap<Sequence>,

    /// Tracks the sequence number for the next packet to be received.
    pub next_sequence_recv: PortChannelIdMap<Sequence>,

    /// Tracks the sequence number for the next packet to be acknowledged.
    pub next_sequence_ack: PortChannelIdMap<Sequence>,

    pub packet_acknowledgement: PortChannelIdMap<BTreeMap<Sequence, AcknowledgementCommitment>>,

    /// Maps ports to the the module that owns it
    pub port_to_module: BTreeMap<PortId, ModuleId>,

    /// Constant-size commitments to packets data fields
    pub packet_commitment: PortChannelIdMap<BTreeMap<Sequence, PacketCommitment>>,

    // Used by unordered channel
    pub packet_receipt: PortChannelIdMap<BTreeMap<Sequence, Receipt>>,
}

#[derive(Default)]
pub struct MockRouterBuilder(MockRouter);

impl RouterBuilder for MockRouterBuilder {
    type Router = MockRouter;

    fn add_route(mut self, module_id: ModuleId, module: impl Module) -> Result<Self, String> {
        match self.0 .0.insert(module_id, Arc::new(module)) {
            None => Ok(self),
            Some(_) => Err("Duplicate module_id".to_owned()),
        }
    }

    fn build(self) -> Self::Router {
        self.0
    }
}

#[derive(Clone, Default)]
pub struct MockRouter(BTreeMap<ModuleId, Arc<dyn Module>>);

impl Debug for MockRouter {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self.0.keys().collect::<Vec<&ModuleId>>())
    }
}

impl Router for MockRouter {
    fn get_route_mut(&mut self, module_id: &impl Borrow<ModuleId>) -> Option<&mut dyn Module> {
        self.0.get_mut(module_id.borrow()).and_then(Arc::get_mut)
    }

    fn has_route(&self, module_id: &impl Borrow<ModuleId>) -> bool {
        self.0.get(module_id.borrow()).is_some()
    }
}

impl RouterContext for MockContext {
    type Router = MockRouter;

    fn router(&self) -> &Self::Router {
        &self.router
    }

    fn router_mut(&mut self) -> &mut Self::Router {
        &mut self.router
    }
}

impl PortReader for MockContext {
    fn lookup_module_by_port(&self, port_id: &PortId) -> Result<ModuleId, PortError> {
        match self.ibc_store.lock().port_to_module.get(port_id) {
            Some(mod_id) => Ok(mod_id.clone()),
            None => Err(PortError::UnknownPort {
                port_id: port_id.clone(),
            }),
        }
    }
}

impl ChannelReader for MockContext {
    fn channel_end(&self, chan_end_path: &ChannelEndsPath) -> Result<ChannelEnd, ChannelError> {
        match self
            .ibc_store
            .lock()
            .channels
            .get(&chan_end_path.0)
            .and_then(|map| map.get(&chan_end_path.1))
        {
            Some(channel_end) => Ok(channel_end.clone()),
            None => Err(ChannelError::ChannelNotFound {
                port_id: chan_end_path.0.clone(),
                channel_id: chan_end_path.1.clone(),
            }),
        }
    }

    fn connection_end(&self, cid: &ConnectionId) -> Result<ConnectionEnd, ChannelError> {
        ConnectionReader::connection_end(self, cid).map_err(ChannelError::Connection)
    }

    fn connection_channels(
        &self,
        cid: &ConnectionId,
    ) -> Result<Vec<(PortId, ChannelId)>, ChannelError> {
        match self.ibc_store.lock().connection_channels.get(cid) {
            Some(pcid) => Ok(pcid.clone()),
            None => Err(ChannelError::MissingChannel),
        }
    }

    fn client_state(&self, client_id: &ClientId) -> Result<Box<dyn ClientState>, ChannelError> {
        ClientReader::client_state(self, client_id)
            .map_err(|e| ChannelError::Connection(ConnectionError::Client(e)))
    }

    fn client_consensus_state(
        &self,
        client_cons_state_path: &ClientConsensusStatePath,
    ) -> Result<Box<dyn ConsensusState>, ChannelError> {
        ClientReader::consensus_state(self, client_cons_state_path)
            .map_err(|e| ChannelError::Connection(ConnectionError::Client(e)))
    }

    fn get_next_sequence_send(
        &self,
        seq_send_path: &SeqSendsPath,
    ) -> Result<Sequence, PacketError> {
        match self
            .ibc_store
            .lock()
            .next_sequence_send
            .get(&seq_send_path.0)
            .and_then(|map| map.get(&seq_send_path.1))
        {
            Some(sequence) => Ok(*sequence),
            None => Err(PacketError::MissingNextSendSeq {
                port_id: seq_send_path.0.clone(),
                channel_id: seq_send_path.1.clone(),
            }),
        }
    }

    fn get_next_sequence_recv(
        &self,
        seq_recv_path: &SeqRecvsPath,
    ) -> Result<Sequence, PacketError> {
        match self
            .ibc_store
            .lock()
            .next_sequence_recv
            .get(&seq_recv_path.0)
            .and_then(|map| map.get(&seq_recv_path.1))
        {
            Some(sequence) => Ok(*sequence),
            None => Err(PacketError::MissingNextRecvSeq {
                port_id: seq_recv_path.0.clone(),
                channel_id: seq_recv_path.1.clone(),
            }),
        }
    }

    fn get_next_sequence_ack(&self, seq_acks_path: &SeqAcksPath) -> Result<Sequence, PacketError> {
        match self
            .ibc_store
            .lock()
            .next_sequence_ack
            .get(&seq_acks_path.0)
            .and_then(|map| map.get(&seq_acks_path.1))
        {
            Some(sequence) => Ok(*sequence),
            None => Err(PacketError::MissingNextAckSeq {
                port_id: seq_acks_path.0.clone(),
                channel_id: seq_acks_path.1.clone(),
            }),
        }
    }

    fn get_packet_commitment(
        &self,
        commitment_path: &CommitmentsPath,
    ) -> Result<PacketCommitment, PacketError> {
        match self
            .ibc_store
            .lock()
            .packet_commitment
            .get(&commitment_path.port_id)
            .and_then(|map| map.get(&commitment_path.channel_id))
            .and_then(|map| map.get(&commitment_path.sequence))
        {
            Some(commitment) => Ok(commitment.clone()),
            None => Err(PacketError::PacketCommitmentNotFound {
                sequence: commitment_path.sequence,
            }),
        }
    }

    fn get_packet_receipt(&self, receipt_path: &ReceiptsPath) -> Result<Receipt, PacketError> {
        match self
            .ibc_store
            .lock()
            .packet_receipt
            .get(&receipt_path.port_id)
            .and_then(|map| map.get(&receipt_path.channel_id))
            .and_then(|map| map.get(&receipt_path.sequence))
        {
            Some(receipt) => Ok(receipt.clone()),
            None => Err(PacketError::PacketReceiptNotFound {
                sequence: receipt_path.sequence,
            }),
        }
    }

    fn get_packet_acknowledgement(
        &self,
        ack_path: &AcksPath,
    ) -> Result<AcknowledgementCommitment, PacketError> {
        match self
            .ibc_store
            .lock()
            .packet_acknowledgement
            .get(&ack_path.port_id)
            .and_then(|map| map.get(&ack_path.channel_id))
            .and_then(|map| map.get(&ack_path.sequence))
        {
            Some(ack) => Ok(ack.clone()),
            None => Err(PacketError::PacketAcknowledgementNotFound {
                sequence: ack_path.sequence,
            }),
        }
    }

    fn hash(&self, value: &[u8]) -> Vec<u8> {
        sha2::Sha256::digest(value).to_vec()
    }

    fn host_height(&self) -> Result<Height, ChannelError> {
        Ok(self.latest_height())
    }

    fn host_timestamp(&self) -> Result<Timestamp, ChannelError> {
        ClientReader::host_timestamp(self).map_err(|e| ChannelError::Other {
            description: e.to_string(),
        })
    }

    fn host_consensus_state(
        &self,
        height: &Height,
    ) -> Result<Box<dyn ConsensusState>, ChannelError> {
        ConnectionReader::host_consensus_state(self, height).map_err(ChannelError::Connection)
    }

    fn pending_host_consensus_state(&self) -> Result<Box<dyn ConsensusState>, ChannelError> {
        ClientReader::pending_host_consensus_state(self)
            .map_err(|e| ChannelError::Connection(ConnectionError::Client(e)))
    }

    fn client_update_time(
        &self,
        client_id: &ClientId,
        height: &Height,
    ) -> Result<Timestamp, ChannelError> {
        match self
            .ibc_store
            .lock()
            .client_processed_times
            .get(&(client_id.clone(), *height))
        {
            Some(time) => Ok(*time),
            None => Err(ChannelError::ProcessedTimeNotFound {
                client_id: client_id.clone(),
                height: *height,
            }),
        }
    }

    fn client_update_height(
        &self,
        client_id: &ClientId,
        height: &Height,
    ) -> Result<Height, ChannelError> {
        match self
            .ibc_store
            .lock()
            .client_processed_heights
            .get(&(client_id.clone(), *height))
        {
            Some(height) => Ok(*height),
            None => Err(ChannelError::ProcessedHeightNotFound {
                client_id: client_id.clone(),
                height: *height,
            }),
        }
    }

    fn channel_counter(&self) -> Result<u64, ChannelError> {
        Ok(self.ibc_store.lock().channel_ids_counter)
    }

    fn max_expected_time_per_block(&self) -> Duration {
        self.block_time
    }
}

impl ChannelKeeper for MockContext {
    fn store_packet_commitment(
        &mut self,
        port_id: PortId,
        channel_id: ChannelId,
        seq: Sequence,
        commitment: PacketCommitment,
    ) -> Result<(), PacketError> {
        self.ibc_store
            .lock()
            .packet_commitment
            .entry(port_id)
            .or_default()
            .entry(channel_id)
            .or_default()
            .insert(seq, commitment);
        Ok(())
    }

    fn store_packet_acknowledgement(
        &mut self,
        port_id: PortId,
        channel_id: ChannelId,
        seq: Sequence,
        ack_commitment: AcknowledgementCommitment,
    ) -> Result<(), PacketError> {
        self.ibc_store
            .lock()
            .packet_acknowledgement
            .entry(port_id)
            .or_default()
            .entry(channel_id)
            .or_default()
            .insert(seq, ack_commitment);
        Ok(())
    }

    fn delete_packet_acknowledgement(
        &mut self,
        port_id: &PortId,
        channel_id: &ChannelId,
        seq: &Sequence,
    ) -> Result<(), PacketError> {
        self.ibc_store
            .lock()
            .packet_acknowledgement
            .get_mut(port_id)
            .and_then(|map| map.get_mut(channel_id))
            .and_then(|map| map.remove(seq));
        Ok(())
    }

    fn store_connection_channels(
        &mut self,
        cid: ConnectionId,
        port_id: PortId,
        channel_id: ChannelId,
    ) -> Result<(), ChannelError> {
        self.ibc_store
            .lock()
            .connection_channels
            .entry(cid)
            .or_insert_with(Vec::new)
            .push((port_id, channel_id));
        Ok(())
    }

    fn store_channel(
        &mut self,
        port_id: PortId,
        channel_id: ChannelId,
        channel_end: ChannelEnd,
    ) -> Result<(), ChannelError> {
        self.ibc_store
            .lock()
            .channels
            .entry(port_id)
            .or_default()
            .insert(channel_id, channel_end);
        Ok(())
    }

    fn store_next_sequence_send(
        &mut self,
        port_id: PortId,
        channel_id: ChannelId,
        seq: Sequence,
    ) -> Result<(), PacketError> {
        self.ibc_store
            .lock()
            .next_sequence_send
            .entry(port_id)
            .or_default()
            .insert(channel_id, seq);
        Ok(())
    }

    fn store_next_sequence_recv(
        &mut self,
        port_id: PortId,
        channel_id: ChannelId,
        seq: Sequence,
    ) -> Result<(), PacketError> {
        self.ibc_store
            .lock()
            .next_sequence_recv
            .entry(port_id)
            .or_default()
            .insert(channel_id, seq);
        Ok(())
    }

    fn store_next_sequence_ack(
        &mut self,
        port_id: PortId,
        channel_id: ChannelId,
        seq: Sequence,
    ) -> Result<(), PacketError> {
        self.ibc_store
            .lock()
            .next_sequence_ack
            .entry(port_id)
            .or_default()
            .insert(channel_id, seq);
        Ok(())
    }

    fn increase_channel_counter(&mut self) {
        self.ibc_store.lock().channel_ids_counter += 1;
    }

    fn delete_packet_commitment(
        &mut self,
        port_id: &PortId,
        channel_id: &ChannelId,
        seq: &Sequence,
    ) -> Result<(), PacketError> {
        self.ibc_store
            .lock()
            .packet_commitment
            .get_mut(port_id)
            .and_then(|map| map.get_mut(channel_id))
            .and_then(|map| map.remove(seq));
        Ok(())
    }

    fn store_packet_receipt(
        &mut self,
        port_id: PortId,
        channel_id: ChannelId,
        seq: Sequence,
        receipt: Receipt,
    ) -> Result<(), PacketError> {
        self.ibc_store
            .lock()
            .packet_receipt
            .entry(port_id)
            .or_default()
            .entry(channel_id)
            .or_default()
            .insert(seq, receipt);
        Ok(())
    }
}

impl ConnectionReader for MockContext {
    fn connection_end(&self, cid: &ConnectionId) -> Result<ConnectionEnd, ConnectionError> {
        match self.ibc_store.lock().connections.get(cid) {
            Some(connection_end) => Ok(connection_end.clone()),
            None => Err(ConnectionError::ConnectionNotFound {
                connection_id: cid.clone(),
            }),
        }
    }

    fn client_state(&self, client_id: &ClientId) -> Result<Box<dyn ClientState>, ConnectionError> {
        // Forward method call to the Ics2 Client-specific method.
        ClientReader::client_state(self, client_id).map_err(ConnectionError::Client)
    }

    fn decode_client_state(
        &self,
        client_state: Any,
    ) -> Result<Box<dyn ClientState>, ConnectionError> {
        ClientReader::decode_client_state(self, client_state).map_err(ConnectionError::Client)
    }

    fn host_current_height(&self) -> Result<Height, ConnectionError> {
        Ok(self.latest_height())
    }

    fn host_oldest_height(&self) -> Result<Height, ConnectionError> {
        // history must be non-empty, so `self.history[0]` is valid
        Ok(self.history[0].height())
    }

    fn commitment_prefix(&self) -> CommitmentPrefix {
        CommitmentPrefix::try_from(b"mock".to_vec()).unwrap()
    }

    fn client_consensus_state(
        &self,
        client_cons_state_path: &ClientConsensusStatePath,
    ) -> Result<Box<dyn ConsensusState>, ConnectionError> {
        // Forward method call to the Ics2Client-specific method.
        ClientReader::consensus_state(self, client_cons_state_path).map_err(ConnectionError::Client)
    }

    fn host_consensus_state(
        &self,
        height: &Height,
    ) -> Result<Box<dyn ConsensusState>, ConnectionError> {
        ClientReader::host_consensus_state(self, height).map_err(ConnectionError::Client)
    }

    fn connection_counter(&self) -> Result<u64, ConnectionError> {
        Ok(self.ibc_store.lock().connection_ids_counter)
    }

    fn validate_self_client(&self, _counterparty_client_state: Any) -> Result<(), ConnectionError> {
        Ok(())
    }
}

impl ConnectionKeeper for MockContext {
    fn store_connection(
        &mut self,
        connection_id: ConnectionId,
        connection_end: ConnectionEnd,
    ) -> Result<(), ConnectionError> {
        self.ibc_store
            .lock()
            .connections
            .insert(connection_id, connection_end);
        Ok(())
    }

    fn store_connection_to_client(
        &mut self,
        connection_id: ConnectionId,
        client_id: ClientId,
    ) -> Result<(), ConnectionError> {
        self.ibc_store
            .lock()
            .client_connections
            .insert(client_id, connection_id);
        Ok(())
    }

    fn increase_connection_counter(&mut self) {
        self.ibc_store.lock().connection_ids_counter += 1;
    }
}

impl ClientReader for MockContext {
    fn client_type(&self, client_id: &ClientId) -> Result<ClientType, ClientError> {
        match self.ibc_store.lock().clients.get(client_id) {
            Some(client_record) => Ok(client_record.client_type.clone()),
            None => Err(ClientError::ClientNotFound {
                client_id: client_id.clone(),
            }),
        }
    }

    fn client_state(&self, client_id: &ClientId) -> Result<Box<dyn ClientState>, ClientError> {
        match self.ibc_store.lock().clients.get(client_id) {
            Some(client_record) => {
                client_record
                    .client_state
                    .clone()
                    .ok_or_else(|| ClientError::ClientNotFound {
                        client_id: client_id.clone(),
                    })
            }
            None => Err(ClientError::ClientNotFound {
                client_id: client_id.clone(),
            }),
        }
    }

    fn decode_client_state(&self, client_state: Any) -> Result<Box<dyn ClientState>, ClientError> {
        if let Ok(client_state) = TmClientState::try_from(client_state.clone()) {
            Ok(client_state.into_box())
        } else if let Ok(client_state) = MockClientState::try_from(client_state.clone()) {
            Ok(client_state.into_box())
        } else {
            Err(ClientError::UnknownClientStateType {
                client_state_type: client_state.type_url,
            })
        }
    }

    fn consensus_state(
        &self,
        client_cons_state_path: &ClientConsensusStatePath,
    ) -> Result<Box<dyn ConsensusState>, ClientError> {
        let height =
            Height::new(client_cons_state_path.epoch, client_cons_state_path.height).unwrap();
        match self
            .ibc_store
            .lock()
            .clients
            .get(&client_cons_state_path.client_id)
        {
            Some(client_record) => match client_record.consensus_states.get(&height) {
                Some(consensus_state) => Ok(consensus_state.clone()),
                None => Err(ClientError::ConsensusStateNotFound {
                    client_id: client_cons_state_path.client_id.clone(),
                    height,
                }),
            },
            None => Err(ClientError::ConsensusStateNotFound {
                client_id: client_cons_state_path.client_id.clone(),
                height,
            }),
        }
    }

    /// Search for the lowest consensus state higher than `height`.
    fn next_consensus_state(
        &self,
        next_client_cons_state_path: &ClientConsensusStatePath,
    ) -> Result<Option<Box<dyn ConsensusState>>, ClientError> {
        let ibc_store = self.ibc_store.lock();
        let client_record = ibc_store
            .clients
            .get(&next_client_cons_state_path.client_id)
            .ok_or_else(|| ClientError::ClientNotFound {
                client_id: next_client_cons_state_path.client_id.clone(),
            })?;

        let height = Height::new(
            next_client_cons_state_path.epoch,
            next_client_cons_state_path.height,
        )
        .unwrap();

        // Get the consensus state heights and sort them in ascending order.
        let mut heights: Vec<Height> = client_record.consensus_states.keys().cloned().collect();
        heights.sort();

        // Search for next state.
        for h in heights {
            if h > height {
                // unwrap should never happen, as the consensus state for h must exist
                return Ok(Some(
                    client_record.consensus_states.get(&h).unwrap().clone(),
                ));
            }
        }
        Ok(None)
    }

    /// Search for the highest consensus state lower than `height`.
    fn prev_consensus_state(
        &self,
        prev_client_cons_state_path: &ClientConsensusStatePath,
    ) -> Result<Option<Box<dyn ConsensusState>>, ClientError> {
        let ibc_store = self.ibc_store.lock();
        let client_record = ibc_store
            .clients
            .get(&prev_client_cons_state_path.client_id)
            .ok_or_else(|| ClientError::ClientNotFound {
                client_id: prev_client_cons_state_path.client_id.clone(),
            })?;

        let height = Height::new(
            prev_client_cons_state_path.epoch,
            prev_client_cons_state_path.height,
        )
        .unwrap();

        // Get the consensus state heights and sort them in descending order.
        let mut heights: Vec<Height> = client_record.consensus_states.keys().cloned().collect();
        heights.sort_by(|a, b| b.cmp(a));

        // Search for previous state.
        for h in heights {
            if h < height {
                // unwrap should never happen, as the consensus state for h must exist
                return Ok(Some(
                    client_record.consensus_states.get(&h).unwrap().clone(),
                ));
            }
        }
        Ok(None)
    }

    fn host_height(&self) -> Result<Height, ClientError> {
        Ok(self.latest_height())
    }

    fn host_timestamp(&self) -> Result<Timestamp, ClientError> {
        Ok(self
            .history
            .last()
            .expect("history cannot be empty")
            .timestamp()
            .add(self.block_time)
            .unwrap())
    }

    fn host_consensus_state(
        &self,
        height: &Height,
    ) -> Result<Box<dyn ConsensusState>, ClientError> {
        match self.host_block(height) {
            Some(block_ref) => Ok(block_ref.clone().into()),
            None => Err(ClientError::MissingLocalConsensusState { height: *height }),
        }
    }

    fn pending_host_consensus_state(&self) -> Result<Box<dyn ConsensusState>, ClientError> {
        Err(ClientError::ImplementationSpecific)
    }

    fn client_counter(&self) -> Result<u64, ClientError> {
        Ok(self.ibc_store.lock().client_ids_counter)
    }
}

impl ClientKeeper for MockContext {
    fn store_client_type(
        &mut self,
        client_id: ClientId,
        client_type: ClientType,
    ) -> Result<(), ClientError> {
        let mut ibc_store = self.ibc_store.lock();
        let client_record = ibc_store
            .clients
            .entry(client_id)
            .or_insert(MockClientRecord {
                client_type: client_type.clone(),
                consensus_states: Default::default(),
                client_state: Default::default(),
            });

        client_record.client_type = client_type;
        Ok(())
    }

    fn store_client_state(
        &mut self,
        client_id: ClientId,
        client_state: Box<dyn ClientState>,
    ) -> Result<(), ClientError> {
        let mut ibc_store = self.ibc_store.lock();
        let client_record = ibc_store
            .clients
            .entry(client_id)
            .or_insert(MockClientRecord {
                client_type: client_state.client_type(),
                consensus_states: Default::default(),
                client_state: Default::default(),
            });

        client_record.client_state = Some(client_state);
        Ok(())
    }

    fn store_consensus_state(
        &mut self,
        client_id: ClientId,
        height: Height,
        consensus_state: Box<dyn ConsensusState>,
    ) -> Result<(), ClientError> {
        let mut ibc_store = self.ibc_store.lock();
        let client_record = ibc_store
            .clients
            .entry(client_id)
            .or_insert(MockClientRecord {
                client_type: mock_client_type(),
                consensus_states: Default::default(),
                client_state: Default::default(),
            });

        client_record
            .consensus_states
            .insert(height, consensus_state);
        Ok(())
    }

    fn increase_client_counter(&mut self) {
        self.ibc_store.lock().client_ids_counter += 1
    }

    fn store_update_time(
        &mut self,
        client_id: ClientId,
        height: Height,
        timestamp: Timestamp,
    ) -> Result<(), ClientError> {
        let _ = self
            .ibc_store
            .lock()
            .client_processed_times
            .insert((client_id, height), timestamp);
        Ok(())
    }

    fn store_update_height(
        &mut self,
        client_id: ClientId,
        height: Height,
        host_height: Height,
    ) -> Result<(), ClientError> {
        let _ = self
            .ibc_store
            .lock()
            .client_processed_heights
            .insert((client_id, height), host_height);
        Ok(())
    }
}

impl RelayerContext for MockContext {
    fn query_latest_height(&self) -> Result<Height, RelayerError> {
        self.host_current_height().map_err(RelayerError::Connection)
    }

    fn query_client_full_state(&self, client_id: &ClientId) -> Option<Box<dyn ClientState>> {
        // Forward call to Ics2.
        ClientReader::client_state(self, client_id).ok()
    }

    fn query_latest_header(&self) -> Option<Box<dyn Header>> {
        let block_ref = self.host_block(&self.host_current_height().unwrap());
        block_ref.cloned().map(Header::into_box)
    }

    fn send(&mut self, msgs: Vec<Any>) -> Result<Vec<IbcEvent>, RelayerError> {
        // Forward call to Ics26 delivery method.
        let mut all_events = vec![];
        for msg in msgs {
            let MsgReceipt { mut events, .. } =
                deliver(self, msg).map_err(RelayerError::TransactionFailed)?;
            all_events.append(&mut events);
        }
        self.advance_host_chain_height(); // Advance chain height
        Ok(all_events)
    }

    fn signer(&self) -> Signer {
        "0CDA3F47EF3C4906693B170EF650EB968C5F4B2C".parse().unwrap()
    }
}

impl NewRouter for MockContext {
    fn get_route(&self, module_id: &ModuleId) -> Option<&dyn Module> {
        self.new_router.get(module_id).map(Arc::as_ref)
    }
    fn get_route_mut(&mut self, module_id: &ModuleId) -> Option<&mut dyn Module> {
        self.new_router.get_mut(module_id).and_then(Arc::get_mut)
    }

    fn has_route(&self, module_id: &ModuleId) -> bool {
        self.new_router.get(module_id).is_some()
    }

    fn lookup_module_by_port(&self, port_id: &PortId) -> Option<ModuleId> {
        <Self as PortReader>::lookup_module_by_port(self, port_id).ok()
    }
}

impl ValidationContext for MockContext {
    fn client_state(&self, client_id: &ClientId) -> Result<Box<dyn ClientState>, ContextError> {
        ClientReader::client_state(self, client_id).map_err(ContextError::ClientError)
    }

    fn decode_client_state(&self, client_state: Any) -> Result<Box<dyn ClientState>, ContextError> {
        ClientReader::decode_client_state(self, client_state).map_err(ContextError::ClientError)
    }

    fn consensus_state(
        &self,
        client_cons_state_path: &ClientConsensusStatePath,
    ) -> Result<Box<dyn ConsensusState>, ContextError> {
        ClientReader::consensus_state(self, client_cons_state_path)
            .map_err(ContextError::ClientError)
    }

    fn next_consensus_state(
        &self,
        next_client_cons_state_path: &ClientConsensusStatePath,
    ) -> Result<Option<Box<dyn ConsensusState>>, ContextError> {
        ClientReader::next_consensus_state(self, next_client_cons_state_path)
            .map_err(ContextError::ClientError)
    }

    fn prev_consensus_state(
        &self,
        prev_client_cons_state_path: &ClientConsensusStatePath,
    ) -> Result<Option<Box<dyn ConsensusState>>, ContextError> {
        ClientReader::prev_consensus_state(self, prev_client_cons_state_path)
            .map_err(ContextError::ClientError)
    }

    fn host_height(&self) -> Result<Height, ContextError> {
        Ok(self.latest_height())
    }

    fn pending_host_consensus_state(&self) -> Result<Box<dyn ConsensusState>, ContextError> {
        ClientReader::pending_host_consensus_state(self).map_err(ContextError::ClientError)
    }

    fn host_consensus_state(
        &self,
        height: &Height,
    ) -> Result<Box<dyn ConsensusState>, ContextError> {
        ConnectionReader::host_consensus_state(self, height).map_err(ContextError::ConnectionError)
    }

    fn client_counter(&self) -> Result<u64, ContextError> {
        ClientReader::client_counter(self).map_err(ContextError::ClientError)
    }

    fn connection_end(&self, conn_id: &ConnectionId) -> Result<ConnectionEnd, ContextError> {
        ConnectionReader::connection_end(self, conn_id).map_err(ContextError::ConnectionError)
    }

    fn validate_self_client(&self, _counterparty_client_state: Any) -> Result<(), ConnectionError> {
        Ok(())
    }

    fn commitment_prefix(&self) -> CommitmentPrefix {
        ConnectionReader::commitment_prefix(self)
    }

    fn connection_counter(&self) -> Result<u64, ContextError> {
        ConnectionReader::connection_counter(self).map_err(ContextError::ConnectionError)
    }

    fn channel_end(&self, chan_end_path: &ChannelEndsPath) -> Result<ChannelEnd, ContextError> {
        ChannelReader::channel_end(self, chan_end_path).map_err(ContextError::ChannelError)
    }

    fn connection_channels(
        &self,
        cid: &ConnectionId,
    ) -> Result<Vec<(PortId, ChannelId)>, ContextError> {
        ChannelReader::connection_channels(self, cid).map_err(ContextError::ChannelError)
    }

    fn get_next_sequence_send(
        &self,
        seq_send_path: &SeqSendsPath,
    ) -> Result<Sequence, ContextError> {
        ChannelReader::get_next_sequence_send(self, seq_send_path)
            .map_err(ContextError::PacketError)
    }

    fn get_next_sequence_recv(
        &self,
        seq_recv_path: &SeqRecvsPath,
    ) -> Result<Sequence, ContextError> {
        ChannelReader::get_next_sequence_recv(self, seq_recv_path)
            .map_err(ContextError::PacketError)
    }

    fn get_next_sequence_ack(&self, seq_ack_path: &SeqAcksPath) -> Result<Sequence, ContextError> {
        ChannelReader::get_next_sequence_ack(self, seq_ack_path).map_err(ContextError::PacketError)
    }

    fn get_packet_commitment(
        &self,
        commitment_path: &CommitmentsPath,
    ) -> Result<PacketCommitment, ContextError> {
        ChannelReader::get_packet_commitment(self, commitment_path)
            .map_err(ContextError::PacketError)
    }

    fn get_packet_receipt(&self, receipt_path: &ReceiptsPath) -> Result<Receipt, ContextError> {
        ChannelReader::get_packet_receipt(self, receipt_path).map_err(ContextError::PacketError)
    }

    fn get_packet_acknowledgement(
        &self,
        ack_path: &AcksPath,
    ) -> Result<AcknowledgementCommitment, ContextError> {
        ChannelReader::get_packet_acknowledgement(self, ack_path).map_err(ContextError::PacketError)
    }

    fn hash(&self, value: &[u8]) -> Vec<u8> {
        sha2::Sha256::digest(value).to_vec()
    }

    fn client_update_time(
        &self,
        client_id: &ClientId,
        height: &Height,
    ) -> Result<Timestamp, ContextError> {
        ChannelReader::client_update_time(self, client_id, height)
            .map_err(ContextError::ChannelError)
    }

    fn client_update_height(
        &self,
        client_id: &ClientId,
        height: &Height,
    ) -> Result<Height, ContextError> {
        ChannelReader::client_update_height(self, client_id, height)
            .map_err(ContextError::ChannelError)
    }

    fn channel_counter(&self) -> Result<u64, ContextError> {
        ChannelReader::channel_counter(self).map_err(ContextError::ChannelError)
    }

    fn max_expected_time_per_block(&self) -> Duration {
        ChannelReader::max_expected_time_per_block(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_log::test;

    use alloc::str::FromStr;

    use crate::core::ics04_channel::channel::{Counterparty, Order};
    use crate::core::ics04_channel::error::ChannelError;
    use crate::core::ics04_channel::handler::ModuleExtras;
    use crate::core::ics04_channel::msgs::acknowledgement::Acknowledgement;
    use crate::core::ics04_channel::packet::Packet;
    use crate::core::ics04_channel::Version;
    use crate::core::ics24_host::identifier::ChainId;
    use crate::core::ics24_host::identifier::{ChannelId, ConnectionId, PortId};
    use crate::core::ics26_routing::context::{
        Module, ModuleId, ModuleOutputBuilder, Router, RouterBuilder,
    };
    use crate::mock::context::MockContext;
    use crate::mock::context::MockRouterBuilder;
    use crate::mock::host::HostType;
    use crate::signer::Signer;
    use crate::test_utils::get_dummy_bech32_account;
    use crate::Height;

    #[test]
    fn test_history_manipulation() {
        pub struct Test {
            name: String,
            ctx: MockContext,
        }
        let cv = 1; // The version to use for all chains.

        let tests: Vec<Test> = vec![
            Test {
                name: "Empty history, small pruning window".to_string(),
                ctx: MockContext::new(
                    ChainId::new("mockgaia".to_string(), cv),
                    HostType::Mock,
                    2,
                    Height::new(cv, 1).unwrap(),
                ),
            },
            Test {
                name: "[Synthetic TM host] Empty history, small pruning window".to_string(),
                ctx: MockContext::new(
                    ChainId::new("mocksgaia".to_string(), cv),
                    HostType::SyntheticTendermint,
                    2,
                    Height::new(cv, 1).unwrap(),
                ),
            },
            Test {
                name: "Large pruning window".to_string(),
                ctx: MockContext::new(
                    ChainId::new("mockgaia".to_string(), cv),
                    HostType::Mock,
                    30,
                    Height::new(cv, 2).unwrap(),
                ),
            },
            Test {
                name: "[Synthetic TM host] Large pruning window".to_string(),
                ctx: MockContext::new(
                    ChainId::new("mocksgaia".to_string(), cv),
                    HostType::SyntheticTendermint,
                    30,
                    Height::new(cv, 2).unwrap(),
                ),
            },
            Test {
                name: "Small pruning window".to_string(),
                ctx: MockContext::new(
                    ChainId::new("mockgaia".to_string(), cv),
                    HostType::Mock,
                    3,
                    Height::new(cv, 30).unwrap(),
                ),
            },
            Test {
                name: "[Synthetic TM host] Small pruning window".to_string(),
                ctx: MockContext::new(
                    ChainId::new("mockgaia".to_string(), cv),
                    HostType::SyntheticTendermint,
                    3,
                    Height::new(cv, 30).unwrap(),
                ),
            },
            Test {
                name: "Small pruning window, small starting height".to_string(),
                ctx: MockContext::new(
                    ChainId::new("mockgaia".to_string(), cv),
                    HostType::Mock,
                    3,
                    Height::new(cv, 2).unwrap(),
                ),
            },
            Test {
                name: "[Synthetic TM host] Small pruning window, small starting height".to_string(),
                ctx: MockContext::new(
                    ChainId::new("mockgaia".to_string(), cv),
                    HostType::SyntheticTendermint,
                    3,
                    Height::new(cv, 2).unwrap(),
                ),
            },
            Test {
                name: "Large pruning window, large starting height".to_string(),
                ctx: MockContext::new(
                    ChainId::new("mockgaia".to_string(), cv),
                    HostType::Mock,
                    50,
                    Height::new(cv, 2000).unwrap(),
                ),
            },
            Test {
                name: "[Synthetic TM host] Large pruning window, large starting height".to_string(),
                ctx: MockContext::new(
                    ChainId::new("mockgaia".to_string(), cv),
                    HostType::SyntheticTendermint,
                    50,
                    Height::new(cv, 2000).unwrap(),
                ),
            },
        ];

        for mut test in tests {
            // All tests should yield a valid context after initialization.
            assert!(
                test.ctx.validate().is_ok(),
                "failed in test {} while validating context {:?}",
                test.name,
                test.ctx
            );

            let current_height = test.ctx.latest_height();

            // After advancing the chain's height, the context should still be valid.
            test.ctx.advance_host_chain_height();
            assert!(
                test.ctx.validate().is_ok(),
                "failed in test {} while validating context {:?}",
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
                test.ctx.host_block(&current_height).unwrap().height(),
                current_height,
                "failed while fetching height {:?} of context {:?}",
                current_height,
                test.ctx
            );
        }
    }

    #[test]
    fn test_router() {
        #[derive(Debug, Default)]
        struct FooModule {
            counter: usize,
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

            fn on_chan_open_init(
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

            fn on_chan_open_try(
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
                    Acknowledgement::try_from(vec![1u8]).unwrap(),
                )
            }

            fn on_recv_packet(
                &mut self,
                _output: &mut ModuleOutputBuilder,
                _packet: &Packet,
                _relayer: &Signer,
            ) -> Acknowledgement {
                self.counter += 1;

                Acknowledgement::try_from(vec![1u8]).unwrap()
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

            fn on_chan_open_init(
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

            fn on_chan_open_try(
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
                    Acknowledgement::try_from(vec![1u8]).unwrap(),
                )
            }

            fn on_recv_packet(
                &mut self,
                _output: &mut ModuleOutputBuilder,
                _packet: &Packet,
                _relayer: &Signer,
            ) -> Acknowledgement {
                Acknowledgement::try_from(vec![1u8]).unwrap()
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

        let r = MockRouterBuilder::default()
            .add_route("foomodule".parse().unwrap(), FooModule::default())
            .unwrap()
            .add_route("barmodule".parse().unwrap(), BarModule::default())
            .unwrap()
            .build();

        let mut ctx = MockContext::new(
            ChainId::new("mockgaia".to_string(), 1),
            HostType::Mock,
            1,
            Height::new(1, 1).unwrap(),
        )
        .with_router(r);

        let mut on_recv_packet_result = |module_id: &'static str| {
            let module_id = ModuleId::from_str(module_id).unwrap();
            let m = ctx.router.get_route_mut(&module_id).unwrap();
            let result = m.on_recv_packet(
                &mut ModuleOutputBuilder::new(),
                &Packet::default(),
                &get_dummy_bech32_account().parse().unwrap(),
            );
            (module_id, result)
        };

        let _results = vec![
            on_recv_packet_result("foomodule"),
            on_recv_packet_result("barmodule"),
        ];
    }
}
