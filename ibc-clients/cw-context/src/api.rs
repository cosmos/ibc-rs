use ibc_core::client::context::client_state::ClientStateExecution;
use ibc_core::client::context::consensus_state::ConsensusState as ConsensusStateTrait;
use ibc_core::client::types::error::ClientError;
use ibc_core::primitives::proto::Any;

use crate::context::Context;

/// Enables users to integrate their implemented light client by introducing
/// their client state and consensus state types into the generic [`Context`]
/// object.
pub trait ClientType<'a>: Sized
where
    <Self::ClientState as TryFrom<Any>>::Error: Into<ClientError>,
    <Self::ConsensusState as TryFrom<Any>>::Error: Into<ClientError>,
{
    type ClientState: ClientStateExecution<Context<'a, Self>> + Clone;
    type ConsensusState: ConsensusStateTrait;
}
