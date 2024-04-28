use ibc_core_client_types::Height;
use ibc_core_handler_types::error::ContextError;
use ibc_core_host_types::identifiers::ClientId;
use ibc_core_host_types::path::{ClientConsensusStatePath, ClientStatePath};
use ibc_primitives::prelude::*;
use ibc_primitives::Timestamp;

use crate::client_state::{ClientStateExecution, ClientStateValidation};
use crate::consensus_state::ConsensusState;

/// Defines the methods available to clients for validating client state
/// transitions. The generic `V` parameter in
/// [crate::client_state::ClientStateValidation] must
/// inherit from this trait.
pub trait ClientValidationContext: Sized {
    type ClientStateRef: ClientStateValidation<Self>;
    type ConsensusStateRef: ConsensusState;

    /// Returns the ClientState for the given identifier `client_id`.
    ///
    /// Note: Clients have the responsibility to store client states on client creation and update.
    fn client_state(&self, client_id: &ClientId) -> Result<Self::ClientStateRef, ContextError>;

    /// Retrieve the consensus state for the given client ID at the specified
    /// height.
    ///
    /// Returns an error if no such state exists.
    ///
    /// Note: Clients have the responsibility to store consensus states on client creation and update.
    fn consensus_state(
        &self,
        client_cons_state_path: &ClientConsensusStatePath,
    ) -> Result<Self::ConsensusStateRef, ContextError>;

    /// Returns the timestamp and height of the host when it processed a client
    /// update request at the specified height.
    fn client_update_meta(
        &self,
        client_id: &ClientId,
        height: &Height,
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
pub trait ClientExecutionContext:
    ClientValidationContext<ClientStateRef = Self::ClientStateMut>
{
    type ClientStateMut: ClientStateExecution<Self>;

    fn client_state_mut(&self, client_id: &ClientId) -> Result<Self::ClientStateMut, ContextError> {
        self.client_state(client_id)
    }

    /// Called upon successful client creation and update
    fn store_client_state(
        &mut self,
        client_state_path: ClientStatePath,
        client_state: Self::ClientStateRef,
    ) -> Result<(), ContextError>;

    /// Called upon successful client creation and update
    fn store_consensus_state(
        &mut self,
        consensus_state_path: ClientConsensusStatePath,
        consensus_state: Self::ConsensusStateRef,
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
        client_id: ClientId,
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
        client_id: ClientId,
        height: Height,
    ) -> Result<(), ContextError>;
}

/// An optional trait that extends the client validation context capabilities by
/// providing additional methods for validating a client state. Mainly
/// benefiting ICS-07 Tendermint clients by granting access to essential
/// information from hosts.
///
/// Categorized under ICS-02, as it may also be utilized by other types of light
/// clients. Developers may view this trait as an example of a custom context
/// definition that expands client validation capabilities, according to their
/// specific light client requirements.
pub trait ExtClientValidationContext: ClientValidationContext {
    /// Returns the current timestamp of the local chain.
    fn host_timestamp(&self) -> Result<Timestamp, ContextError>;

    /// Returns the current height of the local chain.
    fn host_height(&self) -> Result<Height, ContextError>;

    /// Returns all the heights at which a consensus state is stored.
    fn consensus_state_heights(&self, client_id: &ClientId) -> Result<Vec<Height>, ContextError>;

    /// Search for the lowest consensus state higher than `height`.
    fn next_consensus_state(
        &self,
        client_id: &ClientId,
        height: &Height,
    ) -> Result<Option<Self::ConsensusStateRef>, ContextError>;

    /// Search for the highest consensus state lower than `height`.
    fn prev_consensus_state(
        &self,
        client_id: &ClientId,
        height: &Height,
    ) -> Result<Option<Self::ConsensusStateRef>, ContextError>;
}

/// An optional trait that extends the client context required during execution.
///
/// This trait, as it stands right now, serves as a trait alias for types that
/// implement both [`ExtClientValidationContext`] and
/// [`ClientExecutionContext`], and it is auto-implemented for such types.
///
/// Light client developers who wish to define and utilize their own custom
/// client contexts may choose to introduce execution methods within a similar
/// trait, which would allow them to store additional client-specific data and
/// conduct execution operations more efficiently, tailored to their light
/// client implementation.
pub trait ExtClientExecutionContext: ExtClientValidationContext + ClientExecutionContext {}

impl<T> ExtClientExecutionContext for T where T: ExtClientValidationContext + ClientExecutionContext {}

/// General-purpose helper converter enabling `TryFrom` and `Into` conversions
/// primarily intended between an enum and its variants. This usually used by
/// standalone functions as a trait bound allowing them to obtain the concrete
/// local type from the enum containing that concrete type as its variant, like
/// when enum `AnyConsensusState` contains the Tendermint `ConsensusState`.
pub trait Convertible<C>: TryFrom<C> + Into<C> {}

impl<T, C> Convertible<C> for T where T: TryFrom<C> + Into<C> {}
