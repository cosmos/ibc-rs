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
    /// invalid depth range: [{0}, {1}]
    InvalidDepthRange(i32, i32),
    /// mismatch between the number of proofs with that of specs
    NumberOfSpecsMismatch,
    /// mismatch between the number of proofs with that of keys
    NumberOfKeysMismatch,
    /// invalid merkle proof
    InvalidMerkleProof,
    /// proof verification failed
    VerificationFailure,
    /// encoded commitment prefix is not a valid hex string: `{0}`
    EncodingFailure(String),
    /// decoding commitment proof bytes failed: `{0}`
    DecodingFailure(String),
    /// invalid prefix length range: `[{0}, {1}]`
    InvalidPrefixLengthRange(i32, i32),
    /// invalid child size: `{0}`
    InvalidChildSize(i32),
    /// invalid hash operation: `{0}`
    InvalidHashOp(i32),
    /// invalid length operation: `{0}`
    InvalidLengthOp(i32),
}

#[cfg(feature = "std")]
impl std::error::Error for CommitmentError {}
