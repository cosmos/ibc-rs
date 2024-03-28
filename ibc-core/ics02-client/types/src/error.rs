//! Defines the client error type

use displaydoc::Display;
use ibc_core_commitment_types::error::CommitmentError;
use ibc_core_host_types::error::IdentifierError;
use ibc_core_host_types::identifiers::{ClientId, ClientType};
use ibc_primitives::prelude::*;
use ibc_primitives::Timestamp;

use super::status::Status;
use crate::height::Height;

/// Encodes all the possible client errors
#[derive(Debug, Display)]
pub enum ClientError {
    /// upgrade client error: `{0}`
    Upgrade(UpgradeClientError),
    /// client is frozen with description: `{description}`
    ClientFrozen { description: String },
    /// client is not active. Status=`{status}`
    ClientNotActive { status: Status },
    /// client is not frozen or expired. Status=`{status}`
    ClientNotInactive { status: Status },
    /// client state not found: `{client_id}`
    ClientStateNotFound { client_id: ClientId },
    /// client state already exists: `{client_id}`
    ClientStateAlreadyExists { client_id: ClientId },
    /// Substitute client height `{substitute_height}` is not greater than subject client height `{subject_height}` during client recovery
    ClientRecoveryHeightMismatch {
        subject_height: Height,
        substitute_height: Height,
    },
    /// Subject and substitute client state mismatch during client recovery
    ClientRecoveryStateMismatch,
    /// consensus state not found at: `{client_id}` at height `{height}`
    ConsensusStateNotFound { client_id: ClientId, height: Height },
    /// Processed time or height for the client `{client_id}` at height `{height}` not found
    UpdateMetaDataNotFound { client_id: ClientId, height: Height },
    /// header verification failed with reason: `{reason}`
    HeaderVerificationFailure { reason: String },
    /// failed to build trust threshold from fraction: `{numerator}`/`{denominator}`
    InvalidTrustThreshold { numerator: u64, denominator: u64 },
    /// failed to build Tendermint domain type trust threshold from fraction: `{numerator}`/`{denominator}`
    FailedTrustThresholdConversion { numerator: u64, denominator: u64 },
    /// unknown client state type: `{client_state_type}`
    UnknownClientStateType { client_state_type: String },
    /// empty prefix
    EmptyPrefix,
    /// unknown client consensus state type: `{consensus_state_type}`
    UnknownConsensusStateType { consensus_state_type: String },
    /// unknown header type: `{header_type}`
    UnknownHeaderType { header_type: String },
    /// unknown misbehaviour type: `{misbehaviour_type}`
    UnknownMisbehaviourType { misbehaviour_type: String },
    /// missing raw client state
    MissingRawClientState,
    /// missing raw client consensus state
    MissingRawConsensusState,
    /// invalid client id in the update client message: `{0}`
    InvalidMsgUpdateClientId(IdentifierError),
    /// invalid client id in recover client message: `{0}`
    InvalidMsgRecoverClientId(IdentifierError),
    /// invalid client identifier error: `{0}`
    InvalidClientIdentifier(IdentifierError),
    /// invalid raw header error: `{reason}`
    InvalidRawHeader { reason: String },
    /// missing raw client message
    MissingClientMessage,
    /// invalid raw misbehaviour error: `{0}`
    InvalidRawMisbehaviour(IdentifierError),
    /// missing raw misbehaviour
    MissingRawMisbehaviour,
    /// revision height cannot be zero
    InvalidHeight,
    /// height cannot end up zero or negative
    InvalidHeightResult,
    /// the proof height is insufficient: latest_height=`{latest_height}` proof_height=`{proof_height}`
    InvalidProofHeight {
        latest_height: Height,
        proof_height: Height,
    },
    /// invalid commitment proof bytes error: `{0}`
    InvalidCommitmentProof(CommitmentError),
    /// invalid packet timeout timestamp value error: `{0}`
    InvalidPacketTimestamp(ibc_primitives::ParseTimestampError),
    /// mismatch between client and arguments types
    ClientArgsTypeMismatch { client_type: ClientType },
    /// timestamp is invalid or missing, timestamp=`{time1}`,  now=`{time2}`
    InvalidConsensusStateTimestamp { time1: Timestamp, time2: Timestamp },
    /// the local consensus state could not be retrieved for height `{height}`
    MissingLocalConsensusState { height: Height },
    /// invalid signer error: `{reason}`
    InvalidSigner { reason: String },
    /// ics23 verification failure error: `{0}`
    Ics23Verification(CommitmentError),
    /// misbehaviour handling failed with reason: `{reason}`
    MisbehaviourHandlingFailure { reason: String },
    /// client specific error: `{description}`
    ClientSpecific { description: String },
    /// client counter overflow error
    CounterOverflow,
    /// update client message did not contain valid header or misbehaviour
    InvalidUpdateClientMessage,
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

#[cfg(feature = "std")]
impl std::error::Error for ClientError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self {
            Self::InvalidMsgUpdateClientId(e)
            | Self::InvalidClientIdentifier(e)
            | Self::InvalidRawMisbehaviour(e) => Some(e),
            Self::InvalidCommitmentProof(e) | Self::Ics23Verification(e) => Some(e),
            Self::InvalidPacketTimestamp(e) => Some(e),
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
