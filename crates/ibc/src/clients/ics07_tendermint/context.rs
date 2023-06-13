use crate::{
    core::{
        ics02_client::ClientExecutionContext,
        ics24_host::{identifier::ClientId, path::ClientConsensusStatePath},
        timestamp::Timestamp,
        ContextError,
    },
    Height,
};

use super::consensus_state::ConsensusState as TmConsensusState;

/// Client's context required during both validation and execution
pub trait CommonContext {
    type AnyConsensusState: TryInto<TmConsensusState, Error = &'static str>;

    /// Retrieve the consensus state for the given client ID at the specified
    /// height.
    ///
    /// Returns an error if no such state exists.
    fn consensus_state(
        &self,
        client_cons_state_path: &ClientConsensusStatePath,
    ) -> Result<Self::AnyConsensusState, ContextError>;
}

/// Client's context required during validation
pub trait ValidationContext: CommonContext {
    /// Returns the current timestamp of the local chain.
    fn host_timestamp(&self) -> Result<Timestamp, ContextError>;

    /// Search for the lowest consensus state higher than `height`.
    fn next_consensus_state(
        &self,
        client_id: &ClientId,
        height: &Height,
    ) -> Result<Option<Self::AnyConsensusState>, ContextError>;

    /// Search for the highest consensus state lower than `height`.
    fn prev_consensus_state(
        &self,
        client_id: &ClientId,
        height: &Height,
    ) -> Result<Option<Self::AnyConsensusState>, ContextError>;
}

/// Client's context required during execution.
pub trait ExecutionContext: CommonContext + ClientExecutionContext {}
