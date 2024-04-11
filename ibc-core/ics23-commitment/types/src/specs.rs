//! Defines proof specs, which encode the structure of proofs

use ibc_primitives::prelude::*;
use ibc_proto::ics23::{InnerSpec as RawInnerSpec, LeafOp as RawLeafOp, ProofSpec as RawProofSpec};

use crate::error::CommitmentError;
/// An array of proof specifications.
///
/// This type encapsulates different types of proof specifications, mostly predefined, e.g., for
/// Cosmos-SDK.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct ProofSpecs(Vec<ProofSpec>);

impl ProofSpecs {
    /// Returns the specification for Cosmos-SDK proofs
    pub fn cosmos() -> Self {
        vec![
            ics23::iavl_spec(),       // Format of proofs-iavl (iavl merkle proofs)
            ics23::tendermint_spec(), // Format of proofs-tendermint (crypto/ merkle SimpleProof)
        ]
        .try_into()
        .expect("should convert successfully")
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn validate(&self) -> Result<(), CommitmentError> {
        if self.is_empty() {
            return Err(CommitmentError::EmptyProofSpecs);
        }
        for proof_spec in &self.0 {
            // A non-positive `min_depth` or `max_depth` indicates no limit on the respective bound.
            // Both positive `min_depth` and `max_depth` can be specified. However, in this case,
            //  `max_depth` must be greater than `min_depth` to ensure a valid range.
            if 0 < proof_spec.0.min_depth
                && 0 < proof_spec.0.max_depth
                && proof_spec.0.max_depth < proof_spec.0.min_depth
            {
                return Err(CommitmentError::InvalidDepthRange(
                    proof_spec.0.min_depth,
                    proof_spec.0.max_depth,
                ));
            }
        }
        Ok(())
    }
}

impl TryFrom<Vec<RawProofSpec>> for ProofSpecs {
    type Error = CommitmentError;
    fn try_from(ics23_specs: Vec<RawProofSpec>) -> Result<Self, CommitmentError> {
        let mut specs = Vec::new();
        for raw_spec in ics23_specs {
            let spec = ProofSpec::try_from(raw_spec)?;
            specs.push(spec);
        }
        Ok(ProofSpecs(specs))
    }
}

impl From<ProofSpecs> for Vec<RawProofSpec> {
    fn from(specs: ProofSpecs) -> Self {
        specs.0.into_iter().map(Into::into).collect()
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
struct ProofSpec(RawProofSpec);

impl TryFrom<RawProofSpec> for ProofSpec {
    type Error = CommitmentError;
    fn try_from(spec: RawProofSpec) -> Result<Self, CommitmentError> {
        // A non-positive `min_depth` or `max_depth` indicates no limit on the respective bound.
        // Both positive `min_depth` and `max_depth` can be specified. However, in this case,
        //  `max_depth` must be greater than `min_depth` to ensure a valid range.
        if 0 < spec.min_depth && 0 < spec.max_depth && spec.max_depth < spec.min_depth {
            return Err(CommitmentError::InvalidDepthRange(
                spec.min_depth,
                spec.max_depth,
            ));
        }

        let leaf_spec = spec.leaf_spec.map(LeafOp::from).map(|lop| lop.0);
        let inner_spec = spec
            .inner_spec
            .map(InnerSpec::try_from)
            .transpose()?
            .map(|ispec| ispec.0);

        Ok(Self(RawProofSpec {
            leaf_spec,
            inner_spec,
            max_depth: spec.max_depth,
            min_depth: spec.min_depth,
            prehash_key_before_comparison: spec.prehash_key_before_comparison,
        }))
    }
}

impl From<ProofSpec> for RawProofSpec {
    fn from(spec: ProofSpec) -> Self {
        let spec = spec.0;
        RawProofSpec {
            leaf_spec: spec.leaf_spec.map(|lop| LeafOp(lop).into()),
            inner_spec: spec.inner_spec.map(|ispec| InnerSpec(ispec).into()),
            max_depth: spec.max_depth,
            min_depth: spec.min_depth,
            prehash_key_before_comparison: spec.prehash_key_before_comparison,
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
struct LeafOp(RawLeafOp);

impl From<RawLeafOp> for LeafOp {
    fn from(leaf_op: RawLeafOp) -> Self {
        Self(RawLeafOp {
            hash: leaf_op.hash,
            prehash_key: leaf_op.prehash_key,
            prehash_value: leaf_op.prehash_value,
            length: leaf_op.length,
            prefix: leaf_op.prefix,
        })
    }
}

impl From<LeafOp> for RawLeafOp {
    fn from(leaf_op: LeafOp) -> Self {
        let leaf_op = leaf_op.0;
        RawLeafOp {
            hash: leaf_op.hash,
            prehash_key: leaf_op.prehash_key,
            prehash_value: leaf_op.prehash_value,
            length: leaf_op.length,
            prefix: leaf_op.prefix,
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
struct InnerSpec(RawInnerSpec);

impl TryFrom<RawInnerSpec> for InnerSpec {
    type Error = CommitmentError;
    fn try_from(inner_spec: RawInnerSpec) -> Result<Self, CommitmentError> {
        if inner_spec.child_size <= 0 {
            return Err(CommitmentError::InvalidChildSize(inner_spec.child_size));
        }
        if inner_spec.max_prefix_length < inner_spec.min_prefix_length
            || inner_spec.min_prefix_length < 0
            || inner_spec.max_prefix_length < 0
        {
            return Err(CommitmentError::InvalidPrefixLengthRange(
                inner_spec.min_prefix_length,
                inner_spec.max_prefix_length,
            ));
        }

        Ok(Self(RawInnerSpec {
            child_order: inner_spec.child_order,
            child_size: inner_spec.child_size,
            min_prefix_length: inner_spec.min_prefix_length,
            max_prefix_length: inner_spec.max_prefix_length,
            empty_child: inner_spec.empty_child,
            hash: inner_spec.hash,
        }))
    }
}

impl From<InnerSpec> for RawInnerSpec {
    fn from(inner_spec: InnerSpec) -> Self {
        let inner_spec = inner_spec.0;
        RawInnerSpec {
            child_order: inner_spec.child_order,
            child_size: inner_spec.child_size,
            min_prefix_length: inner_spec.min_prefix_length,
            max_prefix_length: inner_spec.max_prefix_length,
            empty_child: inner_spec.empty_child,
            hash: inner_spec.hash,
        }
    }
}

#[cfg(test)]
mod tests {
    use ibc_proto::ics23::{InnerSpec as RawInnerSpec, ProofSpec as RawProofSpec};

    use super::*;
    #[test]
    fn test_proof_specs_try_from_ok() {
        let valid_raw_proof_spec = vec![
            RawProofSpec {
                leaf_spec: None,
                inner_spec: None,
                max_depth: 5,
                min_depth: 3,
                prehash_key_before_comparison: false,
            },
            RawProofSpec {
                leaf_spec: None,
                inner_spec: None,
                max_depth: -3,
                min_depth: 3,
                prehash_key_before_comparison: false,
            },
            RawProofSpec {
                leaf_spec: None,
                inner_spec: None,
                max_depth: 2,
                min_depth: -6,
                prehash_key_before_comparison: false,
            },
            RawProofSpec {
                leaf_spec: None,
                inner_spec: None,
                max_depth: -2,
                min_depth: -6,
                prehash_key_before_comparison: false,
            },
            RawProofSpec {
                leaf_spec: None,
                inner_spec: None,
                max_depth: -6,
                min_depth: -2,
                prehash_key_before_comparison: false,
            },
        ];
        let specs = ProofSpecs::try_from(valid_raw_proof_spec);
        assert!(specs.is_ok());
        assert_eq!(specs.unwrap().0.len(), 5);
    }
    #[test]
    fn test_proof_specs_try_from_err() {
        let invalid_raw_proof_spec = vec![RawProofSpec {
            leaf_spec: None,
            inner_spec: None,
            max_depth: 5,
            min_depth: 6,
            prehash_key_before_comparison: false,
        }];
        let specs = ProofSpecs::try_from(invalid_raw_proof_spec);
        assert!(specs.is_err());
    }
    #[test]
    fn test_inner_specs_try_from_ok() {
        let valid_raw_inner_spec = RawInnerSpec {
            child_order: vec![1],
            child_size: 2,
            min_prefix_length: 1,
            max_prefix_length: 2,
            empty_child: vec![],
            hash: 1,
        };
        let inner_spec = InnerSpec::try_from(valid_raw_inner_spec);
        assert!(inner_spec.is_ok());
    }
    #[test]
    fn test_inner_specs_try_from_err() {
        let invalid_raw_inner_spec = vec![
            RawInnerSpec {
                child_order: vec![1],
                child_size: 2,
                min_prefix_length: 2,
                max_prefix_length: 1,
                empty_child: vec![],
                hash: 1,
            },
            RawInnerSpec {
                child_order: vec![1],
                child_size: 2,
                min_prefix_length: -1,
                max_prefix_length: 1,
                empty_child: vec![],
                hash: 1,
            },
            RawInnerSpec {
                child_order: vec![1],
                child_size: 2,
                min_prefix_length: 1,
                max_prefix_length: -1,
                empty_child: vec![],
                hash: 1,
            },
            RawInnerSpec {
                child_order: vec![1],
                child_size: 2,
                min_prefix_length: -1,
                max_prefix_length: -1,
                empty_child: vec![],
                hash: 1,
            },
            RawInnerSpec {
                child_order: vec![1],
                child_size: 2,
                min_prefix_length: 2,
                max_prefix_length: 1,
                empty_child: vec![],
                hash: 1,
            },
        ];
        for invalid_raw_inner_spec in invalid_raw_inner_spec {
            let inner_spec = InnerSpec::try_from(invalid_raw_inner_spec);
            assert!(inner_spec.is_err());
        }
    }
}
