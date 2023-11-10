use ibc::core::ics23_commitment::commitment::CommitmentProofBytes;
use ibc::prelude::*;
use ibc::proto::core::commitment::v1::MerkleProof as RawMerkleProof;
use ibc::proto::ics23::CommitmentProof;

/// Returns a dummy `CommitmentProofBytes`, for testing only!
pub fn dummy_commitment_proof_bytes() -> CommitmentProofBytes {
    let parsed = CommitmentProof { proof: None };
    let mproofs: Vec<CommitmentProof> = vec![parsed];
    let raw_mp = RawMerkleProof { proofs: mproofs };
    raw_mp
        .try_into()
        .expect("could not convert to CommitmentProofBytes")
}
