//! Defines the trait to be implemented by all concrete consensus state types

use crate::clients::AsAny;
use crate::prelude::*;

use core::fmt::Debug;
use core::marker::{Send, Sync};

use dyn_clone::DynClone;
use ibc_proto::google::protobuf::Any;
use ibc_proto::protobuf::Protobuf as ErasedProtobuf;

use crate::core::ics02_client::error::ClientError;
use crate::core::ics23_commitment::commitment::CommitmentRoot;
use crate::core::timestamp::Timestamp;
use crate::erased::ErasedSerialize;

/// Abstract of consensus state information used by the validity predicate
/// to verify new commits & state roots.
///
/// Users are not expected to implement sealed::ErasedPartialEqConsensusState.
/// Effectively, that trait bound mandates implementors to derive PartialEq,
/// after which our blanket implementation will implement
/// `ErasedPartialEqConsensusState` for their type.
pub trait ConsensusState:
    AsAny
    + sealed::ErasedPartialEqConsensusState
    + DynClone
    + ErasedSerialize
    + ErasedProtobuf<Any, Error = ClientError>
    + Debug
    + Send
    + Sync
{
    /// Commitment root of the consensus state, which is used for key-value pair verification.
    fn root(&self) -> &CommitmentRoot;

    /// The timestamp of the consensus state
    fn timestamp(&self) -> Timestamp;

    /// Convert into a boxed trait object
    fn into_box(self) -> Box<dyn ConsensusState>
    where
        Self: Sized,
    {
        Box::new(self)
    }
}

// Implements `Clone` for `Box<dyn ConsensusState>`
dyn_clone::clone_trait_object!(ConsensusState);

// Implements `serde::Serialize` for all types that have ConsensusState as supertrait
#[cfg(feature = "serde")]
erased_serde::serialize_trait_object!(ConsensusState);

impl PartialEq for dyn ConsensusState {
    fn eq(&self, other: &Self) -> bool {
        self.eq_consensus_state(other)
    }
}

// see https://github.com/rust-lang/rust/issues/31740
impl PartialEq<&Self> for Box<dyn ConsensusState> {
    fn eq(&self, other: &&Self) -> bool {
        self.eq_consensus_state(other.as_ref())
    }
}

mod sealed {
    use super::*;

    pub trait ErasedPartialEqConsensusState {
        fn eq_consensus_state(&self, other: &dyn ConsensusState) -> bool;
    }

    impl<CS> ErasedPartialEqConsensusState for CS
    where
        CS: ConsensusState + PartialEq,
    {
        fn eq_consensus_state(&self, other: &dyn ConsensusState) -> bool {
            other
                .as_any()
                .downcast_ref::<CS>()
                .map_or(false, |h| self == h)
        }
    }
}

pub trait StaticConsensusState: Clone + Debug + Send + Sync {
    type EncodeError;

    /// Commitment root of the consensus state, which is used for key-value pair verification.
    fn root(&self) -> &CommitmentRoot;

    /// The timestamp of the consensus state
    fn timestamp(&self) -> Timestamp;

    /// Serializes the `ConsensusState`. This is expected to be implemented as
    /// first converting to the raw type (i.e. the protobuf definition), and then
    /// serializing that.
    ///
    /// Note that the `Protobuf` trait in `tendermint-proto` provides convenience methods
    /// to do this automatically.
    fn encode_vec(&self) -> Result<Vec<u8>, Self::EncodeError>;
}
