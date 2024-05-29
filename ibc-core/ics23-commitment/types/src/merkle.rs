//! Merkle proof utilities

use ibc_primitives::prelude::*;
use ibc_primitives::proto::Protobuf;
use ibc_proto::ibc::core::commitment::v1::{MerklePath, MerkleProof as RawMerkleProof, MerkleRoot};
use ibc_proto::ics23::commitment_proof::Proof;
use ibc_proto::ics23::{
    calculate_existence_root, verify_membership, verify_non_membership, CommitmentProof,
    HostFunctionsProvider, NonExistenceProof,
};

use crate::commitment::{CommitmentPrefix, CommitmentRoot};
use crate::error::CommitmentError;
use crate::specs::ProofSpecs;

pub fn apply_prefix(prefix: &CommitmentPrefix, mut path: Vec<String>) -> MerklePath {
    let mut key_path: Vec<String> = vec![format!("{prefix:?}")];
    key_path.append(&mut path);
    MerklePath { key_path }
}

impl From<CommitmentRoot> for MerkleRoot {
    fn from(root: CommitmentRoot) -> Self {
        Self {
            hash: root.into_vec(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct MerkleProof {
    pub proofs: Vec<CommitmentProof>,
}

impl Protobuf<RawMerkleProof> for MerkleProof {}

impl TryFrom<RawMerkleProof> for MerkleProof {
    type Error = CommitmentError;

    fn try_from(proof: RawMerkleProof) -> Result<Self, Self::Error> {
        Ok(Self {
            proofs: proof.proofs,
        })
    }
}

impl From<MerkleProof> for RawMerkleProof {
    fn from(proof: MerkleProof) -> Self {
        Self {
            proofs: proof.proofs,
        }
    }
}

impl MerkleProof {
    pub fn verify_membership<H: HostFunctionsProvider>(
        &self,
        specs: &ProofSpecs,
        root: MerkleRoot,
        keys: MerklePath,
        value: Vec<u8>,
        start_index: u64,
    ) -> Result<(), CommitmentError> {
        // validate arguments
        if self.proofs.is_empty() {
            return Err(CommitmentError::EmptyMerkleProof);
        }
        if root.hash.is_empty() {
            return Err(CommitmentError::EmptyMerkleRoot);
        }
        let num = self.proofs.len();
        let ics23_specs = Vec::<ics23::ProofSpec>::from(specs.clone());
        if ics23_specs.len() != num {
            return Err(CommitmentError::NumberOfSpecsMismatch);
        }
        if keys.key_path.len() != num {
            return Err(CommitmentError::NumberOfKeysMismatch);
        }
        if value.is_empty() {
            return Err(CommitmentError::EmptyVerifiedValue);
        }

        let mut subroot = value.clone();
        let mut value = value;
        // keys are represented from root-to-leaf
        for ((proof, spec), key) in self
            .proofs
            .iter()
            .zip(ics23_specs.iter())
            .zip(keys.key_path.iter().rev())
            .skip(
                start_index
                    .try_into()
                    .expect("safe because if u64 is more than usize it will skip all anyway"),
            )
        {
            match &proof.proof {
                Some(Proof::Exist(existence_proof)) => {
                    subroot = calculate_existence_root::<H>(existence_proof)
                        .map_err(|_| CommitmentError::InvalidMerkleProof)?;

                    if !verify_membership::<H>(proof, spec, &subroot, key.as_bytes(), &value) {
                        return Err(CommitmentError::VerificationFailure);
                    }
                    value.clone_from(&subroot);
                }
                _ => return Err(CommitmentError::InvalidMerkleProof),
            }
        }

        if root.hash != subroot {
            return Err(CommitmentError::VerificationFailure);
        }

        Ok(())
    }

    pub fn verify_non_membership<H: HostFunctionsProvider>(
        &self,
        specs: &ProofSpecs,
        root: MerkleRoot,
        keys: MerklePath,
    ) -> Result<(), CommitmentError> {
        // validate arguments
        if self.proofs.is_empty() {
            return Err(CommitmentError::EmptyMerkleProof);
        }
        if root.hash.is_empty() {
            return Err(CommitmentError::EmptyMerkleRoot);
        }
        let num = self.proofs.len();
        let ics23_specs = Vec::<ics23::ProofSpec>::from(specs.clone());
        if ics23_specs.len() != num {
            return Err(CommitmentError::NumberOfSpecsMismatch);
        }
        if keys.key_path.len() != num {
            return Err(CommitmentError::NumberOfKeysMismatch);
        }

        // verify the absence of key in lowest subtree
        let proof = self
            .proofs
            .first()
            .ok_or(CommitmentError::InvalidMerkleProof)?;
        let spec = ics23_specs
            .first()
            .ok_or(CommitmentError::InvalidMerkleProof)?;
        // keys are represented from root-to-leaf
        let key = keys
            .key_path
            .get(num - 1)
            .ok_or(CommitmentError::InvalidMerkleProof)?;
        match &proof.proof {
            Some(Proof::Nonexist(non_existence_proof)) => {
                let subroot = calculate_non_existence_root::<H>(non_existence_proof)?;

                if !verify_non_membership::<H>(proof, spec, &subroot, key.as_bytes()) {
                    return Err(CommitmentError::VerificationFailure);
                }

                // verify membership proofs starting from index 1 with value = subroot
                self.verify_membership::<H>(specs, root, keys, subroot, 1)
            }
            _ => Err(CommitmentError::InvalidMerkleProof),
        }
    }
}

// TODO move to ics23
fn calculate_non_existence_root<H: HostFunctionsProvider>(
    proof: &NonExistenceProof,
) -> Result<Vec<u8>, CommitmentError> {
    if let Some(left) = &proof.left {
        calculate_existence_root::<H>(left).map_err(|_| CommitmentError::InvalidMerkleProof)
    } else if let Some(right) = &proof.right {
        calculate_existence_root::<H>(right).map_err(|_| CommitmentError::InvalidMerkleProof)
    } else {
        Err(CommitmentError::InvalidMerkleProof)
    }
}
