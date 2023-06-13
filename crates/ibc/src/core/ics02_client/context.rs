use crate::core::ics24_host::path::{ClientConsensusStatePath, ClientStatePath};

use super::{client_state::ClientState, consensus_state::ConsensusState, error::ClientError};

pub trait ClientExecutionContext: Sized {
    type ClientValidationContext;
    type AnyClientState: ClientState<Self::ClientValidationContext, Self>;
    type AnyConsensusState: ConsensusState;

    /// Called upon successful client creation and update
    fn store_client_state(
        &mut self,
        client_state_path: ClientStatePath,
        client_state: Self::AnyClientState,
    ) -> Result<(), ClientError>;

    /// Called upon successful client creation and update
    fn store_consensus_state(
        &mut self,
        consensus_state_path: ClientConsensusStatePath,
        consensus_state: Self::AnyConsensusState,
    ) -> Result<(), ClientError>;
}
