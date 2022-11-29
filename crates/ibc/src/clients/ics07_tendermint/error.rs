use crate::prelude::*;

use crate::core::ics02_client::error::ClientError;
use crate::core::ics24_host::error::ValidationError;
use crate::core::ics24_host::identifier::{ChainId, ClientId};
use crate::timestamp::{Timestamp, TimestampOverflowError};
use displaydoc::Display;

use crate::Height;
use tendermint::account::Id;
use tendermint::hash::Hash;
use tendermint::Error as TendermintError;
use tendermint_light_client_verifier::errors::VerificationErrorDetail as LightClientErrorDetail;

#[derive(Debug, Display)]
pub enum Error {
    /// chain-id is (`{chain_id}`) is too long, got: `{len}`, max allowed: `{max_len}`
    ChainIdTooLong {
        chain_id: ChainId,
        len: usize,
        max_len: usize,
    },
    /// invalid trusting period: `{reason}`
    InvalidTrustingPeriod { reason: String },
    /// invalid unbonding period: `{reason}`
    InvalidUnbondingPeriod { reason: String },
    /// invalid address
    InvalidAddress,
    /// invalid header, failed basic validation: `{reason}`, error: `{error}`
    InvalidHeader {
        reason: String,
        error: TendermintError,
    },
    /// invalid client state trust threshold: `{reason}`
    InvalidTrustThreshold { reason: String },
    /// invalid tendermint client state trust threshold error: `{0}`
    InvalidTendermintTrustThreshold(TendermintError),
    /// invalid client state max clock drift: `{reason}`
    InvalidMaxClockDrift { reason: String },
    /// invalid client state latest height: `{reason}`
    InvalidLatestHeight { reason: String },
    /// missing signed header
    MissingSignedHeader,
    /// invalid header, failed basic validation: `{reason}`
    Validation { reason: String },
    /// invalid raw client state: `{reason}`
    InvalidRawClientState { reason: String },
    /// missing validator set
    MissingValidatorSet,
    /// missing trusted validator set
    MissingTrustedValidatorSet,
    /// missing trusted height
    MissingTrustedHeight,
    /// missing trusting period
    MissingTrustingPeriod,
    /// missing unbonding period
    MissingUnbondingPeriod,
    /// invalid chain identifier error: `{0}`
    InvalidChainIdentifier(ValidationError),
    /// negative trusting period
    NegativeTrustingPeriod,
    /// negative unbonding period
    NegativeUnbondingPeriod,
    /// missing max clock drift
    MissingMaxClockDrift,
    /// negative max clock drift
    NegativeMaxClockDrift,
    /// missing latest height
    MissingLatestHeight,
    /// invalid frozen height
    InvalidFrozenHeight,
    /// invalid chain identifier: `{raw_value}`, validation error: `{error}`
    InvalidChainId {
        raw_value: String,
        error: ValidationError,
    },
    /// invalid raw height: `{raw_height}`
    InvalidRawHeight { raw_height: u64 },
    /// invalid raw client consensus state: `{reason}`
    InvalidRawConsensusState { reason: String },
    /// invalid raw header error: `{0}`
    InvalidRawHeader(TendermintError),
    /// invalid raw misbehaviour: `{reason}`
    InvalidRawMisbehaviour { reason: String },
    /// decode error: `{0}`
    Decode(prost::DecodeError),
    /// insufficient overlap: `{reason}`
    InsufficientVotingPower { reason: String },
    /// header timestamp `{low}` must be greater than current client consensus state timestamp `{high}`
    LowUpdateTimestamp { low: String, high: String },
    /// header timestamp `{low}` is outside the trusting period w.r.t. consensus state timestamp `{high}`
    HeaderTimestampOutsideTrustingTime { low: String, high: String },
    /// given other previous updates, header timestamp should be at most `{max}`, but was `{actual}`
    HeaderTimestampTooHigh { actual: String, max: String },
    /// given other previous updates, header timestamp should be at least `{min}`, but was `{actual}`
    HeaderTimestampTooLow { actual: String, min: String },
    /// timestamp overflowed error: `{0}`
    TimestampOverflow(TimestampOverflowError),
    /// not enough time elapsed, current timestamp `{current_time}` is still less than earliest acceptable timestamp `{earliest_time}`
    NotEnoughTimeElapsed {
        current_time: Timestamp,
        earliest_time: Timestamp,
    },
    /// not enough blocks elapsed, current height `{current_height}` is still less than earliest acceptable height `{earliest_height}`
    NotEnoughBlocksElapsed {
        current_height: Height,
        earliest_height: Height,
    },
    /// header revision height = `{height}` is invalid
    InvalidHeaderHeight { height: u64 },
    /// header height is `{height_header}` and is lower than the trusted header height, which is `{trusted_header_height}`
    InvalidTrustedHeaderHeight {
        trusted_header_height: Height,
        height_header: Height,
    },
    /// header height is `{low}` but it must be greater than the current client height which is `{high}`
    LowUpdateHeight { low: Height, high: Height },
    /// the header's current/trusted revision number (`{current_revision}`) and the update's revision number (`{update_revision}`) should be the same
    MismatchedRevisions {
        current_revision: u64,
        update_revision: u64,
    },
    /// invalid validator set: header_validators_hash=`{hash1}` and validators_hash=`{hash2}`
    InvalidValidatorSet { hash1: Hash, hash2: Hash },
    /// not enough trust because insufficient validators overlap: `{reason}`
    NotEnoughTrustedValsSigned { reason: String },
    /// verification failed: `{detail}`
    VerificationError { detail: LightClientErrorDetail },
    /// Processed time for the client `{client_id}` at height `{height}` not found
    ProcessedTimeNotFound { client_id: ClientId, height: Height },
    /// Processed height for the client `{client_id}` at height `{height}` not found
    ProcessedHeightNotFound { client_id: ClientId, height: Height },
    /// the height is insufficient: latest_height=`{latest_height}` target_height=`{target_height}`
    InsufficientHeight {
        latest_height: Height,
        target_height: Height,
    },
    /// the client is frozen: frozen_height=`{frozen_height}` target_height=`{target_height}`
    ClientFrozen {
        frozen_height: Height,
        target_height: Height,
    },
}

#[cfg(feature = "std")]
impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self {
            Error::InvalidHeader { error: e, .. } => Some(e),
            Error::InvalidTendermintTrustThreshold(e) => Some(e),
            Error::InvalidChainIdentifier(e) => Some(e),
            Error::InvalidChainId { error: e, .. } => Some(e),
            Error::InvalidRawHeader(e) => Some(e),
            Error::Decode(e) => Some(e),
            Error::TimestampOverflow(e) => Some(e),
            _ => None,
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for VerificationError {}

#[derive(Debug, Display)]
pub enum VerificationError {
    /// couldn't verify validator signature
    InvalidSignature,
    /// duplicate validator in commit signatures with address `{id}`
    DuplicateValidator { id: Id },
    /// insufficient signers overlap between `{q1}` and `{q2}`
    InsufficientOverlap { q1: u64, q2: u64 },
}

impl From<Error> for ClientError {
    fn from(e: Error) -> Self {
        Self::ClientSpecific {
            description: e.to_string(),
        }
    }
}
