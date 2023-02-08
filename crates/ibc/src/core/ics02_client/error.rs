use crate::prelude::*;

use displaydoc::Display;
use ibc_proto::protobuf::Error as TendermintProtoError;

use crate::core::ics02_client::client_type::ClientType;
use crate::core::ics23_commitment::error::CommitmentError;
use crate::core::ics24_host::error::ValidationError;
use crate::core::ics24_host::identifier::ClientId;
use crate::signer::SignerError;
use crate::timestamp::Timestamp;
use crate::Height;

#[derive(Debug, Display)]
pub enum ClientError {
    /// Client identifier constructor failed for type `{client_type}` with counter `{counter}`, validation error: `{validation_error}`
    ClientIdentifierConstructor {
        client_type: ClientType,
        counter: u64,
        validation_error: ValidationError,
    },
    /// client not found: `{client_id}`
    ClientNotFound { client_id: ClientId },
    /// client is frozen: `{client_id}`
    ClientFrozen { client_id: ClientId },
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
    InvalidMsgUpdateClientId(ValidationError),
    /// Encode error: `{0}`
    Encode(TendermintProtoError),
    /// decode error: `{0}`
    Decode(prost::DecodeError),
    /// invalid client identifier error: `{0}`
    InvalidClientIdentifier(ValidationError),
    /// invalid raw header error: `{0}`
    InvalidRawHeader(TendermintProtoError),
    /// missing raw header
    MissingRawHeader,
    /// invalid raw misbehaviour error: `{0}`
    InvalidRawMisbehaviour(ValidationError),
    /// missing raw misbehaviour
    MissingRawMisbehaviour,
    /// revision height cannot be zero
    InvalidHeight,
    /// height cannot end up zero or negative
    InvalidHeightResult,
    /// invalid proof for the upgraded client state error: `{0}`
    InvalidUpgradeClientProof(CommitmentError),
    /// invalid proof for the upgraded consensus state error: `{0}`
    InvalidUpgradeConsensusStateProof(CommitmentError),
    /// invalid commitment proof bytes error: `{0}`
    InvalidCommitmentProof(CommitmentError),
    /// invalid packet timeout timestamp value error: `{0}`
    InvalidPacketTimestamp(crate::timestamp::ParseTimestampError),
    /// mismatch between client and arguments types
    ClientArgsTypeMismatch { client_type: ClientType },
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
    /// invalid connection end error: `{0}`
    InvalidConnectionEnd(TendermintProtoError),
    /// invalid channel end error: `{0}`
    InvalidChannelEnd(TendermintProtoError),
    /// invalid any client consensus state error: `{0}`
    InvalidAnyConsensusState(TendermintProtoError),
    /// failed to parse signer error: `{0}`
    Signer(SignerError),
    /// ics23 verification failure error: `{0}`
    Ics23Verification(CommitmentError),
    /// misbehaviour handling failed with reason: `{reason}`
    MisbehaviourHandlingFailure { reason: String },
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
            Self::InvalidMsgUpdateClientId(e) => Some(e),
            Self::InvalidClientIdentifier(e) => Some(e),
            Self::InvalidRawHeader(e) => Some(e),
            Self::InvalidRawMisbehaviour(e) => Some(e),
            Self::InvalidUpgradeClientProof(e) => Some(e),
            Self::InvalidUpgradeConsensusStateProof(e) => Some(e),
            Self::InvalidCommitmentProof(e) => Some(e),
            Self::InvalidPacketTimestamp(e) => Some(e),
            Self::InvalidConnectionEnd(e) => Some(e),
            Self::InvalidChannelEnd(e) => Some(e),
            Self::InvalidAnyConsensusState(e) => Some(e),
            Self::Signer(e) => Some(e),
            Self::Ics23Verification(e) => Some(e),
            _ => None,
        }
    }
}
