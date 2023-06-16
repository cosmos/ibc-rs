use crate::{
    core::{
        ics02_client::ClientTypes, ics24_host::identifier::ClientId, timestamp::Timestamp,
        ContextError,
    },
    Height,
};

/// Client's context required during validation
pub trait ValidationContext: ClientTypes {
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
