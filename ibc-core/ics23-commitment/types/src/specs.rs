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
            // Both positive `min_depth` and `max_depth` can be specified. However, in that case,
            //  `max_depth` must be greater than or equal to `min_depth` to ensure a valid range.
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
        ics23_specs
            .into_iter()
            .map(ProofSpec::try_from)
            .collect::<Result<Vec<_>, _>>()
            .map(Self)
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
        // Both positive `min_depth` and `max_depth` can be specified. However, in that case,
        //  `max_depth` must be greater than or equal to `min_depth` to ensure a valid range.
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

        // ensure that min_prefix_length and max_prefix_length are non-negative integers.
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
    use rstest::rstest;

    use super::*;

    fn mock_raw_proof_spec(min_depth: i32, max_depth: i32) -> RawProofSpec {
        RawProofSpec {
            leaf_spec: None,
            inner_spec: None,
            max_depth,
            min_depth,
            prehash_key_before_comparison: false,
        }
    }

    fn mock_inner_spec(min_prefix_length: i32, max_prefix_length: i32) -> RawInnerSpec {
        RawInnerSpec {
            child_order: vec![1],
            child_size: 2,
            min_prefix_length,
            max_prefix_length,
            empty_child: vec![],
            hash: 1,
        }
    }

    #[rstest]
    #[case(5, 6)]
    #[case(-3,3)]
    #[case(2,-6)]
    #[case(-2,-6)]
    #[case(-6,-2)]
    fn test_proof_specs_try_from_ok(#[case] min_depth: i32, #[case] max_depth: i32) {
        assert!(ProofSpec::try_from(mock_raw_proof_spec(min_depth, max_depth)).is_ok())
    }

    #[rstest]
    #[case(5, 3)]
    fn test_proof_specs_try_from_err(#[case] min_depth: i32, #[case] max_depth: i32) {
        assert!(ProofSpec::try_from(mock_raw_proof_spec(min_depth, max_depth)).is_err())
    }

    #[rstest]
    #[case(1, 2)]
    fn test_inner_specs_try_from_ok(
        #[case] min_prefix_length: i32,
        #[case] max_prefix_length: i32,
    ) {
        assert!(InnerSpec::try_from(mock_inner_spec(min_prefix_length, max_prefix_length)).is_ok())
    }

    #[rstest]
    #[case(2, 1)]
    #[case(-2,1)]
    #[case(2,-1)]
    #[case(-2,-1)]
    #[case(-1,-2)]
    fn test_inner_specs_try_from_err(
        #[case] min_prefix_length: i32,
        #[case] max_prefix_length: i32,
    ) {
        assert!(InnerSpec::try_from(mock_inner_spec(min_prefix_length, max_prefix_length)).is_err())
    }
}
