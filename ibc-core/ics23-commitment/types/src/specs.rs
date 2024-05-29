//! Defines proof specs, which encode the structure of proofs

use ibc_primitives::prelude::*;
use ibc_proto::ics23::{InnerSpec as RawInnerSpec, LeafOp as RawLeafOp, ProofSpec as RawProofSpec};
use ics23::{HashOp, LengthOp};

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
            // For simplicity, negative values for `min_depth` and `max_depth` are not allowed
            // and only `0` is used to indicate no limit. When `min_depth` and `max_depth` are both positive,
            // `max_depth` must be greater than or equal to `min_depth` to ensure a valid range.
            if proof_spec.0.max_depth < 0
                || proof_spec.0.min_depth < 0
                || (0 < proof_spec.0.min_depth
                    && 0 < proof_spec.0.max_depth
                    && proof_spec.0.max_depth < proof_spec.0.min_depth)
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
        // no proof specs provided
        if ics23_specs.is_empty() {
            return Err(CommitmentError::EmptyProofSpecs);
        }

        ics23_specs
            .into_iter()
            .map(ProofSpec::try_from)
            .collect::<Result<_, _>>()
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
        // For simplicity, negative values for `min_depth` and `max_depth` are not allowed
        // and only `0` is used to indicate no limit. When `min_depth` and `max_depth` are both positive,
        // `max_depth` must be greater than or equal to `min_depth` to ensure a valid range.
        if spec.max_depth < 0
            || spec.min_depth < 0
            || (0 < spec.min_depth && 0 < spec.max_depth && spec.max_depth < spec.min_depth)
        {
            return Err(CommitmentError::InvalidDepthRange(
                spec.min_depth,
                spec.max_depth,
            ));
        }

        let leaf_spec = spec
            .leaf_spec
            .map(LeafOp::try_from)
            .transpose()?
            .map(|lop| lop.0);
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
        spec.0
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
struct LeafOp(RawLeafOp);

impl TryFrom<RawLeafOp> for LeafOp {
    type Error = CommitmentError;
    fn try_from(leaf_op: RawLeafOp) -> Result<Self, Self::Error> {
        let _ = HashOp::try_from(leaf_op.hash)
            .map_err(|_| CommitmentError::InvalidHashOp(leaf_op.hash))?;
        let _ = HashOp::try_from(leaf_op.prehash_key)
            .map_err(|_| CommitmentError::InvalidHashOp(leaf_op.prehash_key))?;
        let _ = HashOp::try_from(leaf_op.prehash_value)
            .map_err(|_| CommitmentError::InvalidHashOp(leaf_op.prehash_value))?;
        let _ = LengthOp::try_from(leaf_op.length)
            .map_err(|_| CommitmentError::InvalidLengthOp(leaf_op.length))?;

        Ok(Self(leaf_op))
    }
}

impl From<LeafOp> for RawLeafOp {
    fn from(leaf_op: LeafOp) -> Self {
        leaf_op.0
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

        // Negative prefix lengths are not allowed and the maximum prefix length must
        // be greater than or equal to the minimum prefix length.
        if inner_spec.min_prefix_length < 0
            || inner_spec.max_prefix_length < 0
            || inner_spec.max_prefix_length < inner_spec.min_prefix_length
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
        inner_spec.0
    }
}

#[cfg(test)]
mod tests {
    use ibc_proto::ics23::{InnerSpec as RawInnerSpec, ProofSpec as RawProofSpec};
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case(0, 0)]
    #[case(2, 2)]
    #[case(5, 6)]
    #[should_panic(expected = "InvalidDepthRange")]
    #[case(-3,3)]
    #[should_panic(expected = "InvalidDepthRange")]
    #[case(2,-6)]
    #[should_panic(expected = "InvalidDepthRange")]
    #[case(-2,-6)]
    #[should_panic(expected = "InvalidDepthRange")]
    #[case(-6,-2)]
    #[should_panic(expected = "InvalidDepthRange")]
    #[case(5, 3)]
    fn test_proof_specs_try_from(#[case] min_depth: i32, #[case] max_depth: i32) {
        let raw_proof_spec = RawProofSpec {
            leaf_spec: None,
            inner_spec: None,
            max_depth,
            min_depth,
            prehash_key_before_comparison: false,
        };
        ProofSpec::try_from(raw_proof_spec).unwrap();
    }

    #[rstest]
    #[case(0, 0)]
    #[case(1, 2)]
    #[case(2, 2)]
    #[should_panic(expected = "InvalidPrefixLengthRange")]
    #[case(2, 1)]
    #[should_panic(expected = "InvalidPrefixLengthRange")]
    #[case(-2,1)]
    #[should_panic(expected = "InvalidPrefixLengthRange")]
    #[case(2,-1)]
    #[should_panic(expected = "InvalidPrefixLengthRange")]
    #[case(-2,-1)]
    #[should_panic(expected = "InvalidPrefixLengthRange")]
    #[case(-1,-2)]
    fn test_inner_specs_try_from(#[case] min_prefix_length: i32, #[case] max_prefix_length: i32) {
        let raw_inner_spec = RawInnerSpec {
            child_order: vec![1],
            child_size: 2,
            min_prefix_length,
            max_prefix_length,
            empty_child: vec![],
            hash: 1,
        };
        InnerSpec::try_from(raw_inner_spec).unwrap();
    }

    #[rstest]
    #[case(0, 0, 0, 0)]
    #[case(9, 9, 9, 8)]
    #[should_panic(expected = "InvalidHashOp")]
    #[case(-1, 4, 4, 4)]
    #[should_panic(expected = "InvalidHashOp")]
    #[case(10, 4, 4, 4)]
    #[should_panic(expected = "InvalidHashOp")]
    #[case(4, -1, 4, 4)]
    #[should_panic(expected = "InvalidHashOp")]
    #[case(4, 10, 4, 4)]
    #[should_panic(expected = "InvalidHashOp")]
    #[case(4, 4, -1, 4)]
    #[should_panic(expected = "InvalidHashOp")]
    #[case(4, 4, 10, 4)]
    #[should_panic(expected = "InvalidLengthOp")]
    #[case(4, 4, 4, -1)]
    #[should_panic(expected = "InvalidLengthOp")]
    #[case(4, 4, 4, 9)]
    fn test_leaf_op_try_from(
        #[case] hash: i32,
        #[case] prehash_key: i32,
        #[case] prehash_value: i32,
        #[case] length: i32,
    ) {
        let raw_leaf_op = RawLeafOp {
            hash,
            prehash_key,
            prehash_value,
            length,
            prefix: vec![],
        };
        LeafOp::try_from(raw_leaf_op).unwrap();
    }
}
