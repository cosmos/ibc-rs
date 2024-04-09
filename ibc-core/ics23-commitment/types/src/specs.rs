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
        ].try_into().expect("should convert successfully")
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn validate(&self) -> Result<(), CommitmentError> {
        if self.is_empty() {
            return Err(CommitmentError::EmptyProofSpecs);
        }
        for proof_spec in &self.0 {
            if proof_spec.0.max_depth < proof_spec.0.min_depth
                || proof_spec.0.min_depth < 0
                || proof_spec.0.max_depth < 0
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
        if spec.max_depth < spec.min_depth
            || spec.min_depth < 0
            || spec.max_depth < 0
        {
            return Err(CommitmentError::InvalidDepthRange(spec.min_depth, spec.max_depth));
        }

        let leaf_spec = spec.leaf_spec.map(|lop| LeafOp::from(lop)).map(|lop| lop.0);
        let inner_spec = spec.inner_spec
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
         || inner_spec.max_prefix_length < 0 {
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
