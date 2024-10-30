//! Defines the main channel, port, and packet error types

use displaydoc::Display;
use ibc_eureka_core_client_types::error::ClientError;
use ibc_eureka_core_client_types::Height;
use ibc_eureka_core_host_types::error::{DecodingError, HostError, IdentifierError};
use ibc_eureka_core_host_types::identifiers::{ClientId, Sequence};
use ibc_primitives::prelude::*;
use ibc_primitives::{Timestamp, TimestampError};

use super::timeout::TimeoutHeight;
use crate::commitment::PacketCommitment;
use crate::timeout::TimeoutTimestamp;
use crate::Version;

/// Errors that arise from the ICS04 Channel module
#[derive(Debug, Display, derive_more::From)]
pub enum ChannelError {
    /// decoding error: {0}
    Decoding(DecodingError),
    /// host error: {0}
    Host(HostError),
    /// client error: {0}
    Client(ClientError),
    /// timestamp error: {0}
    Timestamp(TimestampError),
    /// packet acknowledgment for sequence `{0}` already exists
    DuplicateAcknowledgment(Sequence),
    /// insufficient packet timeout height: should have `{timeout_height}` > `{chain_height}`
    InsufficientPacketHeight {
        chain_height: Height,
        timeout_height: TimeoutHeight,
    },
    /// expired packet timestamp: should be greater than chain block timestamp
    ExpiredPacketTimestamp,
    /// packet timeout height `{timeout_height}` > chain height `{chain_height} and timeout timestamp `{timeout_timestamp}` > chain timestamp `{chain_timestamp}`
    InsufficientPacketTimeout {
        timeout_height: TimeoutHeight,
        chain_height: Height,
        timeout_timestamp: TimeoutTimestamp,
        chain_timestamp: Timestamp,
    },
    /// invalid channel state: expected `{expected}`, actual `{actual}`
    InvalidState { expected: String, actual: String },
    /// invalid connection hops length: expected `{expected}`, actual `{actual}`
    InvalidConnectionHopsLength { expected: u64, actual: u64 },
    /// missing acknowledgment status
    MissingAcknowledgmentStatus,
    /// missing counterparty
    MissingCounterparty,
    /// missing timeout
    MissingTimeout,
    /// mismatched counterparty: expected `{expected}`, actual `{actual}`
    MismatchCounterparty {
        expected: ClientId,
        actual: ClientId,
    },
    /// mismatched packet sequence: expected `{expected}`, actual `{actual}`
    MismatchedPacketSequence {
        expected: Sequence,
        actual: Sequence,
    },
    /// mismatched packet commitments: expected `{expected:?}`, actual `{actual:?}`
    MismatchedPacketCommitment {
        expected: PacketCommitment,
        actual: PacketCommitment,
    },
    /// unsupported version: expected `{expected}`, actual `{actual}`
    UnsupportedVersion { expected: Version, actual: Version },
    /// application specific error: `{description}`
    AppSpecific { description: String },
}

impl From<IdentifierError> for ChannelError {
    fn from(e: IdentifierError) -> Self {
        Self::Decoding(DecodingError::Identifier(e))
    }
}

#[cfg(feature = "std")]
impl std::error::Error for ChannelError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self {
            Self::Decoding(e) => Some(e),
            Self::Client(e) => Some(e),
            Self::Host(e) => Some(e),
            Self::Timestamp(e) => Some(e),
            _ => None,
        }
    }
}
