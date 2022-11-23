use displaydoc::Display;
use prost::DecodeError;

#[derive(Debug, Display)]
pub enum Error {
    /// invalid raw merkle proof, error(`{0}`)
    InvalidRawMerkleProof(DecodeError),
    /// failed to decode commitment proof, error(`{0}`)
    CommitmentProofDecodingFailed(DecodeError),
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
}
