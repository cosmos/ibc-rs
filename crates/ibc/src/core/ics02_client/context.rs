use crate::core::ics02_client::client_state::ClientState;
use crate::core::ics02_client::consensus_state::ConsensusState;
use crate::core::ics24_host::identifier::ClientId;
use crate::core::ics24_host::path::ClientConsensusStatePath;
use crate::core::ics24_host::path::ClientStatePath;
use crate::core::ContextError;

/// Defines the types that all clients must implement.
pub trait ClientTypes: Sized {
    type V: ClientValidationContext;
    type E: ClientExecutionContext;
    type AnyClientState: ClientState<Self::V, Self::E>;
    type AnyConsensusState: ConsensusState;
}

/// Defines the methods that all client `ValidationContext`s (precisely the
/// generic parameter of
/// [`crate::core::ics02_client::client_state::ClientStateValidation`] ) must
/// implement.
pub trait ClientValidationContext: ClientTypes {
    /// Returns the ClientState for the given identifier `client_id`.
    fn client_state(&self, client_id: &ClientId) -> Result<Self::AnyClientState, ContextError>;

    /// Returns the consensus state for the given client ID at the specified
    /// height.
    fn consensus_state(
        &self,
        client_cons_state_path: &ClientConsensusStatePath,
    ) -> Result<Self::AnyConsensusState, ContextError>;
}

/// Defines the methods that all client `ExecutionContext`s (precisely the
/// generic parameter of
/// [`crate::core::ics02_client::client_state::ClientStateExecution`] ) must
/// implement.
///
/// Specifically, clients have the responsibility to store their client state
/// and consensus states. This trait defines a uniform interface to do that for
/// all clients.
pub trait ClientExecutionContext: ClientValidationContext {
    /// Called upon successful client creation and update
    fn store_client_state(
        &mut self,
        client_state_path: ClientStatePath,
        client_state: Self::AnyClientState,
    ) -> Result<(), ContextError>;

    /// Called upon successful client creation and update
    fn store_consensus_state(
        &mut self,
        consensus_state_path: ClientConsensusStatePath,
        consensus_state: Self::AnyConsensusState,
    ) -> Result<(), ContextError>;
}
