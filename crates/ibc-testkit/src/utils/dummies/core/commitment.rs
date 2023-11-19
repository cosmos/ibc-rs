use ibc::core::commitment_types::commitment::CommitmentProofBytes;
use ibc::core::commitment_types::proto::ics23::CommitmentProof;
use ibc::core::commitment_types::proto::v1::MerkleProof as RawMerkleProof;
use ibc::core::primitives::prelude::*;

/// Returns a dummy `CommitmentProofBytes`, for testing purposes only!
pub fn dummy_commitment_proof_bytes() -> CommitmentProofBytes {
    let parsed = CommitmentProof { proof: None };
    let mproofs: Vec<CommitmentProof> = vec![parsed];
    let raw_mp = RawMerkleProof { proofs: mproofs };
    raw_mp
        .try_into()
        .expect("could not convert to CommitmentProofBytes")
}
