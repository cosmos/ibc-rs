use crate::core::{
    ics24_host::path::{ClientConsensusStatePath, ClientStatePath},
    ContextError,
};

use super::{client_state::ClientState, consensus_state::ConsensusState};

/// Defines the methods that all client `ExecutionContext`s (precisely the
/// generic parameter of
/// [`crate::core::ics02_client::client_state::ClientStateExecution`] ) must
/// implement.
///
/// Specifically, clients have the responsibility to store their client state
/// and consensus states. This trait defines a uniform interface to do that for
/// all clients.
pub trait ClientExecutionContext: Sized {
    type ClientValidationContext;
    type AnyClientState: ClientState<Self::ClientValidationContext, Self>;
    type AnyConsensusState: ConsensusState;

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
