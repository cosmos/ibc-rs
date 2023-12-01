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
    /// decoding failure: `{0}`
    DecodingFailure(String),
}

#[cfg(feature = "std")]
impl std::error::Error for CommitmentError {}
