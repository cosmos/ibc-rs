//! Defines the Tendermint light client's error type

use core::time::Duration;

use displaydoc::Display;
use ibc_core_client_types::error::ClientError;
use ibc_core_client_types::Height;
use ibc_core_commitment_types::error::CommitmentError;
use ibc_core_host_types::error::IdentifierError;
use ibc_core_host_types::identifiers::ClientId;
use ibc_primitives::prelude::*;
use tendermint::{Error as TendermintError, Hash};
use tendermint_light_client_verifier::errors::VerificationErrorDetail as LightClientErrorDetail;
use tendermint_light_client_verifier::operations::VotingPowerTally;
use tendermint_light_client_verifier::Verdict;

/// The main error type
#[derive(Debug, Display)]
pub enum Error {
    /// invalid identifier: `{0}`
    InvalidIdentifier(IdentifierError),
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
    /// invalid client proof specs: `{0}`
    InvalidProofSpec(CommitmentError),
    /// invalid raw client state: `{reason}`
    InvalidRawClientState { reason: String },
    /// missing validator set
    MissingValidatorSet,
    /// missing trusted next validator set
    MissingTrustedNextValidatorSet,
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
    /// given other previous updates, header timestamp should be at most `{max}`, but was `{actual}`
    HeaderTimestampTooHigh { actual: String, max: String },
    /// given other previous updates, header timestamp should be at least `{min}`, but was `{actual}`
    HeaderTimestampTooLow { actual: String, min: String },
    /// header revision height = `{height}` is invalid
    InvalidHeaderHeight { height: u64 },
    /// frozen height is missing
    MissingFrozenHeight,
    /// the header's trusted revision number (`{trusted_revision}`) and the update's revision number (`{header_revision}`) should be the same
    MismatchHeightRevisions {
        trusted_revision: u64,
        header_revision: u64,
    },
    /// the given chain-id (`{given}`) does not match the chain-id of the client (`{expected}`)
    MismatchHeaderChainId { given: String, expected: String },
    /// not enough trust because insufficient validators overlap: `{reason}`
    NotEnoughTrustedValsSigned { reason: VotingPowerTally },
    /// verification failed: `{detail}`
    VerificationError { detail: Box<LightClientErrorDetail> },
    /// Processed time or height for the client `{client_id}` at height `{height}` not found
    UpdateMetaDataNotFound { client_id: ClientId, height: Height },
    /// The given hash of the validators does not matches the given hash in the signed header. Expected: `{signed_header_validators_hash}`, got: `{validators_hash}`
    MismatchValidatorsHashes {
        validators_hash: Hash,
        signed_header_validators_hash: Hash,
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
}

#[cfg(feature = "std")]
impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self {
            Self::InvalidIdentifier(e) => Some(e),
            Self::InvalidHeader { error: e, .. }
            | Self::InvalidTendermintTrustThreshold(e)
            | Self::InvalidRawHeader(e) => Some(e),
            _ => None,
        }
    }
}

impl From<Error> for ClientError {
    fn from(e: Error) -> Self {
        Self::ClientSpecific {
            description: e.to_string(),
        }
    }
}

impl From<IdentifierError> for Error {
    fn from(e: IdentifierError) -> Self {
        Self::InvalidIdentifier(e)
    }
}

impl From<CommitmentError> for Error {
    fn from(e: CommitmentError) -> Self {
        Self::InvalidProofSpec(e)
    }
}

pub trait IntoResult<T, E> {
    fn into_result(self) -> Result<T, E>;
}

impl IntoResult<(), Error> for Verdict {
    fn into_result(self) -> Result<(), Error> {
        match self {
            Verdict::Success => Ok(()),
            Verdict::NotEnoughTrust(reason) => Err(Error::NotEnoughTrustedValsSigned { reason }),
            Verdict::Invalid(detail) => Err(Error::VerificationError {
                detail: Box::new(detail),
            }),
        }
    }
}
