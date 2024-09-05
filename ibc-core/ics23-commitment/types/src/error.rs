//! Defines the commitment error type

use displaydoc::Display;
use ibc_core_host_types::error::DecodingError;
use ibc_primitives::prelude::*;

#[derive(Debug, Display)]
pub enum CommitmentError {
    /// decoding error: `{0}`
    Decoding(DecodingError),
    /// empty commitment prefix
    EmptyCommitmentPrefix,
    /// empty commitment root
    EmptyCommitmentRoot,
    /// empty merkle proof
    EmptyMerkleProof,
    /// empty merkle root
    EmptyMerkleRoot,
    /// empty verified value
    EmptyVerifiedValue,
    /// empty proof specs
    EmptyProofSpecs,
    /// mismatched number of proofs: expected `{expected}`, actual `{actual}`
    MismatchedNumberOfProofs { expected: usize, actual: usize },
    /// invalid range: [`{min}`, `{max}`]
    InvalidRange { min: i32, max: i32 },
    /// invalid merkle proof
    InvalidMerkleProof,
    /// invalid child size: `{0}`
    InvalidChildSize(i32),
    /// failed to verify membership
    FailedToVerifyMembership,
}

impl From<DecodingError> for CommitmentError {
    fn from(e: DecodingError) -> Self {
        Self::Decoding(e)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for CommitmentError {}
