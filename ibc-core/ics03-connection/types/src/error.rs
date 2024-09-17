//! Defines the connection error type

use displaydoc::Display;
use ibc_core_client_types::error::ClientError;
use ibc_core_client_types::Height;
use ibc_core_host_types::error::{DecodingError, HostError, IdentifierError};
use ibc_primitives::prelude::*;
use ibc_primitives::{Timestamp, TimestampError};

#[derive(Debug, Display)]
pub enum ConnectionError {
    /// decoding error: `{0}`
    Decoding(DecodingError),
    /// host error: `{0}`
    Host(HostError),
    /// invalid counterparty
    InvalidCounterparty,
    /// invalid client state: `{description}`
    InvalidClientState { description: String },
    /// mismatched connection states: expected `{expected}`, actual `{actual}`
    MismatchedConnectionStates { expected: String, actual: String },
    /// missing supported features
    MissingFeatures,
    /// missing common version
    MissingCommonVersion,
    /// missing counterparty
    MissingCounterparty,
    /// insufficient consensus height `{current_height}` for host chain; needs to meet counterparty's height `{target_height}`
    InsufficientConsensusHeight {
        target_height: Height,
        current_height: Height,
    },
    /// insufficient blocks elapsed: current height `{current_host_height}` needs to meet `{earliest_valid_height}`
    InsufficientBlocksElapsed {
        current_host_height: Height,
        earliest_valid_height: Height,
    },
    /// insufficient time elapsed: current timestamp `{current_host_time}` needs to meet `{earliest_valid_time}`
    InsufficientTimeElapsed {
        current_host_time: Timestamp,
        earliest_valid_time: Timestamp,
    },
    /// failed to verify client: `{0}`
    FailedToVerifyClient(ClientError),
    /// overflowed timestamp: `{0}`
    OverflowedTimestamp(TimestampError),
}

impl From<DecodingError> for ConnectionError {
    fn from(e: DecodingError) -> Self {
        Self::Decoding(e)
    }
}

impl From<IdentifierError> for ConnectionError {
    fn from(e: IdentifierError) -> Self {
        Self::Decoding(DecodingError::Identifier(e))
    }
}

impl From<HostError> for ConnectionError {
    fn from(e: HostError) -> Self {
        Self::Host(e)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for ConnectionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self {
            Self::Decoding(e) => Some(e),
            Self::Host(e) => Some(e),
            Self::FailedToVerifyClient(e) => Some(e),
            _ => None,
        }
    }
}
