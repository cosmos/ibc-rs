//! Defines the main channel, port, and packet error types

use displaydoc::Display;
use ibc_core_client_types::error::ClientError;
use ibc_core_client_types::Height;
use ibc_core_host_types::error::{DecodingError, IdentifierError};
use ibc_core_host_types::identifiers::{ChannelId, PortId, Sequence};
use ibc_primitives::prelude::*;
use ibc_primitives::{Timestamp, TimestampError};

use super::channel::Counterparty;
use super::timeout::TimeoutHeight;
use crate::commitment::PacketCommitment;
use crate::timeout::TimeoutTimestamp;
use crate::Version;

#[derive(Debug, Display)]
pub enum ChannelError {
    /// decoding error: `{0}`
    Decoding(DecodingError),
    /// identifier error: `{0}`
    InvalidIdentifier(IdentifierError),
    /// invalid channel id: expected `{expected}`, actual `{actual}`
    InvalidChannelId { expected: String, actual: String },
    /// invalid channel state: expected `{expected}`, actual `{actual}`
    InvalidState { expected: String, actual: String },
    /// invalid channel order type: expected `{expected}`, actual `{actual}`
    InvalidOrderType { expected: String, actual: String },
    /// invalid connection hops length: expected `{expected}`, actual `{actual}`
    InvalidConnectionHopsLength { expected: u64, actual: u64 },
    /// invalid counterparty: expected `{expected}`, actual `{actual}`
    InvalidCounterparty {
        expected: Counterparty,
        actual: Counterparty,
    },
    /// missing proof
    MissingProof,
    /// missing proof height
    MissingProofHeight,
    /// missing counterparty
    MissingCounterparty,
    /// unsupported channel upgrade sequence
    UnsupportedChannelUpgradeSequence,
    /// unsupported version: expected `{expected}`, actual `{actual}`
    UnsupportedVersion { expected: Version, actual: Version },
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
    /// failed proof verification: `{0}`
    FailedProofVerification(ClientError),

    // TODO(seanchen1991): These variants should be encoded by host-relevant error types
    // once those have been defined.
    /// application module error: `{description}`
    AppModule { description: String },
    /// missing channel counter
    MissingCounter,
    /// failed to update counter: `{description}`
    FailedToUpdateCounter { description: String },
    /// failed to store channel: `{description}`
    FailedToStoreChannel { description: String },
}

#[derive(Debug, Display)]
pub enum PacketError {
    /// channel error: `{0}`
    Channel(ChannelError),
    /// decoding error: `{0}`
    Decoding(DecodingError),
    /// insufficient packet timeout height: should have `{timeout_height}` > `{chain_height}`
    InsufficientPacketHeight {
        chain_height: Height,
        timeout_height: TimeoutHeight,
    },
    /// insufficient packet timestamp: should be greater than chain block timestamp
    InsufficientPacketTimestamp,
    /// mismatched packet sequences: expected `{expected}`, actual `{actual}`
    MismatchedPacketSequences {
        expected: Sequence,
        actual: Sequence,
    },
    /// mismatched commitments for packet `{sequence}`: expected `{expected:?}`, actual `{actual:?}`
    MismatchedPacketCommitments {
        sequence: Sequence,
        expected: PacketCommitment,
        actual: PacketCommitment,
    },
    /// missing timeout
    MissingTimeout,
    /// invalid timeout height: `{0}`
    InvalidTimeoutHeight(ClientError),
    /// invalid timeout timestamp: `{0}`
    InvalidTimeoutTimestamp(TimestampError),
    /// invalid identifier: `{0}`
    InvalidIdentifier(IdentifierError),
    /// empty acknowledgment status not allowed
    EmptyAcknowledgmentStatus,
    /// packet acknowledgment for sequence `{0}` already exists
    DuplicateAcknowledgment(Sequence),
    /// packet sequence cannot be 0
    ZeroPacketSequence,
    /// packet timeout height `{timeout_height}` > chain height `{chain_height} and timeout timestamp `{timeout_timestamp}` > chain timestamp `{chain_timestamp}`
    PacketTimeoutNotReached {
        timeout_height: TimeoutHeight,
        chain_height: Height,
        timeout_timestamp: TimeoutTimestamp,
        chain_timestamp: Timestamp,
    },
    /// implementation-specific error
    ImplementationSpecific,

    // TODO(seanchen1991): Move these variants to host-relevant error types
    /// application module error: `{description}`
    AppModule { description: String },
    /// missing acknowledgment for packet `{0}`
    MissingPacketAcknowledgment(Sequence),
    /// missing packet receipt for packet `{0}`
    MissingPacketReceipt(Sequence),
}

impl From<IdentifierError> for ChannelError {
    fn from(e: IdentifierError) -> Self {
        Self::InvalidIdentifier(e)
    }
}

impl From<IdentifierError> for PacketError {
    fn from(e: IdentifierError) -> Self {
        Self::InvalidIdentifier(e)
    }
}

impl From<DecodingError> for ChannelError {
    fn from(e: DecodingError) -> Self {
        Self::Decoding(e)
    }
}

impl From<DecodingError> for PacketError {
    fn from(e: DecodingError) -> Self {
        Self::Decoding(e)
    }
}

impl From<TimestampError> for PacketError {
    fn from(e: TimestampError) -> Self {
        Self::InvalidTimeoutTimestamp(e)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for PacketError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self {
            Self::Channel(e) => Some(e),
            Self::Decoding(e) => Some(e),
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
            Self::Decoding(e) => Some(e),
            Self::FailedPacketVerification {
                client_error: e, ..
            } => Some(e),
            _ => None,
        }
    }
}
