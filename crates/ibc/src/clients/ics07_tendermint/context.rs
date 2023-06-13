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

pub trait ValidationContext {
    type AnyConsensusState: TryInto<TmConsensusState, Error = &'static str>;

    /// Returns the current height of the local chain.
    fn host_height(&self) -> Result<Height, ContextError>;

    /// Returns the current timestamp of the local chain.
    fn host_timestamp(&self) -> Result<Timestamp, ContextError>;

    /// Retrieve the consensus state for the given client ID at the specified
    /// height.
    ///
    /// Returns an error if no such state exists.
    fn consensus_state(
        &self,
        client_cons_state_path: &ClientConsensusStatePath,
    ) -> Result<Self::AnyConsensusState, ContextError>;

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

pub trait ExecutionContext: ValidationContext + ClientExecutionContext {}
