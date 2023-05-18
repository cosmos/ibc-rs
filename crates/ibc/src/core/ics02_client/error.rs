//! Defines the client error type

use crate::prelude::*;

use displaydoc::Display;
use ibc_proto::protobuf::Error as TendermintProtoError;

use crate::core::ics02_client::client_type::ClientType;
use crate::core::ics23_commitment::error::CommitmentError;
use crate::core::ics24_host::identifier::{ClientId, IdentifierError};
use crate::core::timestamp::Timestamp;
use crate::core::ContextError;
use crate::Height;

/// Encodes all the possible client errors
#[derive(Debug, Display)]
pub enum ClientError {
    /// upgrade client error: `{0}`
    Upgrade(UpgradeClientError),
    /// Client identifier constructor failed for type `{client_type}` with counter `{counter}`, validation error: `{validation_error}`
    ClientIdentifierConstructor {
        client_type: ClientType,
        counter: u64,
        validation_error: IdentifierError,
    },
    /// client is frozen with description: `{description}`
    ClientFrozen { description: String },
    /// client state not found: `{client_id}`
    ClientStateNotFound { client_id: ClientId },
    /// client state already exists: `{client_id}`
    ClientStateAlreadyExists { client_id: ClientId },
    /// consensus state not found at: `{client_id}` at height `{height}`
    ConsensusStateNotFound { client_id: ClientId, height: Height },
    /// implementation specific error
    ImplementationSpecific,
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
    /// encode error: `{0}`
    Encode(prost::EncodeError),
    /// decode error: `{0}`
    Decode(prost::DecodeError),
    /// invalid client identifier error: `{0}`
    InvalidClientIdentifier(IdentifierError),
    /// invalid raw header error: `{0}`
    InvalidRawHeader(TendermintProtoError),
    /// missing raw header
    MissingRawHeader,
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
    InvalidPacketTimestamp(crate::core::timestamp::ParseTimestampError),
    /// mismatch between client and arguments types
    ClientArgsTypeMismatch { client_type: ClientType },
    /// received header height (`{header_height}`) is lower than (or equal to) client latest height (`{latest_height}`)
    LowHeaderHeight {
        header_height: Height,
        latest_height: Height,
    },
    /// timestamp is invalid or missing, timestamp=`{time1}`,  now=`{time2}`
    InvalidConsensusStateTimestamp { time1: Timestamp, time2: Timestamp },
    /// header not within trusting period: expires_at=`{latest_time}` now=`{update_time}`
    HeaderNotWithinTrustPeriod {
        latest_time: Timestamp,
        update_time: Timestamp,
    },
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
    /// other error: `{description}`
    Other { description: String },
}

impl From<ContextError> for ClientError {
    fn from(context_error: ContextError) -> Self {
        match context_error {
            ContextError::ClientError(e) => e,
            _ => ClientError::Other {
                description: context_error.to_string(),
            },
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for ClientError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self {
            Self::ClientIdentifierConstructor {
                validation_error: e,
                ..
            } => Some(e),
            Self::InvalidMsgUpdateClientId(e) => Some(e),
            Self::InvalidClientIdentifier(e) => Some(e),
            Self::InvalidRawHeader(e) => Some(e),
            Self::InvalidRawMisbehaviour(e) => Some(e),
            Self::InvalidCommitmentProof(e) => Some(e),
            Self::InvalidPacketTimestamp(e) => Some(e),
            Self::Ics23Verification(e) => Some(e),
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

impl std::error::Error for UpgradeClientError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self {
            Self::InvalidUpgradeClientProof(e) => Some(e),
            Self::InvalidUpgradeConsensusStateProof(e) => Some(e),
            _ => None,
        }
    }
}

impl From<UpgradeClientError> for ClientError {
    fn from(e: UpgradeClientError) -> Self {
        ClientError::Upgrade(e)
    }
}
