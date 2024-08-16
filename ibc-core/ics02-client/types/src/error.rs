//! Defines the client error type

use core::convert::Infallible;

use displaydoc::Display;
use ibc_core_commitment_types::error::CommitmentError;
use ibc_core_host_types::error::IdentifierError;
use ibc_core_host_types::identifiers::ClientId;
use ibc_primitives::prelude::*;
use ibc_primitives::Timestamp;

use super::status::Status;
use crate::height::Height;

/// Encodes all the possible client errors
#[derive(Debug, Display)]
pub enum ClientError {
    /// upgrade client error: `{0}`
    Upgrade(UpgradeClientError),
    /// invalid client status: `{actual}`
    InvalidStatus { actual: Status },
    /// missing client state : `{client_id}`
    MissingClientState { client_id: ClientId },
    /// client state already exists: `{client_id}`
    AlreadyExistingClientState { client_id: ClientId },
    /// client recovery heights not allowed: expected substitute client height `{substitute_height}` > subject client height `{subject_height}`
    NotAllowedClientRecoveryHeights {
        subject_height: Height,
        substitute_height: Height,
    },
    /// mismatched client recovery states
    MismatchedClientRecoveryStates,
    /// missing consensus state at `{client_id}`/`{height}`
    MissingConsensusState { client_id: ClientId, height: Height },
    /// missing update client metadata at `{client_id}`/`{height}`
    MissingUpdateMetaData { client_id: ClientId, height: Height },
    /// failed header verification: `{description}`
    FailedHeaderVerification { description: String },
    /// invalid trust threshold: `{numerator}`/`{denominator}`
    InvalidTrustThreshold { numerator: u64, denominator: u64 },
    /// invalid client state type: `{actual}`
    InvalidClientStateType { actual: String },
    /// invalid client consensus state type: `{actual}`
    InvalidConsensusStateType { actual: String },
    /// invalid header type: `{actual}`
    InvalidHeaderType { actual: String },
    /// invalid misbehaviour type: `{actual}`
    InvalidMisbehaviourType { actual: String },
    /// missing raw client state
    MissingRawClientState,
    /// missing raw client consensus state
    MissingRawConsensusState,
    /// invalid update client message
    InvalidUpdateClientMessage,
    /// invalid client identifier: `{0}`
    InvalidClientIdentifier(IdentifierError),
    /// invalid raw header: `{description}`
    InvalidRawHeader { description: String },
    /// missing raw client message
    MissingRawClientMessage,
    /// missing raw misbehaviour
    MissingRawMisbehaviour,
    /// invalid height; cannot be zero or negative
    InvalidHeight,
    /// invalid proof height; expected `{actual}` >= `{expected}`
    InvalidProofHeight { actual: Height, expected: Height },
    /// invalid consensus state timestamp: `{actual}`
    InvalidConsensusStateTimestamp { actual: Timestamp },
    /// missing local consensus state at `{height}`
    MissingLocalConsensusState { height: Height },
    /// failed ics23 verification: `{0}`
    FailedIcs23Verification(CommitmentError),
    /// failed misbehaviour handling: `{description}`
    FailedMisbehaviourHandling { description: String },
    /// invalid attribute key: `{actual}`
    InvalidAttributeKey { actual: String },
    /// invalid attribute value: `{actual}`
    InvalidAttributeValue { actual: String },
    /// missing attribute key
    MissingAttributeKey,
    /// missing attribute value
    MissingAttributeValue,

    // TODO(seanchen1991): Can we remove these two variants?
    /// client-specific error: `{description}`
    ClientSpecific { description: String },
    /// other error: `{description}`
    Other { description: String },
}

impl From<&'static str> for ClientError {
    fn from(s: &'static str) -> Self {
        Self::Other {
            description: s.to_string(),
        }
    }
}

impl From<Infallible> for ClientError {
    fn from(value: Infallible) -> Self {
        match value {}
    }
}

#[cfg(feature = "std")]
impl std::error::Error for ClientError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self {
            Self::InvalidClientIdentifier(e) => Some(e),
            Self::FailedIcs23Verification(e) => Some(e),
            _ => None,
        }
    }
}

/// Encodes all the possible upgrade client errors
#[derive(Debug, Display)]
pub enum UpgradeClientError {
    /// invalid proof for the upgraded client state error: `{0}`
    InvalidUpgradeClientProof(CommitmentError),
    /// invalid proof for the upgraded consensus state error: `{0}`
    InvalidUpgradeConsensusStateProof(CommitmentError),
    /// upgraded client height `{upgraded_height}` must be at greater than current client height `{client_height}`
    LowUpgradeHeight {
        upgraded_height: Height,
        client_height: Height,
    },
    /// Invalid upgrade path: `{reason}`
    InvalidUpgradePath { reason: String },
    /// invalid upgrade proposal: `{reason}`
    InvalidUpgradeProposal { reason: String },
    /// invalid upgrade plan: `{reason}`
    InvalidUpgradePlan { reason: String },
    /// other upgrade client error: `{reason}`
    Other { reason: String },
}

impl From<UpgradeClientError> for ClientError {
    fn from(e: UpgradeClientError) -> Self {
        ClientError::Upgrade(e)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for UpgradeClientError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self {
            Self::InvalidUpgradeClientProof(e) | Self::InvalidUpgradeConsensusStateProof(e) => {
                Some(e)
            }
            _ => None,
        }
    }
}
