//! Defines the main channel, port, and packet error types

use displaydoc::Display;

use ibc_core_client_types::error::ClientError;
use ibc_core_client_types::Height;
use ibc_core_connection_types::error as connection_error;
use ibc_core_host_types::error::IdentifierError;
use ibc_core_host_types::identifiers::{ChannelId, ConnectionId, PortId, Sequence};
use ibc_primitives::prelude::*;
use ibc_primitives::{Timestamp, TimestampError};

use super::channel::Counterparty;
use super::timeout::TimeoutHeight;

use crate::channel::State;
use crate::timeout::TimeoutTimestamp;
use crate::Version;

#[derive(Debug, Display)]
pub enum ChannelError {
    /// invalid channel id: expected `{expected}`, actual `{actual}`
    InvalidChannelId { expected: String, actual: String },
    /// invalid channel state: expected `{expected}`, actual `{actual}`
    InvalidState { expected: String, actual: String },
    /// invalid channel order type: expected `{expected}`, actual `{actual}`
    InvalidOrderType { expected: String, actual: String },
    /// invalid connection hops length: expected `{expected}`, actual `{actual}`
    InvalidConnectionHopsLength { expected: u64, actual: u64 },
    /// missing proof
    MissingProof,
    /// missing proof height
    MissingProofHeight,
    /// packet data bytes must be valid UTF-8
    NonUtf8PacketData,
    /// missing counterparty
    MissingCounterparty,
    /// unsupported channel upgrade sequence
    UnsupportedChannelUpgradeSequence,
    /// unsupported version: expected `{expected}`, actual `{actual}`
    UnsupportedVersion { expected: Version, actual: Version },
    /// missing channel end
    MissingChannelEnd,
    /// non-existent channel end: (`{port_id}`, `{channel_id}`)
    NonexistentChannel {
        port_id: PortId,
        channel_id: ChannelId,
    },
    /// failed packet verification for packet with sequence `{sequence}`: `{client_error}`
    FailedPacketVerification {
        sequence: Sequence,
        client_error: ClientError,
    },
    /// failed channel verification: `{0}`
    FailedChannelVerification(ClientError),
    /// failed to parse `{actual}` as packet sequence: `{error}`
    FailedToParseSequence {
        actual: String,
        error: core::num::ParseIntError,
    },
    /// invalid counterparty: expected `{expected}`, actual `{actual}`
    InvalidCounterparty {
        expected: Counterparty,
        actual: Counterparty,
    },
    /// application module error: `{description}`
    AppModule { description: String },
    /// undefined counterparty for connection: `{connection_id}`
    UndefinedConnectionCounterparty { connection_id: ConnectionId },
    /// identifier error: `{0}`
    InvalidIdentifier(IdentifierError),
    /// missing channel counter
    MissingCounter,
    /// failed to update counter: `{description}`
    FailedToUpdateCounter { description: String },
    /// failed to store channel: `{description}`
    FailedToStoreChannel { description: String },
}

#[derive(Debug, Display)]
pub enum PacketError {
    /// connection error: `{0}`
    Connection(connection_error::ConnectionError),
    /// channel error: `{0}`
    Channel(ChannelError),
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
    /// implementation-specific error
    ImplementationSpecific,
    /// Undefined counterparty connection for `{connection_id}`
    UndefinedConnectionCounterparty { connection_id: ConnectionId },
    /// invalid proof: empty proof
    InvalidProof,
    /// Packet timeout height `{timeout_height}` > chain height `{chain_height} and timeout timestamp `{timeout_timestamp}` > chain timestamp `{chain_timestamp}`
    PacketTimeoutNotReached {
        timeout_height: TimeoutHeight,
        chain_height: Height,
        timeout_timestamp: TimeoutTimestamp,
        chain_timestamp: Timestamp,
    },
    /// Packet acknowledgement exists for the packet with the sequence `{sequence}`
    AcknowledgementExists { sequence: Sequence },
    /// Acknowledgment cannot be empty
    InvalidAcknowledgement,
    /// Acknowledgment status cannot be empty
    EmptyAcknowledgementStatus,
    /// Acknowledgment for the packet `{sequence}` not found
    PacketAcknowledgementNotFound { sequence: Sequence },
    /// invalid proof: missing height
    MissingHeight,
    /// there is no packet in this message
    MissingPacket,
    /// invalid signer error: `{reason}`
    InvalidSigner { reason: String },
    /// application module error: `{description}`
    AppModule { description: String },
    /// route not found
    RouteNotFound,
    /// packet sequence cannot be 0
    ZeroPacketSequence,
    /// packet data bytes cannot be empty
    ZeroPacketData,
    /// invalid timeout height with error: `{0}`
    InvalidTimeoutHeight(ClientError),
    /// Invalid timeout timestamp with error: `{0}`
    InvalidTimeoutTimestamp(TimestampError),
    /// missing timeout
    MissingTimeout,
    /// invalid identifier error: `{0}`
    InvalidIdentifier(IdentifierError),
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
    /// other error: `{description}`
    Other { description: String },
}

impl From<IdentifierError> for ChannelError {
    fn from(err: IdentifierError) -> Self {
        Self::InvalidIdentifier(err)
    }
}

impl From<IdentifierError> for PacketError {
    fn from(err: IdentifierError) -> Self {
        Self::InvalidIdentifier(err)
    }
}

impl From<TimestampError> for PacketError {
    fn from(err: TimestampError) -> Self {
        Self::InvalidTimeoutTimestamp(err)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for PacketError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self {
            Self::Connection(e) => Some(e),
            Self::Channel(e) => Some(e),
            Self::InvalidIdentifier(e) => Some(e),
            _ => None,
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for ChannelError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self {
            Self::InvalidIdentifier(e) => Some(e),
            Self::FailedPacketVerification {
                client_error: e, ..
            } => Some(e),
            Self::FailedToParseSequence { error: e, .. } => Some(e),
            _ => None,
        }
    }
}
