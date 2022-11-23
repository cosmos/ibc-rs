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

#[cfg(feature = "std")]
impl std::error::Error for Error {}

#[derive(Debug, Display)]
pub enum Error {
    /// unknown client type: `{client_type}`
    UnknownClientType { client_type: String },
    /// Client identifier constructor failed for type `{client_type}` with counter `{counter}`, error(`{validation_error}`)
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
    /// invalid raw client identifier `{client_id}`, error(`{validation_error}`)
    InvalidRawClientId {
        client_id: String,
        validation_error: ValidationError,
    },
    /// error decoding raw client state, error(`{0}`)
    DecodeRawClientState(TendermintProtoError),
    /// missing raw client state
    MissingRawClientState,
    /// invalid raw client consensus state, error(`{0}`)
    InvalidRawConsensusState(TendermintProtoError),
    /// missing raw client consensus state
    MissingRawConsensusState,
    /// invalid client id in the update client message, error(`{0}`)
    InvalidMsgUpdateClientId(ValidationError),
    /// decode error(`{0}`)
    Decode(prost::DecodeError),
    /// invalid raw client consensus state: the height field is missing
    MissingHeight,
    /// invalid client identifier, error(`{0}`)
    InvalidClientIdentifier(ValidationError),
    /// invalid raw header, error(`{0}`)
    InvalidRawHeader(TendermintProtoError),
    /// missing raw header
    MissingRawHeader,
    /// invalid raw misbehaviour, error(`{0}`)
    DecodeRawMisbehaviour(TendermintProtoError),
    /// invalid raw misbehaviour, error(`{0}`)
    InvalidRawMisbehaviour(ValidationError),
    /// missing raw misbehaviour
    MissingRawMisbehaviour,
    /// String `{value}` cannnot be converted to height, error(`{height_error}`)
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
    /// invalid proof for the upgraded client state, error(`{0}`)
    InvalidUpgradeClientProof(Ics23Error),
    /// invalid proof for the upgraded consensus state, error(`{0}`)
    InvalidUpgradeConsensusStateProof(Ics23Error),
    /// invalid commitment proof bytes, error(`{0}`)
    InvalidCommitmentProof(Ics23Error),
    /// invalid packet timeout timestamp value, error(`{0}`)
    InvalidPacketTimestamp(crate::timestamp::ParseTimestampError),
    /// mismatch between client and arguments types, expected: `{client_type}`
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
    /// invalid connection end, error(`{0}`)
    InvalidConnectionEnd(TendermintProtoError),
    /// invalid channel end, error(`{0}`)
    InvalidChannelEnd(TendermintProtoError),
    /// invalid any client state, error(`{0}`)
    InvalidAnyClientState(TendermintProtoError),
    /// invalid any client consensus state, error(`{0}`)
    InvalidAnyConsensusState(TendermintProtoError),
    /// failed to parse signer, error(`{0}`)
    Signer(SignerError),
    /// ics23 verification failure, error(`{0}`)
    Ics23Verification(Ics23Error),
    /// client specific error: `{description}`
    ClientSpecific { description: String },
    /// other error: `{description}`
    Other { description: String },
}
