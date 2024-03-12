//! Implementation of a global context mock. Used in testing handlers of all IBC modules.

use alloc::sync::Arc;
use core::cmp::min;
use core::fmt::Debug;
use core::ops::{Add, Sub};
use core::time::Duration;

use ibc::clients::tendermint::client_state::ClientState as TmClientState;
use ibc::clients::tendermint::types::TENDERMINT_CLIENT_TYPE;
use ibc::core::channel::types::channel::ChannelEnd;
use ibc::core::channel::types::commitment::{AcknowledgementCommitment, PacketCommitment};
use ibc::core::channel::types::packet::Receipt;
use ibc::core::client::types::Height;
use ibc::core::connection::types::ConnectionEnd;
use ibc::core::entrypoint::dispatch;
use ibc::core::handler::types::events::IbcEvent;
use ibc::core::handler::types::msgs::MsgEnvelope;
use ibc::core::host::types::identifiers::{
    ChainId, ChannelId, ClientId, ClientType, ConnectionId, PortId, Sequence,
};
use ibc::core::host::ValidationContext;
use ibc::core::primitives::prelude::*;
use ibc::core::primitives::Timestamp;
use ibc::core::router::router::Router;
use parking_lot::Mutex;
use tendermint_testgen::Validator as TestgenValidator;
use typed_builder::TypedBuilder;

use super::client_ctx::{MockClientRecord, PortChannelIdMap};
use crate::fixtures::clients::tendermint::ClientStateConfig as TmClientStateConfig;
use crate::fixtures::core::context::MockContextConfig;
use crate::hosts::block::{HostBlock, HostType};
use crate::relayer::error::RelayerError;
use crate::testapp::ibc::clients::mock::client_state::{
    client_type as mock_client_type, MockClientState, MOCK_CLIENT_TYPE,
};
use crate::testapp::ibc::clients::mock::consensus_state::MockConsensusState;
use crate::testapp::ibc::clients::mock::header::MockHeader;
use crate::testapp::ibc::clients::{AnyClientState, AnyConsensusState};
pub const DEFAULT_BLOCK_TIME_SECS: u64 = 3;

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

    /// All the channels in the store. TODO Make new key PortId X ChannelId
    pub channels: PortChannelIdMap<ChannelEnd>,

    /// Tracks the sequence number for the next packet to be sent.
    pub next_sequence_send: PortChannelIdMap<Sequence>,

    /// Tracks the sequence number for the next packet to be received.
    pub next_sequence_recv: PortChannelIdMap<Sequence>,

    /// Tracks the sequence number for the next packet to be acknowledged.
    pub next_sequence_ack: PortChannelIdMap<Sequence>,

    pub packet_acknowledgement: PortChannelIdMap<BTreeMap<Sequence, AcknowledgementCommitment>>,

    /// Constant-size commitments to packets data fields
    pub packet_commitment: PortChannelIdMap<BTreeMap<Sequence, PacketCommitment>>,

    /// Used by unordered channel
    pub packet_receipt: PortChannelIdMap<BTreeMap<Sequence, Receipt>>,

    /// Emitted IBC events in order
    pub events: Vec<IbcEvent>,

    /// Logs of the IBC module
    pub logs: Vec<String>,
}

/// A context implementing the dependencies necessary for testing any IBC module.
#[derive(Debug)]
pub struct MockContext {
    /// The type of host chain underlying this mock context.
    pub host_chain_type: HostType,

    /// Host chain identifier.
    pub host_chain_id: ChainId,

    /// Maximum size for the history of the host chain. Any block older than this is pruned.
    pub max_history_size: u64,

    /// The chain of blocks underlying this context. A vector of size up to `max_history_size`
    /// blocks, ascending order by their height (latest block is on the last position).
    pub history: Vec<HostBlock>,

    /// Average time duration between blocks
    pub block_time: Duration,

    /// An object that stores all IBC related data.
    pub ibc_store: Arc<Mutex<MockIbcStore>>,
}

#[derive(Debug, TypedBuilder)]
pub struct MockClientConfig {
    #[builder(default = ChainId::new("mockZ-1").expect("no error"))]
    client_chain_id: ChainId,
    #[builder(default = ClientId::new("07-tendermint", 0).expect("no error"))]
    client_id: ClientId,
    #[builder(default = mock_client_type())]
    client_type: ClientType,
    latest_height: Height,
    #[builder(default)]
    consensus_state_heights: Vec<Height>,
    #[builder(default = Timestamp::now())]
    latest_timestamp: Timestamp,

    #[builder(default = Duration::from_secs(64000))]
    trusting_period: Duration,
    #[builder(default = Duration::from_millis(3000))]
    max_clock_drift: Duration,
    #[builder(default = Duration::from_secs(128_000))]
    unbonding_period: Duration,
}

/// Returns a MockContext with bare minimum initialization: no clients, no connections and no channels are
/// present, and the chain has Height(5). This should be used sparingly, mostly for testing the
/// creation of new domain objects.
impl Default for MockContext {
    fn default() -> Self {
        MockContextConfig::builder().build()
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
    #[deprecated(
        since = "0.50.0",
        note = "Please use `MockContextConfig::builder().build()` instead"
    )]
    pub fn new(
        host_id: ChainId,
        host_type: HostType,
        max_history_size: u64,
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
        let n = min(max_history_size, latest_height.revision_height());

        assert_eq!(
            host_id.revision_number(),
            latest_height.revision_number(),
            "The version in the chain identifier must match the version in the latest height"
        );

        let block_time = Duration::from_secs(DEFAULT_BLOCK_TIME_SECS);
        let next_block_timestamp = Timestamp::now().add(block_time).expect("Never fails");
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
                        latest_height.sub(i).expect("Never fails").revision_height(),
                        next_block_timestamp
                            .sub(Duration::from_secs(DEFAULT_BLOCK_TIME_SECS * (i + 1)))
                            .expect("Never fails"),
                    )
                })
                .collect(),
            block_time,
            ibc_store: Arc::new(Mutex::new(MockIbcStore::default())),
        }
    }

    /// Same as [Self::new] but with custom validator sets for each block.
    /// Note: the validator history is used accordingly for current validator set and next validator set.
    /// `validator_history[i]` and `validator_history[i+1]` is i'th block's current and next validator set.
    /// The number of blocks will be `validator_history.len() - 1` due to the above.
    #[deprecated(
        since = "0.50.0",
        note = "Please use `MockContextConfig::builder().build()` instead"
    )]
    pub fn new_with_validator_history(
        host_id: ChainId,
        host_type: HostType,
        validator_history: &[Vec<TestgenValidator>],
        latest_height: Height,
    ) -> Self {
        let max_history_size = validator_history.len() as u64 - 1;

        assert_ne!(
            max_history_size, 0,
            "The chain must have a non-zero max_history_size"
        );

        assert_ne!(
            latest_height.revision_height(),
            0,
            "The chain must have a non-zero revision_height"
        );

        assert!(
            max_history_size <= latest_height.revision_height(),
            "The number of blocks must be greater than the number of validator set histories"
        );

        assert_eq!(
            host_id.revision_number(),
            latest_height.revision_number(),
            "The version in the chain identifier must match the version in the latest height"
        );

        let block_time = Duration::from_secs(DEFAULT_BLOCK_TIME_SECS);
        let next_block_timestamp = Timestamp::now().add(block_time).expect("Never fails");

        let history = (0..max_history_size)
            .rev()
            .map(|i| {
                // generate blocks with timestamps -> N, N - BT, N - 2BT, ...
                // where N = now(), BT = block_time
                HostBlock::generate_block_with_validators(
                    host_id.clone(),
                    host_type,
                    latest_height.sub(i).expect("Never fails").revision_height(),
                    next_block_timestamp
                        .sub(Duration::from_secs(DEFAULT_BLOCK_TIME_SECS * (i + 1)))
                        .expect("Never fails"),
                    &validator_history[(max_history_size - i) as usize - 1],
                    &validator_history[(max_history_size - i) as usize],
                )
            })
            .collect();

        MockContext {
            host_chain_type: host_type,
            host_chain_id: host_id,
            max_history_size,
            history,
            block_time,
            ibc_store: Arc::new(Mutex::new(MockIbcStore::default())),
        }
    }

    /// Associates a client record to this context.
    /// Given a client id and a height, registers a new client in the context and also associates
    /// to this client a mock client state and a mock consensus state for height `height`. The type
    /// of this client is implicitly assumed to be Mock.
    #[deprecated(
        since = "0.50.0",
        note = "Please use `MockClientConfig::builder().build()` instead"
    )]
    pub fn with_client(self, client_id: &ClientId, height: Height) -> Self {
        // NOTE: this is wrong; the client chain ID is supposed to represent
        // the chain ID of the counterparty chain. But at this point this is
        // too ingrained in our tests; `with_client()` is called everywhere,
        // which delegates to this.
        let client_chain_id = self.host_chain_id.clone();

        self.with_client_config(
            MockClientConfig::builder()
                .client_chain_id(client_chain_id)
                .client_id(client_id.clone())
                .latest_height(height)
                .build(),
        )
    }

    /// Similar to `with_client`, this function associates a client record to this context, but
    /// additionally permits to parametrize two details of the client. If `client_type` is None,
    /// then the client will have type Mock, otherwise the specified type. If
    /// `consensus_state_height` is None, then the client will be initialized with a consensus
    /// state matching the same height as the client state (`client_state_height`).
    #[deprecated(
        since = "0.50.0",
        note = "Please use `MockClientConfig::builder().build()` instead"
    )]
    pub fn with_client_parametrized(
        self,
        client_id: &ClientId,
        client_state_height: Height,
        client_type: Option<ClientType>,
        consensus_state_height: Option<Height>,
    ) -> Self {
        // NOTE: this is wrong; the client chain ID is supposed to represent
        // the chain ID of the counterparty chain. But at this point this is
        // too ingrained in our tests; `with_client()` is called everywhere,
        // which delegates to this.
        let client_chain_id = self.host_chain_id.clone();

        self.with_client_config(
            MockClientConfig::builder()
                .client_chain_id(client_chain_id)
                .client_id(client_id.clone())
                .latest_height(client_state_height)
                .client_type(client_type.unwrap_or_else(mock_client_type))
                .consensus_state_heights(
                    vec![consensus_state_height.unwrap_or(client_state_height)],
                )
                .build(),
        )
    }

    #[deprecated(
        since = "0.50.0",
        note = "Please use `MockClientConfig::builder().build()` instead"
    )]
    pub fn with_client_parametrized_with_chain_id(
        self,
        client_chain_id: ChainId,
        client_id: &ClientId,
        client_state_height: Height,
        client_type: Option<ClientType>,
        consensus_state_height: Option<Height>,
    ) -> Self {
        self.with_client_config(
            MockClientConfig::builder()
                .client_chain_id(client_chain_id)
                .client_id(client_id.clone())
                .latest_height(client_state_height)
                .client_type(client_type.unwrap_or_else(mock_client_type))
                .consensus_state_heights(
                    vec![consensus_state_height.unwrap_or(client_state_height)],
                )
                .build(),
        )
    }

    #[deprecated(
        since = "0.50.0",
        note = "Please use `MockClientConfig::builder().build()` instead"
    )]
    pub fn with_client_parametrized_history(
        self,
        client_id: &ClientId,
        client_state_height: Height,
        client_type: Option<ClientType>,
        consensus_state_height: Option<Height>,
    ) -> Self {
        let client_chain_id = self.host_chain_id.clone();
        let current_consensus_height = consensus_state_height.unwrap_or(client_state_height);
        let prev_consensus_height = current_consensus_height
            .sub(1)
            .unwrap_or(client_state_height);
        self.with_client_config(
            MockClientConfig::builder()
                .client_chain_id(client_chain_id)
                .client_id(client_id.clone())
                .latest_height(client_state_height)
                .client_type(client_type.unwrap_or_else(mock_client_type))
                .consensus_state_heights(vec![prev_consensus_height, current_consensus_height])
                .build(),
        )
    }

    #[deprecated(
        since = "0.50.0",
        note = "Please use `MockClientConfig::builder().build()` instead"
    )]
    pub fn with_client_parametrized_history_with_chain_id(
        self,
        client_chain_id: ChainId,
        client_id: &ClientId,
        client_state_height: Height,
        client_type: Option<ClientType>,
        consensus_state_height: Option<Height>,
    ) -> Self {
        let current_consensus_height = consensus_state_height.unwrap_or(client_state_height);
        let prev_consensus_height = current_consensus_height
            .sub(1)
            .unwrap_or(client_state_height);
        self.with_client_config(
            MockClientConfig::builder()
                .client_chain_id(client_chain_id)
                .client_id(client_id.clone())
                .latest_height(client_state_height)
                .client_type(client_type.unwrap_or_else(mock_client_type))
                .consensus_state_heights(vec![prev_consensus_height, current_consensus_height])
                .build(),
        )
    }

    pub fn with_client_config(self, client: MockClientConfig) -> Self {
        let cs_heights = if client.consensus_state_heights.is_empty() {
            vec![client.latest_height]
        } else {
            client.consensus_state_heights
        };

        fn blocks_since(a: Height, b: Height) -> Option<u64> {
            (a.revision_number() == b.revision_number()
                && a.revision_height() >= b.revision_height())
            .then(|| a.revision_height() - b.revision_height())
        }

        let (client_state, consensus_states) = match client.client_type.as_str() {
            MOCK_CLIENT_TYPE => {
                let blocks: Vec<_> = cs_heights
                    .into_iter()
                    .map(|cs_height| {
                        let n_blocks = blocks_since(client.latest_height, cs_height)
                            .expect("less or equal height");
                        (
                            cs_height,
                            MockHeader::new(cs_height).with_timestamp(
                                client
                                    .latest_timestamp
                                    .sub(self.block_time * (n_blocks as u32))
                                    .expect("never fails"),
                            ),
                        )
                    })
                    .collect();

                let client_state = MockClientState::new(
                    MockHeader::new(client.latest_height).with_timestamp(client.latest_timestamp),
                );

                let cs_states = blocks
                    .into_iter()
                    .map(|(height, block)| (height, MockConsensusState::new(block).into()))
                    .collect();

                (client_state.into(), cs_states)
            }
            TENDERMINT_CLIENT_TYPE => {
                let blocks: Vec<_> = cs_heights
                    .into_iter()
                    .map(|cs_height| {
                        let n_blocks = blocks_since(client.latest_height, cs_height)
                            .expect("less or equal height");
                        (
                            cs_height,
                            HostBlock::generate_tm_block(
                                client.client_chain_id.clone(),
                                cs_height.revision_height(),
                                client
                                    .latest_timestamp
                                    .sub(self.block_time * (n_blocks as u32))
                                    .expect("never fails"),
                            ),
                        )
                    })
                    .collect();

                let client_state: TmClientState = TmClientStateConfig::builder()
                    .chain_id(client.client_chain_id)
                    .latest_height(client.latest_height)
                    .trusting_period(client.trusting_period)
                    .max_clock_drift(client.max_clock_drift)
                    .unbonding_period(client.unbonding_period)
                    .build()
                    .try_into()
                    .expect("never fails");

                client_state.inner().validate().expect("never fails");

                let cs_states = blocks
                    .into_iter()
                    .map(|(height, block)| (height, block.into()))
                    .collect();

                (client_state.into(), cs_states)
            }
            _ => panic!("unknown client type"),
        };

        let client_record = MockClientRecord {
            client_state: Some(client_state),
            consensus_states,
        };

        self.ibc_store
            .lock()
            .clients
            .insert(client.client_id, client_record);
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

    /// Accessor for a block of the local (host) chain from this context.
    /// Returns `None` if the block at the requested height does not exist.
    pub fn host_block(&self, target_height: &Height) -> Option<&HostBlock> {
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
        let new_block = HostBlock::generate_block(
            self.host_chain_id.clone(),
            self.host_chain_type,
            latest_block.height().increment().revision_height(),
            latest_block
                .timestamp()
                .add(self.block_time)
                .expect("Never fails"),
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
        self.ibc_store.lock().clients[client_id]
            .client_state
            .as_ref()
            .expect("Never fails")
            .clone()
    }

    pub fn latest_consensus_states(
        &self,
        client_id: &ClientId,
        height: &Height,
    ) -> AnyConsensusState {
        self.ibc_store.lock().clients[client_id]
            .consensus_states
            .get(height)
            .expect("Never fails")
            .clone()
    }

    pub fn latest_height(&self) -> Height {
        self.history
            .last()
            .expect("history cannot be empty")
            .height()
    }

    pub fn ibc_store_share(&self) -> Arc<Mutex<MockIbcStore>> {
        self.ibc_store.clone()
    }

    pub fn query_latest_header(&self) -> Option<HostBlock> {
        let block_ref = self.host_block(&self.host_height().expect("Never fails"));
        block_ref.cloned()
    }

    pub fn get_events(&self) -> Vec<IbcEvent> {
        self.ibc_store.lock().events.clone()
    }

    pub fn get_logs(&self) -> Vec<String> {
        self.ibc_store.lock().logs.clone()
    }
}

#[cfg(test)]
mod tests {
    use ibc::core::channel::types::acknowledgement::Acknowledgement;
    use ibc::core::channel::types::channel::{Counterparty, Order};
    use ibc::core::channel::types::error::{ChannelError, PacketError};
    use ibc::core::channel::types::packet::Packet;
    use ibc::core::channel::types::Version;
    use ibc::core::primitives::Signer;
    use ibc::core::router::module::Module;
    use ibc::core::router::types::module::{ModuleExtras, ModuleId};

    use super::*;
    use crate::fixtures::core::channel::PacketConfig;
    use crate::fixtures::core::signer::dummy_bech32_account;
    use crate::testapp::ibc::core::router::MockRouter;

    #[test]
    fn test_history_manipulation() {
        pub struct Test {
            name: String,
            ctx: MockContext,
        }
        let cv = 1; // The version to use for all chains.

        let mock_chain_id = ChainId::new(&format!("mockgaia-{cv}")).unwrap();

        let tests: Vec<Test> = vec![
            Test {
                name: "Empty history, small pruning window".to_string(),
                ctx: MockContextConfig::builder()
                    .host_id(mock_chain_id.clone())
                    .max_history_size(2)
                    .latest_height(Height::new(cv, 1).expect("Never fails"))
                    .build(),
            },
            Test {
                name: "[Synthetic TM host] Empty history, small pruning window".to_string(),
                ctx: MockContextConfig::builder()
                    .host_id(mock_chain_id.clone())
                    .host_type(HostType::SyntheticTendermint)
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
                name: "[Synthetic TM host] Large pruning window".to_string(),
                ctx: MockContextConfig::builder()
                    .host_id(mock_chain_id.clone())
                    .host_type(HostType::SyntheticTendermint)
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
                name: "[Synthetic TM host] Small pruning window".to_string(),
                ctx: MockContextConfig::builder()
                    .host_id(mock_chain_id.clone())
                    .host_type(HostType::SyntheticTendermint)
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
                name: "[Synthetic TM host] Small pruning window, small starting height".to_string(),
                ctx: MockContextConfig::builder()
                    .host_id(mock_chain_id.clone())
                    .host_type(HostType::SyntheticTendermint)
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
            Test {
                name: "[Synthetic TM host] Large pruning window, large starting height".to_string(),
                ctx: MockContextConfig::builder()
                    .host_id(mock_chain_id)
                    .host_type(HostType::SyntheticTendermint)
                    .max_history_size(50)
                    .latest_height(Height::new(cv, 2000).expect("Never fails"))
                    .build(),
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
