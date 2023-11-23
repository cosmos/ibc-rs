use ibc_core_client_types::Height;
use ibc_core_handler_types::error::ContextError;
use ibc_core_host_types::identifiers::ClientId;
use ibc_core_host_types::path::{ClientConsensusStatePath, ClientStatePath};
use ibc_primitives::Timestamp;

use super::client_state::ClientState;
use super::consensus_state::ConsensusState;

/// Defines the methods available to clients for validating client state
/// transitions. The generic `V` parameter in
/// [crate::client_state::ClientStateValidation] must
/// inherit from this trait.
pub trait ClientValidationContext {
    /// Returns the time and height when the client state for the given
    /// [`ClientId`] was updated with a header for the given [`Height`]
    fn client_update_meta(
        &self,
        client_id: &ClientId,
        height: Height,
    ) -> Result<(Timestamp, Height), ContextError>;
}

/// Defines the methods that all client `ExecutionContext`s (precisely the
/// generic parameter of
/// [`crate::client_state::ClientStateExecution`] ) must
/// implement.
///
/// Specifically, clients have the responsibility to store their client state
/// and consensus states. This trait defines a uniform interface to do that for
/// all clients.
pub trait ClientExecutionContext: Sized {
    type V: ClientValidationContext;
    type AnyClientState: ClientState<Self::V, Self>;
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

    /// Delete the consensus state from the store located at the given `ClientConsensusStatePath`
    fn delete_consensus_state(
        &mut self,
        consensus_state_path: ClientConsensusStatePath,
    ) -> Result<(), ContextError>;

    /// Called upon successful client update.
    ///
    /// Implementations are expected to use this to record the specified time
    /// and height as the time at which this update (or header) was processed.
    fn store_update_meta(
        &mut self,
        client_id: &ClientId,
        height: Height,
        host_timestamp: Timestamp,
        host_height: Height,
    ) -> Result<(), ContextError>;

    /// Delete the update time and height associated with the client at the
    /// specified height.
    ///
    /// This update time should be associated with a consensus state through the
    /// specified height.
    ///
    /// Note that this timestamp is determined by the host.
    fn delete_update_meta(
        &mut self,
        client_id: &ClientId,
        height: Height,
    ) -> Result<(), ContextError>;
}
