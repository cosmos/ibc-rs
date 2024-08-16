//! Defines the commitment error type

use displaydoc::Display;
use ibc_primitives::prelude::*;

#[derive(Debug, Display)]
pub enum CommitmentError {
    /// empty commitment prefix
    EmptyCommitmentPrefix,
    /// empty merkle proof
    EmptyMerkleProof,
    /// empty merkle root
    EmptyMerkleRoot,
    /// empty verified value
    EmptyVerifiedValue,
    /// empty proof specs
    EmptyProofSpecs,
    /// invalid range: [`{min}`, `{max}`]
    InvalidRange { min: i32, max: i32 },
    /// mismatched number of proofs: expected `{expected}`, got `{actual}`
    MismatchedNumberOfProofs { expected: usize, actual: usize },
    /// invalid merkle proof
    InvalidMerkleProof,
    /// failed decoding commitment proof: `{0}`
    FailedDecoding(String),
    /// invalid child size: `{0}`
    InvalidChildSize(i32),
    /// invalid hash operation: `{0}`
    InvalidHashOp(i32),
    /// invalid length operation: `{0}`
    InvalidLengthOp(i32),

    // TODO(seanchen1991): Can this variant be removed?
    /// failed verification
    VerificationFailure,
}

#[cfg(feature = "std")]
impl std::error::Error for CommitmentError {}
