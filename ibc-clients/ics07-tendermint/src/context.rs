use ibc_core_client::context::ClientExecutionContext;
use ibc_core_client::types::Height;
use ibc_core_handler_types::error::ContextError;
use ibc_core_host::types::identifiers::ClientId;
use ibc_core_host::types::path::ClientConsensusStatePath;
use ibc_primitives::prelude::*;
use ibc_primitives::Timestamp;
use tendermint_light_client_verifier::ProdVerifier;

use crate::consensus_state::ConsensusState as TmConsensusState;

/// Client's context required during both validation and execution
pub trait CommonContext {
    type ConversionError: ToString;
    type AnyConsensusState: TryInto<TmConsensusState, Error = Self::ConversionError>;

    /// Returns the current timestamp of the local chain.
    fn host_timestamp(&self) -> Result<Timestamp, ContextError>;

    /// Returns the current height of the local chain.
    fn host_height(&self) -> Result<Height, ContextError>;

    /// Retrieve the consensus state for the given client ID at the specified
    /// height.
    ///
    /// Returns an error if no such state exists.
    fn consensus_state(
        &self,
        client_cons_state_path: &ClientConsensusStatePath,
    ) -> Result<Self::AnyConsensusState, ContextError>;

    /// Returns all the heights at which a consensus state is stored
    fn consensus_state_heights(&self, client_id: &ClientId) -> Result<Vec<Height>, ContextError>;
}

/// Client's context required during validation
pub trait ValidationContext: CommonContext {
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
///
/// This trait is automatically implemented for all types that implement
/// [`CommonContext`] and [`ClientExecutionContext`]
pub trait ExecutionContext: CommonContext + ClientExecutionContext {}

impl<T> ExecutionContext for T where T: CommonContext + ClientExecutionContext {}

/// Specifies the Verifier interface that hosts must adhere to when customizing
/// Tendermint client verification behaviour.
///
/// Defaults to utilizing the `ProdVerifier` provided by the Tendermint light client.
pub trait TmVerifier {
    type Verifier: tendermint_light_client_verifier::Verifier;

    fn verifier(&self) -> Self::Verifier;
}

/// The default verifier for IBC clients that don't require custom verification logic.
pub struct DefaultVerifier;

impl TmVerifier for DefaultVerifier {
    type Verifier = ProdVerifier;

    fn verifier(&self) -> Self::Verifier {
        ProdVerifier::default()
    }
}
