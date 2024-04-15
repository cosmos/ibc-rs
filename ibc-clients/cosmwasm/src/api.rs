use ibc_core::client::context::client_state::ClientStateExecution;
use ibc_core::client::context::consensus_state::ConsensusState as ConsensusStateTrait;
use ibc_core::client::types::error::ClientError;
use ibc_core::primitives::proto::Any;

use crate::context::Context;

/// Enables the introduction of custom client and consensus state types tailored
/// for Sovereign light clients.
pub trait ClientType<'a>: Sized {
    type ClientState: ClientStateExecution<Context<'a, Self>> + Clone;
    type ConsensusState: ConsensusStateTrait + Into<Any> + TryFrom<Any, Error = ClientError>;
}
