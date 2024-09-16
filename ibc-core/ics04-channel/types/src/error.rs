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

/// Errors that arise from the ICS04 Channel module
#[derive(Debug, Display)]
pub enum ChannelError {
    /// decoding error: `{0}`
    Decoding(DecodingError),
    /// packet acknowledgment for sequence `{0}` already exists
    DuplicateAcknowledgment(Sequence),
    /// empty acknowledgment status not allowed
    EmptyAcknowledgmentStatus,
    /// failed verification: `{0}`
    FailedVerification(ClientError),
    /// insufficient packet timeout height: should have `{timeout_height}` > `{chain_height}`
    InsufficientPacketHeight {
        chain_height: Height,
        timeout_height: TimeoutHeight,
    },
    /// insufficient packet timestamp: should be greater than chain block timestamp
    InsufficientPacketTimestamp,
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
    /// invalid timeout height: `{0}`
    InvalidTimeoutHeight(ClientError),
    /// invalid timeout timestamp: `{0}`
    InvalidTimeoutTimestamp(TimestampError),
    /// missing proof
    MissingProof,
    /// missing proof height
    MissingProofHeight,
    /// missing counterparty
    MissingCounterparty,
    /// missing timeout
    MissingTimeout,
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
    /// non-existent channel end: (`{port_id}`, `{channel_id}`)
    NonexistentChannel {
        port_id: PortId,
        channel_id: ChannelId,
    },
    /// packet timeout height `{timeout_height}` > chain height `{chain_height} and timeout timestamp `{timeout_timestamp}` > chain timestamp `{chain_timestamp}`
    PacketTimeoutNotReached {
        timeout_height: TimeoutHeight,
        chain_height: Height,
        timeout_timestamp: TimeoutTimestamp,
        chain_timestamp: Timestamp,
    },
    /// unsupported channel upgrade sequence
    UnsupportedChannelUpgradeSequence,
    /// unsupported version: expected `{expected}`, actual `{actual}`
    UnsupportedVersion { expected: Version, actual: Version },
    /// packet sequence cannot be 0
    ZeroPacketSequence,
}

impl From<IdentifierError> for ChannelError {
    fn from(e: IdentifierError) -> Self {
        Self::Decoding(DecodingError::Identifier(e))
    }
}

impl From<DecodingError> for ChannelError {
    fn from(e: DecodingError) -> Self {
        Self::Decoding(e)
    }
}

impl From<TimestampError> for ChannelError {
    fn from(e: TimestampError) -> Self {
        Self::InvalidTimeoutTimestamp(e)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for ChannelError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self {
            Self::Decoding(e) => Some(e),
            Self::FailedVerification(e) => Some(e),
            _ => None,
        }
    }
}
