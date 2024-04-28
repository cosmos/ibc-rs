//! Defines the trait to be implemented by all concrete consensus state types

use ibc_core_commitment_types::commitment::CommitmentRoot;
use ibc_primitives::prelude::*;
use ibc_primitives::proto::Any;
use ibc_primitives::Timestamp;

use crate::Convertible;

/// Defines methods that all `ConsensusState`s should provide.
///
/// One can think of a "consensus state" as a pruned header, to be stored on chain. In other words,
/// a consensus state only contains the header's information needed by IBC message handlers.
pub trait ConsensusState: Send + Sync + Convertible<Any> {
    /// Commitment root of the consensus state, which is used for key-value pair verification.
    fn root(&self) -> &CommitmentRoot;

    /// The timestamp of the consensus state
    fn timestamp(&self) -> Timestamp;
}
