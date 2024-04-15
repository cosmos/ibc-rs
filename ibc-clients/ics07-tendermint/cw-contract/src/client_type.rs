use ibc_client_cw::api::ClientType;
use ibc_client_tendermint::client_state::ClientState;
use ibc_client_tendermint::consensus_state::ConsensusState;
use ibc_client_tendermint::types::{
    ConsensusState as ConsensusStateType, TENDERMINT_CONSENSUS_STATE_TYPE_URL,
};
use ibc_core::client::types::error::ClientError;
use ibc_core::derive::ConsensusState as ConsensusStateDerive;
use ibc_core::primitives::proto::Any;

/// A unit struct that represents the Tendermint client type.
#[derive(Clone, Debug)]
pub struct TendermintClient;

impl<'a> ClientType<'a> for TendermintClient {
    type ClientState = ClientState;
    type ConsensusState = AnyConsensusState;
}

#[derive(Clone, Debug, ConsensusStateDerive)]
pub enum AnyConsensusState {
    Tendermint(ConsensusState),
}

impl From<ConsensusStateType> for AnyConsensusState {
    fn from(value: ConsensusStateType) -> Self {
        AnyConsensusState::Tendermint(value.into())
    }
}

impl TryFrom<AnyConsensusState> for ConsensusStateType {
    type Error = ClientError;

    fn try_from(value: AnyConsensusState) -> Result<Self, Self::Error> {
        match value {
            AnyConsensusState::Tendermint(state) => Ok(state.into_inner()),
        }
    }
}

impl From<AnyConsensusState> for Any {
    fn from(value: AnyConsensusState) -> Self {
        match value {
            AnyConsensusState::Tendermint(cs) => cs.into(),
        }
    }
}

impl TryFrom<Any> for AnyConsensusState {
    type Error = ClientError;

    fn try_from(raw: Any) -> Result<Self, Self::Error> {
        match raw.type_url.as_str() {
            TENDERMINT_CONSENSUS_STATE_TYPE_URL => {
                let cs = ConsensusState::try_from(raw)?;
                Ok(AnyConsensusState::Tendermint(cs))
            }
            _ => Err(ClientError::UnknownConsensusStateType {
                consensus_state_type: raw.type_url,
            }),
        }
    }
}
