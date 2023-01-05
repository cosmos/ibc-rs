use crate::prelude::*;

use crate::core::ics02_client::error::ClientError;
use crate::core::ics24_host::identifier::{ChainId, ClientId};
use crate::timestamp::{Timestamp, TimestampOverflowError};
use crate::Height;

use core::time::Duration;

use displaydoc::Display;
use tendermint::account::Id;
use tendermint::{Error as TendermintError, Hash};
use tendermint_light_client_verifier::errors::VerificationErrorDetail as LightClientErrorDetail;
use tendermint_light_client_verifier::operations::VotingPowerTally;
use tendermint_light_client_verifier::types::Validator;
use tendermint_light_client_verifier::Verdict;

#[derive(Debug, Display)]
pub enum Error {
    /// chain-id is (`{chain_id}`) is too long, got: `{len}`, max allowed: `{max_len}`
    ChainIdTooLong {
        chain_id: ChainId,
        len: usize,
        max_len: usize,
    },
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
    /// negative max clock drift
    NegativeMaxClockDrift,
    /// missing latest height
    MissingLatestHeight,
    /// invalid raw header error: `{0}`
    InvalidRawHeader(TendermintError),
    /// invalid raw misbehaviour: `{reason}`
    InvalidRawMisbehaviour { reason: String },
    /// decode error: `{0}`
    Decode(prost::DecodeError),
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
    /// the header's current/trusted revision number (`{current_revision}`) and the update's revision number (`{update_revision}`) should be the same
    MismatchedRevisions {
        current_revision: u64,
        update_revision: u64,
    },
    /// not enough trust because insufficient validators overlap: `{reason}`
    NotEnoughTrustedValsSigned { reason: VotingPowerTally },
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
    /// trusted validators `{trusted_validator_set:?}`, does not hash to latest trusted validators. Expected: `{next_validators_hash}`, got: `{trusted_val_hash}`
    MisbehaviourTrustedValidatorHashMismatch {
        trusted_validator_set: Vec<Validator>,
        next_validators_hash: Hash,
        trusted_val_hash: Hash,
    },
    /// current timestamp minus the latest consensus state timestamp is greater than or equal to the trusting period (`{duration_since_consensus_state:?}` >= `{trusting_period:?}`)
    ConsensusStateTimestampGteTrustingPeriod {
        duration_since_consensus_state: Duration,
        trusting_period: Duration,
    },
    /// headers block hashes are equal
    MisbehaviourHeadersBlockHashesEqual,
    /// headers are not at same height and are monotonically increasing
    MisbehaviourHeadersNotAtSameHeight,
    /// header chain-id `{header_chain_id}` does not match the light client's chain-id `{chain_id}`)
    MisbehaviourHeadersChainIdMismatch {
        header_chain_id: String,
        chain_id: String,
    },
    /// invalid raw client id: `{client_id}`
    InvalidRawClientId { client_id: String },
}

#[cfg(feature = "std")]
impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self {
            Self::InvalidHeader { error: e, .. } => Some(e),
            Self::InvalidTendermintTrustThreshold(e) => Some(e),
            Self::InvalidRawHeader(e) => Some(e),
            Self::Decode(e) => Some(e),
            Self::TimestampOverflow(e) => Some(e),
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

pub(crate) trait IntoResult<T, E> {
    fn into_result(self) -> Result<T, E>;
}

impl IntoResult<(), Error> for Verdict {
    fn into_result(self) -> Result<(), Error> {
        match self {
            Verdict::Success => Ok(()),
            Verdict::NotEnoughTrust(reason) => Err(Error::NotEnoughTrustedValsSigned { reason }),
            Verdict::Invalid(detail) => Err(Error::VerificationError { detail }),
        }
    }
}
