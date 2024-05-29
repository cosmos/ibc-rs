//! Required traits for blanket implementations of [`gRPC query services`](crate::core).

use ibc::core::channel::types::channel::IdentifiedChannelEnd;
use ibc::core::channel::types::packet::PacketState;
use ibc::core::client::types::Height;
use ibc::core::connection::types::IdentifiedConnectionEnd;
use ibc::core::handler::types::error::ContextError;
use ibc::core::host::types::identifiers::{ClientId, ConnectionId, Sequence};
use ibc::core::host::types::path::{ChannelEndPath, Path};
use ibc::core::host::{ClientStateRef, ConsensusStateRef, ValidationContext};
use ibc::core::primitives::prelude::*;

/// Context to be implemented by the host to provide proofs in query responses
pub trait ProvableContext {
    /// Returns the proof for the given path at the given height.
    /// As this is in the context of IBC, the path is expected to be an [`IbcPath`](Path).
    fn get_proof(&self, height: Height, path: &Path) -> Option<Vec<u8>>;
}

/// Context to be implemented by the host that provides gRPC query services.
pub trait QueryContext: ProvableContext + ValidationContext {
    // Client queries

    /// Returns the list of all clients.
    fn client_states(&self) -> Result<Vec<(ClientId, ClientStateRef<Self>)>, ContextError>;

    /// Returns the list of all consensus states for the given client.
    fn consensus_states(
        &self,
        client_id: &ClientId,
    ) -> Result<Vec<(Height, ConsensusStateRef<Self>)>, ContextError>;

    /// Returns the list of all heights at which consensus states for the given client are.
    fn consensus_state_heights(&self, client_id: &ClientId) -> Result<Vec<Height>, ContextError>;

    // Connection queries

    /// Returns the list of all connection ends.
    fn connection_ends(&self) -> Result<Vec<IdentifiedConnectionEnd>, ContextError>;

    /// Returns the list of all connection ids of the given client.
    fn client_connection_ends(
        &self,
        client_id: &ClientId,
    ) -> Result<Vec<ConnectionId>, ContextError>;

    // Channel queries

    /// Returns the list of all channel ends.
    fn channel_ends(&self) -> Result<Vec<IdentifiedChannelEnd>, ContextError>;

    // Packet queries

    /// Returns the list of all packet commitments for the given channel end.
    fn packet_commitments(
        &self,
        channel_end_path: &ChannelEndPath,
    ) -> Result<Vec<PacketState>, ContextError>;

    /// Filters the list of packet sequences for the given channel end that are acknowledged.
    /// Returns all the packet acknowledgements if `sequences` is empty.
    fn packet_acknowledgements(
        &self,
        channel_end_path: &ChannelEndPath,
        sequences: impl ExactSizeIterator<Item = Sequence>,
    ) -> Result<Vec<PacketState>, ContextError>;

    /// Filters the packet sequences for the given channel end that are not received.
    fn unreceived_packets(
        &self,
        channel_end_path: &ChannelEndPath,
        sequences: impl ExactSizeIterator<Item = Sequence>,
    ) -> Result<Vec<Sequence>, ContextError>;

    /// Filters the list of packet sequences for the given channel end whose acknowledgement is not received.
    /// Returns all the unreceived acknowledgements if `sequences` is empty.
    fn unreceived_acks(
        &self,
        channel_end_path: &ChannelEndPath,
        sequences: impl ExactSizeIterator<Item = Sequence>,
    ) -> Result<Vec<Sequence>, ContextError>;
}
