//! Defines the trait to be implemented by all concrete consensus state types

use crate::prelude::*;

use core::marker::{Send, Sync};

use crate::core::ics23_commitment::commitment::CommitmentRoot;
use crate::core::timestamp::Timestamp;

pub trait ConsensusState: Send + Sync {
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
    fn encode_vec(&self) -> Result<Vec<u8>, tendermint_proto::Error>;
}
