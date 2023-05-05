//! Defines the connection error type

use crate::core::ics02_client::error as client_error;
use crate::core::ics03_connection::version::Version;
use crate::core::ics24_host::identifier::{ClientId, ConnectionId, IdentifierError};
use crate::core::timestamp::{Timestamp, TimestampOverflowError};
use crate::Height;

use alloc::string::String;
use displaydoc::Display;
use ibc_proto::protobuf::Error as ProtoError;

#[derive(Debug, Display)]
pub enum ConnectionError {
    /// client error: `{0}`
    Client(client_error::ClientError),
    /// invalid connection state: expected `{expected}`, actual `{actual}`
    InvalidState { expected: String, actual: String },
    /// invalid connection end error: `{0}`
    InvalidConnectionEnd(ProtoError),
    /// consensus height claimed by the client on the other party is too advanced: `{target_height}` (host chain current height: `{current_height}`)
    InvalidConsensusHeight {
        target_height: Height,
        current_height: Height,
    },
    /// identifier error: `{0}`
    InvalidIdentifier(IdentifierError),
    /// ConnectionEnd domain object could not be constructed out of empty proto object
    EmptyProtoConnectionEnd,
    /// empty supported versions
    EmptyVersions,
    /// single version must be negotiated on connection before opening channel
    InvalidVersionLength,
    /// version \"`{version}`\" not supported
    VersionNotSupported { version: Version },
    /// no common version
    NoCommonVersion,
    /// empty supported features
    EmptyFeatures,
    /// feature \"`{feature}`\" not supported
    FeatureNotSupported { feature: String },
    /// no common features
    NoCommonFeatures,
    /// missing proof height
    MissingProofHeight,
    /// missing consensus height
    MissingConsensusHeight,
    /// invalid connection proof error
    InvalidProof,
    /// verifying connection state error: `{0}`
    VerifyConnectionState(client_error::ClientError),
    /// invalid signer error: `{reason}`
    InvalidSigner { reason: String },
    /// no connection was found for the previous connection id provided `{connection_id}`
    ConnectionNotFound { connection_id: ConnectionId },
    /// invalid counterparty
    InvalidCounterparty,
    /// missing counterparty
    MissingCounterparty,
    /// missing client state
    MissingClientState,
    /// the consensus proof verification failed (height: `{height}`), client error: `{client_error}`
    ConsensusStateVerificationFailure {
        height: Height,
        client_error: client_error::ClientError,
    },
    /// the client state proof verification failed for client id `{client_id}`, client error: `{client_error}`
    ClientStateVerificationFailure {
        // TODO: use more specific error source
        client_id: ClientId,
        client_error: client_error::ClientError,
    },
    /// invalid client state: `{reason}`
    InvalidClientState { reason: String },
    /// not enough blocks elapsed, current height `{current_host_height}` is still less than earliest acceptable height `{earliest_valid_height}`
    NotEnoughBlocksElapsed {
        current_host_height: Height,
        earliest_valid_height: Height,
    },
    /// not enough time elapsed, current timestamp `{current_host_time}` is still less than earliest acceptable timestamp `{earliest_valid_time}`
    NotEnoughTimeElapsed {
        current_host_time: Timestamp,
        earliest_valid_time: Timestamp,
    },
    /// timestamp overflowed error: `{0}`
    TimestampOverflow(TimestampOverflowError),
    /// other error: `{description}`
    Other { description: String },
}

#[cfg(feature = "std")]
impl std::error::Error for ConnectionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self {
            Self::Client(e) => Some(e),
            Self::InvalidIdentifier(e) => Some(e),
            Self::VerifyConnectionState(e) => Some(e),
            Self::ConsensusStateVerificationFailure {
                client_error: e, ..
            } => Some(e),
            Self::ClientStateVerificationFailure {
                client_error: e, ..
            } => Some(e),
            Self::TimestampOverflow(e) => Some(e),
            _ => None,
        }
    }
}
