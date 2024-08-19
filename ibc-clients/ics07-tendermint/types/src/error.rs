//! Defines the Tendermint light client's error type

use core::time::Duration;

use displaydoc::Display;
use ibc_core_client_types::error::ClientError;
use ibc_core_commitment_types::error::CommitmentError;
use ibc_core_host_types::error::IdentifierError;
use ibc_primitives::prelude::*;
use ibc_primitives::TimestampError;
use tendermint::{Error as TendermintError, Hash};
use tendermint_light_client_verifier::errors::VerificationErrorDetail as LightClientErrorDetail;
use tendermint_light_client_verifier::operations::VotingPowerTally;
use tendermint_light_client_verifier::Verdict;

/// The main error type for the Tendermint light client
#[derive(Debug, Display)]
pub enum Error {
    /// invalid identifier: `{0}`
    InvalidIdentifier(IdentifierError),
    /// invalid header, failed basic validation: `{description}`
    InvalidHeader { description: String },
    /// invalid client state trust threshold: `{description}`
    InvalidTrustThreshold { description: String },
    /// invalid clock drift; must be greater than 0
    InvalidMaxClockDrift,
    /// invalid client proof specs: `{0}`
    InvalidProofSpec(CommitmentError),
    /// invalid raw client state: `{description}`
    InvalidRawClientState { description: String },
    /// invalid raw header error: `{0}`
    InvalidRawHeader(TendermintError),
    /// invalid raw misbehaviour: `{description}`
    InvalidRawMisbehaviour { description: String },
    /// invalid header timestamp: `{0}`
    InvalidHeaderTimestamp(TimestampError),
    /// invalid header height: `{0}`
    InvalidHeaderHeight(u64),
    /// missing signed header
    MissingSignedHeader,
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
    /// missing the latest height
    MissingLatestHeight,
    /// missing frozen height
    MissingFrozenHeight,
    /// mismatched revision heights: expected `{expected}`, actual `{actual}`
    MismatchedRevisionHeights { expected: u64, actual: u64 },
    /// mismatched header chain ids: expected `{expected}`, actual `{actual}`
    MismatchedHeaderChainIds { expected: String, actual: String },
    /// mismatched validator hashes: expected `{expected}`, actual `{actual}`
    MismatchedValidatorHashes { expected: Hash, actual: Hash },
    /// insufficient validator overlap: `{0}`
    InsufficientValidatorOverlap(VotingPowerTally),
    /// light client verifier returned an error: `{0}`
    LightClientVerifierError(Box<LightClientErrorDetail>),
    /// consensus state timestamp `{duration_since_consensus_state:?}` should be < `{trusting_period:?}`
    ConsensusStateTimestampGteTrustingPeriod {
        duration_since_consensus_state: Duration,
        trusting_period: Duration,
    },
}

#[cfg(feature = "std")]
impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self {
            Self::InvalidIdentifier(e) => Some(e),
            Self::InvalidRawHeader(e) => Some(e),
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
            Verdict::NotEnoughTrust(tally) => Err(Error::InsufficientValidatorOverlap(tally)),
            Verdict::Invalid(detail) => Err(Error::LightClientVerifierError(Box::new(detail))),
        }
    }
}
