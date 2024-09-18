//! Defines the client error type

use displaydoc::Display;
use ibc_core_commitment_types::error::CommitmentError;
use ibc_core_host_types::error::{DecodingError, HostError, IdentifierError};
use ibc_core_host_types::identifiers::ClientId;
use ibc_primitives::prelude::*;
use ibc_primitives::Timestamp;

use crate::height::Height;
use crate::Status;

/// Encodes all the possible client errors
#[derive(Debug, Display)]
pub enum ClientError {
    /// host error : `{0}`
    Host(HostError),
    /// upgrade client error: `{0}`
    Upgrade(UpgradeClientError),
    /// decoding error: `{0}`
    Decoding(DecodingError),
    /// invalid trust threshold: `{numerator}`/`{denominator}`
    InvalidTrustThreshold { numerator: u64, denominator: u64 },
    /// invalid client state type: `{0}`
    InvalidClientStateType(String),
    /// invalid update client message
    InvalidUpdateClientMessage,
    /// invalid height; cannot be zero or negative
    InvalidHeight,
    /// invalid proof height; expected `{actual}` >= `{expected}`
    InvalidProofHeight { actual: Height, expected: Height },
    /// invalid consensus state timestamp: `{0}`
    InvalidConsensusStateTimestamp(Timestamp),
    /// invalid header type: `{0}`
    InvalidHeaderType(String),
    /// missing local consensus state at `{0}`
    MissingLocalConsensusState(Height),
    /// unexpected status found: `{0}`
    UnexpectedStatus(Status),
    /// client state already exists: `{0}`
    DuplicateClientState(ClientId),
    /// mismatched client recovery states
    MismatchedClientRecoveryStates,
    /// client recovery heights not allowed: expected substitute client height `{substitute_height}` > subject client height `{subject_height}`
    NotAllowedClientRecoveryHeights {
        subject_height: Height,
        substitute_height: Height,
    },
    /// failed ICS23 verification: `{0}`
    FailedICS23Verification(CommitmentError),
    /// failed header verification: `{description}`
    FailedHeaderVerification { description: String },
    /// failed misbehaviour handling: `{description}`
    FailedMisbehaviourHandling { description: String },

    // TODO(seanchen1991): Incorporate this error into its own variants
    /// client-specific error: `{description}`
    ClientSpecific { description: String },
}

impl From<CommitmentError> for ClientError {
    fn from(e: CommitmentError) -> Self {
        Self::FailedICS23Verification(e)
    }
}

impl From<DecodingError> for ClientError {
    fn from(e: DecodingError) -> Self {
        Self::Decoding(e)
    }
}

impl From<HostError> for ClientError {
    fn from(e: HostError) -> Self {
        Self::Host(e)
    }
}

impl From<IdentifierError> for ClientError {
    fn from(e: IdentifierError) -> Self {
        Self::Decoding(DecodingError::Identifier(e))
    }
}

#[cfg(feature = "std")]
impl std::error::Error for ClientError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self {
            Self::FailedICS23Verification(e) => Some(e),
            Self::Decoding(e) => Some(e),
            Self::Upgrade(e) => Some(e),
            Self::Host(e) => Some(e),
            _ => None,
        }
    }
}

/// Encodes all the possible upgrade client errors
#[derive(Debug, Display)]
pub enum UpgradeClientError {
    /// decoding error: `{0}`
    Decoding(DecodingError),
    /// host chain error: `{0}`
    Host(HostError),
    /// invalid upgrade proposal: `{description}`
    InvalidUpgradeProposal { description: String },
    /// invalid proof for the upgraded client state: `{0}`
    InvalidUpgradeClientStateProof(CommitmentError),
    /// invalid proof for the upgraded consensus state: `{0}`
    InvalidUpgradeConsensusStateProof(CommitmentError),
    /// invalid upgrade path: `{description}`
    InvalidUpgradePath { description: String },
    /// missing upgrade path
    MissingUpgradePath,
    /// insufficient upgrade client height `{upgraded_height}`; must be greater than current client height `{client_height}`
    InsufficientUpgradeHeight {
        upgraded_height: Height,
        client_height: Height,
    },
}

impl From<UpgradeClientError> for ClientError {
    fn from(e: UpgradeClientError) -> Self {
        ClientError::Upgrade(e)
    }
}

impl From<DecodingError> for UpgradeClientError {
    fn from(e: DecodingError) -> Self {
        Self::Decoding(e)
    }
}

impl From<HostError> for UpgradeClientError {
    fn from(e: HostError) -> Self {
        Self::Host(e)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for UpgradeClientError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self {
            Self::Decoding(e) => Some(e),
            Self::Host(e) => Some(e),
            Self::InvalidUpgradeClientStateProof(e)
            | Self::InvalidUpgradeConsensusStateProof(e) => Some(e),
            _ => None,
        }
    }
}
