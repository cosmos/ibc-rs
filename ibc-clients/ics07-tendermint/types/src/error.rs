//! Defines the Tendermint light client's error type

use core::time::Duration;

use displaydoc::Display;
use ibc_core_client_types::error::ClientError;
use ibc_core_client_types::Height;
use ibc_core_commitment_types::error::CommitmentError;
use ibc_core_host_types::error::{DecodingError, IdentifierError};
use ibc_primitives::prelude::*;
use ibc_primitives::TimestampError;
use tendermint::Hash;
use tendermint_light_client_verifier::errors::VerificationErrorDetail as LightClientErrorDetail;
use tendermint_light_client_verifier::operations::VotingPowerTally;
use tendermint_light_client_verifier::Verdict;

/// The main error type for the Tendermint light client
#[derive(Debug, Display)]
pub enum TendermintClientError {
    /// decoding error: `{0}`
    Decoding(DecodingError),
    /// invalid client state trust threshold: `{description}`
    InvalidTrustThreshold { description: String },
    /// invalid clock drift; must be greater than 0
    InvalidMaxClockDrift,
    /// invalid client proof specs: `{0}`
    InvalidProofSpec(CommitmentError),
    /// invalid header timestamp: `{0}`
    InvalidHeaderTimestamp(TimestampError),
    /// invalid header height: `{0}`
    InvalidHeaderHeight(u64),
    /// mismatched revision heights: expected `{expected}`, actual `{actual}`
    MismatchedRevisionHeights { expected: u64, actual: u64 },
    /// mismatched header chain ids: expected `{expected}`, actual `{actual}`
    MismatchedHeaderChainIds { expected: String, actual: String },
    /// mismatched validator hashes: expected `{expected}`, actual `{actual}`
    MismatchedValidatorHashes { expected: Hash, actual: Hash },
    /// empty client state upgrade-path key
    EmptyUpgradePathKey,
    /// failed to verify header: `{0}`
    FailedToVerifyHeader(Box<LightClientErrorDetail>),
    /// insufficient validator overlap: `{0}`
    InsufficientValidatorOverlap(VotingPowerTally),
    /// insufficient trusting period `{trusting_period:?}`; should be > consensus state timestamp `{duration_since_consensus_state:?}`
    InsufficientTrustingPeriod {
        duration_since_consensus_state: Duration,
        trusting_period: Duration,
    },
    /// insufficient misbehaviour header height: header1 height `{height_1}` shoul be >= header2 height `{height_2}`
    InsufficientMisbehaviourHeaderHeight { height_1: Height, height_2: Height },
}

#[cfg(feature = "std")]
impl std::error::Error for TendermintClientError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self {
            Self::Decoding(e) => Some(e),
            _ => None,
        }
    }
}

// TODO(seanchen1991): Should this impl be deprecated in favor of a
// From<ClientError> for TendermintClientError impl?
impl From<TendermintClientError> for ClientError {
    fn from(e: TendermintClientError) -> Self {
        Self::ClientSpecific {
            description: e.to_string(),
        }
    }
}

impl From<IdentifierError> for TendermintClientError {
    fn from(e: IdentifierError) -> Self {
        Self::Decoding(DecodingError::Identifier(e))
    }
}

impl From<CommitmentError> for TendermintClientError {
    fn from(e: CommitmentError) -> Self {
        Self::InvalidProofSpec(e)
    }
}

impl From<DecodingError> for TendermintClientError {
    fn from(e: DecodingError) -> Self {
        Self::Decoding(e)
    }
}

pub trait IntoResult<T, E> {
    fn into_result(self) -> Result<T, E>;
}

impl IntoResult<(), TendermintClientError> for Verdict {
    fn into_result(self) -> Result<(), TendermintClientError> {
        match self {
            Verdict::Success => Ok(()),
            Verdict::NotEnoughTrust(tally) => {
                Err(TendermintClientError::InsufficientValidatorOverlap(tally))
            }
            Verdict::Invalid(detail) => Err(TendermintClientError::FailedToVerifyHeader(Box::new(
                detail,
            ))),
        }
    }
}
