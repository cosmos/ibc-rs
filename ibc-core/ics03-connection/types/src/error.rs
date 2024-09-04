//! Defines the connection error type

use displaydoc::Display;
use ibc_core_client_types::error::ClientError;
use ibc_core_client_types::Height;
use ibc_core_host_types::error::{DecodingError, IdentifierError};
use ibc_core_host_types::identifiers::ConnectionId;
use ibc_primitives::prelude::*;
use ibc_primitives::{Timestamp, TimestampError};

use crate::version::Version;

#[derive(Debug, Display)]
pub enum ConnectionError {
    /// decoding error: `{0}`
    Decoding(DecodingError),
    /// identifier error: `{0}`
    Identifier(IdentifierError),
    /// invalid state for initializing new ConnectionEnd; expected `Init` connection state and a single version
    InvalidStateForConnectionEndInit,
    /// invalid connection proof
    InvalidProof,
    /// invalid counterparty
    InvalidCounterparty,
    /// invalid client state: `{description}`
    InvalidClientState { description: String },
    /// mismatched connection states: expected `{expected}`, actual `{actual}`
    MismatchedConnectionStates { expected: String, actual: String },
    /// empty proto connection end; failed to construct ConnectionEnd domain object
    EmptyProtoConnectionEnd,
    /// empty supported versions
    EmptyVersions,
    /// empty supported features
    EmptyFeatures,
    /// unsupported version \"`{0}`\"
    UnsupportedVersion(Version),
    /// unsupported feature \"`{0}`\"
    UnsupportedFeature(String),
    /// missing common version
    MissingCommonVersion,
    /// missing common features
    MissingCommonFeatures,
    /// missing proof height
    MissingProofHeight,
    /// missing consensus height
    MissingConsensusHeight,
    /// missing counterparty
    MissingCounterparty,
    /// missing client state
    MissingClientState,
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
    /// failed to verify connection state: `{0}`
    FailedToVerifyConnectionState(ClientError),
    /// failed to verify consensus state: `{0}`
    FailedToVerifyConsensusState(ClientError),
    /// failed to verify client state: `{0}`
    FailedToVerifyClientState(ClientError),
    /// overflowed timestamp: `{0}`
    OverflowedTimestamp(TimestampError),

    // TODO(seanchen1991): Move these variants to host-relevant error types
    /// missing connection `{0}`
    MissingConnection(ConnectionId),
    /// missing connection counter
    MissingConnectionCounter,
    /// failed to store connection IDs
    FailedToStoreConnectionIds,
    /// failed to store connection end
    FailedToStoreConnectionEnd,
    /// failed to update connection counter
    FailedToUpdateConnectionCounter,
}

impl From<DecodingError> for ConnectionError {
    fn from(e: DecodingError) -> Self {
        Self::Decoding(e)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for ConnectionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self {
            Self::FailedToVerifyConnectionState(e)
            | Self::FailedToVerifyConsensusState(e)
            | Self::FailedToVerifyClientState(e) => Some(e),
            // Self::InvalidIdentifier(e) => Some(e),
            // Self::OverflowedTimestamp(e) => Some(e),
            _ => None,
        }
    }
}
