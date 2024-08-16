//! Defines the main channel, port, and packet error types

use displaydoc::Display;

use ibc_core_client_types::error::ClientError;
use ibc_core_client_types::Height;
use ibc_core_host_types::error::IdentifierError;
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
    /// channel error: `{0}`
    Channel(ChannelError),
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
    /// missing packet receipt for packet `{sequence}`
    MissingPacketReceipt { sequence: Sequence },
    /// missing proof
    MissingProof,
    /// missing acknowledgment for packet `{sequence}`
    MissingPacketAcknowledgment { sequence: Sequence },
    /// missing proof height
    MissingProofHeight,
    /// missing timeout
    MissingTimeout,
    /// invalid timeout height: `{0}`
    InvalidTimeoutHeight(ClientError),
    /// invalid timeout timestamp: `{0}`
    InvalidTimeoutTimestamp(TimestampError),
    /// invalid identifier: `{0}`
    InvalidIdentifier(IdentifierError),
    /// packet timeout height `{timeout_height}` > chain height `{chain_height} and timeout timestamp `{timeout_timestamp}` > chain timestamp `{chain_timestamp}`
    PacketTimeoutNotReached {
        timeout_height: TimeoutHeight,
        chain_height: Height,
        timeout_timestamp: TimeoutTimestamp,
        chain_timestamp: Timestamp,
    },
    /// packet acknowledgment for sequence `{sequence}` already exists
    DuplicateAcknowledgment { sequence: Sequence },
    /// emtpy acknowledgment not allowed
    EmptyAcknowledgment,
    /// empty acknowledgment status not allowed
    EmptyAcknowledgmentStatus,
    /// packet data bytes cannot be empty
    EmptyPacketData,
    /// packet sequence cannot be 0
    ZeroPacketSequence,

    /// implementation-specific error
    ImplementationSpecific,
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
