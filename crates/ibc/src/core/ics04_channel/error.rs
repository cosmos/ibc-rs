use super::packet::Sequence;
use super::timeout::TimeoutHeight;
use crate::core::ics02_client::error as client_error;
use crate::core::ics03_connection::error as connection_error;
use crate::core::ics04_channel::channel::State;
use crate::core::ics05_port::error as port_error;
use crate::core::ics24_host::error::ValidationError;
use crate::core::ics24_host::identifier::{ChannelId, ClientId, ConnectionId, PortId};
use crate::prelude::*;
use crate::signer::SignerError;
use crate::timestamp::Timestamp;
use crate::Height;

use displaydoc::Display;

#[derive(Debug, Display)]
pub enum ChannelError {
    /// connection error: `{0}`
    Connection(connection_error::ConnectionError),
    /// port error: `{0}`
    Port(port_error::PortError),
    /// channel state unknown: `{state}`
    UnknownState { state: i32 },
    /// channel order type unknown: `{type_id}`
    UnknownOrderType { type_id: String },
    /// invalid connection hops length: expected `{expected}`; actual `{actual}`
    InvalidConnectionHopsLength { expected: usize, actual: usize },
    /// invalid signer address error: `{0}`
    Signer(SignerError),
    /// invalid proof: missing height
    MissingHeight,
    /// packet data bytes must be valid UTF-8 (this restriction will be lifted in the future)
    NonUtf8PacketData,
    /// missing counterparty
    MissingCounterparty,
    /// no commong version
    NoCommonVersion,
    /// missing channel end
    MissingChannel,
    /// single version must be negociated on connection before opening channel
    InvalidVersionLengthConnection,
    /// the channel ordering is not supported by connection
    ChannelFeatureNotSuportedByConnection,
    /// the channel end (`{port_id}`, `{channel_id}`) does not exist
    ChannelNotFound {
        port_id: PortId,
        channel_id: ChannelId,
    },
    /// Verification fails for the packet with the sequence number `{sequence}`, error: `{client_error}`
    PacketVerificationFailed {
        sequence: Sequence,
        client_error: client_error::ClientError,
    },
    /// Error verifying channel state error: `{0}`
    VerifyChannelFailed(client_error::ClientError),
    /// String `{value}` cannot be converted to packet sequence, error: `{error}`
    InvalidStringAsSequence {
        value: String,
        error: core::num::ParseIntError,
    },
    /// Invalid channel id in counterparty
    InvalidCounterpartyChannelId,
    /// Processed time for the client `{client_id}` at height `{height}` not found
    ProcessedTimeNotFound { client_id: ClientId, height: Height },
    /// Processed height for the client `{client_id}` at height `{height}` not found
    ProcessedHeightNotFound { client_id: ClientId, height: Height },
    /// route not found
    RouteNotFound,
    /// application module error: `{description}`
    AppModule { description: String },
    /// other error: `{description}`
    Other { description: String },
    /// Channel `{channel_id}` is Closed
    ChannelClosed { channel_id: ChannelId },
    /// the associated connection `{connection_id}` is not OPEN
    ConnectionNotOpen { connection_id: ConnectionId },
    /// Undefined counterparty connection for `{connection_id}`
    UndefinedConnectionCounterparty { connection_id: ConnectionId },
    /// Client with id `{client_id}` is frozen
    FrozenClient { client_id: ClientId },
    /// Channel `{channel_id}` should not be state `{state}`
    InvalidChannelState { channel_id: ChannelId, state: State },
    /// invalid proof: empty proof
    InvalidProof,
    /// identifier error: `{0}`
    Identifier(ValidationError),
}

#[derive(Debug, Display)]
pub enum PacketError {
    /// connection error: `{0}`
    Connection(connection_error::ConnectionError),
    /// channel error: `{0}`
    Channel(ChannelError),
    /// Channel `{channel_id}` is Closed
    ChannelClosed { channel_id: ChannelId },
    /// packet destination port `{port_id}` and channel `{channel_id}` doesn't match the counterparty's port/channel
    InvalidPacketCounterparty {
        port_id: PortId,
        channel_id: ChannelId,
    },
    /// Client with id `{client_id}` is frozen
    FrozenClient { client_id: ClientId },
    /// Receiving chain block height `{chain_height}` >= packet timeout height `{timeout_height}`
    LowPacketHeight {
        chain_height: Height,
        timeout_height: TimeoutHeight,
    },
    /// Receiving chain block timestamp >= packet timeout timestamp
    LowPacketTimestamp,
    /// Invalid packet sequence `{given_sequence}` ≠ next send sequence `{next_sequence}`
    InvalidPacketSequence {
        given_sequence: Sequence,
        next_sequence: Sequence,
    },
    /// Channel `{channel_id}` should not be state `{state}`
    InvalidChannelState { channel_id: ChannelId, state: State },
    /// the associated connection `{connection_id}` is not OPEN
    ConnectionNotOpen { connection_id: ConnectionId },
    /// Receipt for the packet `{sequence}` not found
    PacketReceiptNotFound { sequence: Sequence },
    /// The stored commitment of the packet `{sequence}` is incorrect
    IncorrectPacketCommitment { sequence: Sequence },
    /// implementation specific error
    ImplementationSpecific,
    /// Undefined counterparty connection for `{connection_id}`
    UndefinedConnectionCounterparty { connection_id: ConnectionId },
    /// invalid proof: empty proof
    InvalidProof,
    /// Packet timeout height `{timeout_height}` > chain height `{chain_height}`
    PacketTimeoutHeightNotReached {
        timeout_height: TimeoutHeight,
        chain_height: Height,
    },
    /// Packet timeout timestamp `{timeout_timestamp}` > chain timestamp `{chain_timestamp}`
    PacketTimeoutTimestampNotReached {
        timeout_timestamp: Timestamp,
        chain_timestamp: Timestamp,
    },
    /// Packet acknowledgement exists for the packet with the sequence `{sequence}`
    AcknowledgementExists { sequence: Sequence },
    /// Acknowledgment cannot be empty
    InvalidAcknowledgement,
    /// Acknowledgment for the packet `{sequence}` not found
    PacketAcknowledgementNotFound { sequence: Sequence },
    /// invalid proof: missing height
    MissingHeight,
    /// there is no packet in this message
    MissingPacket,
    /// invalid signer address error: `{0}`
    Signer(SignerError),
    /// application module error: `{description}`
    AppModule { description: String },
    /// route not found
    RouteNotFound,
    /// packet sequence cannot be 0
    ZeroPacketSequence,
    /// invalid timeout height for the packet
    InvalidTimeoutHeight,
    /// packet data bytes cannot be empty
    ZeroPacketData,
    /// Invalid packet timeout timestamp value error: `{0}`
    InvalidPacketTimestamp(crate::timestamp::ParseTimestampError),
    /// identifier error: `{0}`
    Identifier(ValidationError),
    /// Missing sequence number for sending packets on port `{port_id}` and channel `{channel_id}`
    MissingNextSendSeq {
        port_id: PortId,
        channel_id: ChannelId,
    },
    /// the channel end (`{port_id}`, `{channel_id}`) does not exist
    ChannelNotFound {
        port_id: PortId,
        channel_id: ChannelId,
    },
    /// Commitment for the packet `{sequence}` not found
    PacketCommitmentNotFound { sequence: Sequence },
    /// Missing sequence number for receiving packets on port `{port_id}` and channel `{channel_id}`
    MissingNextRecvSeq {
        port_id: PortId,
        channel_id: ChannelId,
    },
    /// Missing sequence number for ack packets on port `{port_id}` and channel `{channel_id}`
    MissingNextAckSeq {
        port_id: PortId,
        channel_id: ChannelId,
    },
}

#[cfg(feature = "std")]
impl std::error::Error for PacketError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self {
            Self::Connection(e) => Some(e),
            Self::Channel(e) => Some(e),
            Self::Signer(e) => Some(e),
            Self::Identifier(e) => Some(e),
            _ => None,
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for ChannelError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self {
            Self::Connection(e) => Some(e),
            Self::Port(e) => Some(e),
            Self::Identifier(e) => Some(e),
            Self::Signer(e) => Some(e),
            Self::PacketVerificationFailed {
                client_error: e, ..
            } => Some(e),
            Self::InvalidStringAsSequence { error: e, .. } => Some(e),
            _ => None,
        }
    }
}
