//! Defines the commitment error type

use displaydoc::Display;
use ibc_core_host_types::error::DecodingError;
use ibc_primitives::prelude::*;

#[derive(Debug, Display)]
pub enum CommitmentError {
    /// decoding error: `{0}`
    Decoding(DecodingError),
    /// missing commitment root
    MissingCommitmentRoot,
    /// missing commitment prefix
    MissingCommitmentPrefix,
    /// missing merkle proof
    MissingMerkleProof,
    /// missing merkle root
    MissingMerkleRoot,
    /// missing verified value
    MissingVerifiedValue,
    /// missing proof specs
    MissingProofSpecs,
    /// mismatched number of proofs: expected `{expected}`, actual `{actual}`
    MismatchedNumberOfProofs { expected: usize, actual: usize },
    /// invalid range: [`{min}`, `{max}`]
    InvalidRange { min: i32, max: i32 },
    /// invalid merkle proof
    InvalidMerkleProof,
    /// invalid child size: `{0}`
    InvalidChildSize(i32),
    /// invalid hash operation: `{0}`
    InvalidHashOp(i32),
    /// invalid length operation: `{0}`
    InvalidLengthOp(i32),
    /// failed to verify membership
    FailedToVerifyMembership,
}

impl From<DecodingError> for CommitmentError {
    fn from(e: DecodingError) -> Self {
        Self::Decoding(e)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for CommitmentError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self {
            Self::Decoding(e) => Some(e),
            _ => None,
        }
    }
}
