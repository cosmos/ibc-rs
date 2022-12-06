use displaydoc::Display;
use prost::DecodeError;

#[derive(Debug, Display)]
pub enum CommitmentError {
    /// invalid raw merkle proof error: `{0}`
    InvalidRawMerkleProof(DecodeError),
    /// failed to decode commitment proof error: `{0}`
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

#[cfg(feature = "std")]
impl std::error::Error for CommitmentError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self {
            Self::InvalidRawMerkleProof(e) => Some(e),
            Self::CommitmentProofDecodingFailed(e) => Some(e),
            _ => None,
        }
    }
}
