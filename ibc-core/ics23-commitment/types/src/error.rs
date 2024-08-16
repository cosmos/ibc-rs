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
    /// mismatched number of proofs: expected `{expected}`, actual `{actual}`
    MismatchedNumberOfProofs { expected: usize, actual: usize },
    /// mismatched proofs: expected `{expected}`, actual `{actual}`
    MismatchedProofs { expected: String, actual: String },
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
    /// failed decoding commitment proof: `{0}`
    FailedDecoding(String),
    /// failed to verify membership
    FailedToVerifyMembership,
}

#[cfg(feature = "std")]
impl std::error::Error for CommitmentError {}
