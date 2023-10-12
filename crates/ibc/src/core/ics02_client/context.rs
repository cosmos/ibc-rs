use super::client_state::ClientState;
use super::consensus_state::ConsensusState;
use crate::core::ics24_host::identifier::ClientId;
use crate::core::ics24_host::path::{ClientConsensusStatePath, ClientStatePath};
use crate::core::timestamp::Timestamp;
use crate::core::ContextError;
use crate::Height;

/// Defines the methods available to clients for validating client state
/// transitions. The generic `V` parameter in
/// [crate::core::ics02_client::client_state::ClientStateValidation] must
/// inherit from this trait.
pub trait ClientValidationContext {
    /// Returns the time when the client state for the given [`ClientId`] was updated with a header for the given [`Height`]
    fn client_update_time(
        &self,
        client_id: &ClientId,
        height: &Height,
    ) -> Result<Timestamp, ContextError>;

    /// Returns the height when the client state for the given [`ClientId`] was updated with a header for the given [`Height`]
    fn client_update_height(
        &self,
        client_id: &ClientId,
        height: &Height,
    ) -> Result<Height, ContextError>;
}

/// Defines the methods that all client `ExecutionContext`s (precisely the
/// generic parameter of
/// [`crate::core::ics02_client::client_state::ClientStateExecution`] ) must
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

    /// Called upon successful client update.
    /// Implementations are expected to use this to record the specified time as the time at which
    /// this update (or header) was processed.
    fn store_update_time(
        &mut self,
        client_id: ClientId,
        height: Height,
        host_timestamp: Timestamp,
    ) -> Result<(), ContextError>;

    /// Called upon successful client update.
    /// Implementations are expected to use this to record the specified height as the height at
    /// at which this update (or header) was processed.
    fn store_update_height(
        &mut self,
        client_id: ClientId,
        height: Height,
        host_height: Height,
    ) -> Result<(), ContextError>;
}
