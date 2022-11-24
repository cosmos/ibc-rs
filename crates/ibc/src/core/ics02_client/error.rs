use crate::prelude::*;
use displaydoc::Display;
use ibc_proto::protobuf::Error as TendermintProtoError;

use crate::core::ics02_client::client_type::ClientType;
use crate::core::ics02_client::height::HeightError;
use crate::core::ics23_commitment::error::Error as Ics23Error;
use crate::core::ics24_host::error::ValidationError;
use crate::core::ics24_host::identifier::ClientId;
use crate::signer::SignerError;
use crate::timestamp::Timestamp;
use crate::Height;

#[derive(Debug, Display)]
pub enum ClientError {
    /// unknown client type: `{client_type}`
    UnknownClientType { client_type: String },
    /// Client identifier constructor failed for type `{client_type}` with counter `{counter}`
    ClientIdentifierConstructor {
        client_type: ClientType,
        counter: u64,
        validation_error: ValidationError,
    },
    /// client already exists: `{client_id}`
    ClientAlreadyExists { client_id: ClientId },
    /// client not found: `{client_id}`
    ClientNotFound { client_id: ClientId },
    /// client is frozen: `{client_id}`
    ClientFrozen { client_id: ClientId },
    /// consensus state not found at: `{client_id}` at height `{height}`
    ConsensusStateNotFound { client_id: ClientId, height: Height },
    /// implementation specific error
    ImplementationSpecific,
    /// header verification failed with reason: `{reaseon}`
    HeaderVerificationFailure { reaseon: String },
    /// failed to build trust threshold from fraction: `{numerator}`/`{denominator}`
    InvalidTrustThreshold { numerator: u64, denominator: u64 },
    /// failed to build Tendermint domain type trust threshold from fraction: `{numerator}`/`{denominator}`
    FailedTrustThresholdConversion { numerator: u64, denominator: u64 },
    /// unknown client state type: `{client_state_type}`
    UnknownClientStateType { client_state_type: String },
    /// the client state was not found
    EmptyClientStateResponse,
    /// empty prefix
    EmptyPrefix,
    /// unknown client consensus state type: `{consensus_state_type}`
    UnknownConsensusStateType { consensus_state_type: String },
    /// the client consensus state was not found
    EmptyConsensusStateResponse,
    /// unknown header type: `{header_type}`
    UnknownHeaderType { header_type: String },
    /// unknown misbehaviour type: `{misbehavior_type}`
    UnknownMisbehaviourType { misbehavior_type: String },
    /// invalid raw client identifier `{client_id}`
    InvalidRawClientId {
        client_id: String,
        validation_error: ValidationError,
    },
    /// error decoding raw client state
    DecodeRawClientState(TendermintProtoError),
    /// missing raw client state
    MissingRawClientState,
    /// invalid raw client consensus state
    InvalidRawConsensusState(TendermintProtoError),
    /// missing raw client consensus state
    MissingRawConsensusState,
    /// invalid client id in the update client message
    InvalidMsgUpdateClientId(ValidationError),
    /// decode error
    Decode(prost::DecodeError),
    /// invalid raw client consensus state: the height field is missing
    MissingHeight,
    /// invalid client identifier
    InvalidClientIdentifier(ValidationError),
    /// invalid raw header
    InvalidRawHeader(TendermintProtoError),
    /// missing raw header
    MissingRawHeader,
    /// invalid raw misbehaviour
    DecodeRawMisbehaviour(TendermintProtoError),
    /// invalid raw misbehaviour
    InvalidRawMisbehaviour(ValidationError),
    /// missing raw misbehaviour
    MissingRawMisbehaviour,
    /// String `{value}` cannnot be converted to height
    InvalidStringAsHeight {
        value: String,
        height_error: HeightError,
    },
    /// revision height cannot be zero
    InvalidHeight,
    /// height cannot end up zero or negative
    InvalidHeightResult,
    /// invalid address
    InvalidAddress,
    /// invalid proof for the upgraded client state
    InvalidUpgradeClientProof(Ics23Error),
    /// invalid proof for the upgraded consensus state
    InvalidUpgradeConsensusStateProof(Ics23Error),
    /// invalid commitment proof bytes
    InvalidCommitmentProof(Ics23Error),
    /// invalid packet timeout timestamp value
    InvalidPacketTimestamp(crate::timestamp::ParseTimestampError),
    /// mismatch between client and arguments types
    ClientArgsTypeMismatch { client_type: ClientType },
    /// Insufficient overlap `{reason}`
    InsufficientVotingPower { reason: String },
    /// mismatch in raw client consensus state `{state_type}` with expected state `{consensus_type}`
    RawClientAndConsensusStateTypesMismatch {
        state_type: ClientType,
        consensus_type: ClientType,
    },
    /// received header height (`{header_height}`) is lower than (or equal to) client latest height (`{latest_height}`)
    LowHeaderHeight {
        header_height: Height,
        latest_height: Height,
    },
    /// upgraded client height `{upgraded_height}` must be at greater than current client height `{client_height}`
    LowUpgradeHeight {
        upgraded_height: Height,
        client_height: Height,
    },
    /// timestamp is invalid or missing, timestamp=`{time1}`,  now=`{time2}`
    InvalidConsensusStateTimestamp { time1: Timestamp, time2: Timestamp },
    /// header not withing trusting period: expires_at=`{latest_time}` now=`{update_time}`
    HeaderNotWithinTrustPeriod {
        latest_time: Timestamp,
        update_time: Timestamp,
    },
    /// the local consensus state could not be retrieved for height `{height}`
    MissingLocalConsensusState { height: Height },
    /// invalid connection end
    InvalidConnectionEnd(TendermintProtoError),
    /// invalid channel end
    InvalidChannelEnd(TendermintProtoError),
    /// invalid any client state
    InvalidAnyClientState(TendermintProtoError),
    /// invalid any client consensus state
    InvalidAnyConsensusState(TendermintProtoError),
    /// failed to parse signer
    Signer(SignerError),
    /// ics23 verification failure
    Ics23Verification(Ics23Error),
    /// client specific error: `{description}`
    ClientSpecific { description: String },
    /// other error: `{description}`
    Other { description: String },
}

#[cfg(feature = "std")]
impl std::error::Error for ClientError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self {
            Self::ClientIdentifierConstructor {
                validation_error: e,
                ..
            } => Some(e),
            Self::InvalidRawClientId {
                validation_error: e,
                ..
            } => Some(e),
            Self::DecodeRawClientState(e) => Some(e),
            Self::InvalidRawConsensusState(e) => Some(e),
            Self::InvalidMsgUpdateClientId(e) => Some(e),
            Self::Decode(e) => Some(e),
            Self::InvalidClientIdentifier(e) => Some(e),
            Self::InvalidRawHeader(e) => Some(e),
            Self::DecodeRawMisbehaviour(e) => Some(e),
            Self::InvalidRawMisbehaviour(e) => Some(e),
            Self::InvalidStringAsHeight {
                height_error: e, ..
            } => Some(e),
            Self::InvalidUpgradeClientProof(e) => Some(e),
            Self::InvalidUpgradeConsensusStateProof(e) => Some(e),
            Self::InvalidCommitmentProof(e) => Some(e),
            Self::InvalidPacketTimestamp(e) => Some(e),
            Self::InvalidConnectionEnd(e) => Some(e),
            Self::InvalidChannelEnd(e) => Some(e),
            Self::InvalidAnyClientState(e) => Some(e),
            Self::InvalidAnyConsensusState(e) => Some(e),
            Self::Signer(e) => Some(e),
            Self::Ics23Verification(e) => Some(e),
            _ => None,
        }
    }
}
