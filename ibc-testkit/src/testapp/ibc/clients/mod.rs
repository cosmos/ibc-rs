pub mod mock;

use alloc::fmt::Debug;

use basecoin_store::context::ProvableStore;
use derive_more::From;
use ibc::clients::tendermint::client_state::ClientState as TmClientState;
use ibc::clients::tendermint::consensus_state::ConsensusState as TmConsensusState;
use ibc::clients::tendermint::types::{
    ClientState as ClientStateType, ConsensusState as ConsensusStateType,
    TENDERMINT_CLIENT_STATE_TYPE_URL, TENDERMINT_CONSENSUS_STATE_TYPE_URL,
};
use ibc::core::client::types::error::ClientError;
use ibc::core::client::types::Height;
use ibc::core::primitives::prelude::*;
use ibc::derive::{ClientState, ConsensusState};
use ibc::primitives::proto::{Any, Protobuf};

use super::core::types::MockIbcStore;
use crate::testapp::ibc::clients::mock::client_state::{
    MockClientState, MOCK_CLIENT_STATE_TYPE_URL,
};
use crate::testapp::ibc::clients::mock::consensus_state::{
    MockConsensusState, MOCK_CONSENSUS_STATE_TYPE_URL,
};

#[derive(Debug, Clone, From, PartialEq, ClientState)]
#[validation(MockIbcStore<S: ProvableStore + Debug>)]
#[execution(MockIbcStore<S: ProvableStore + Debug>)]
pub enum AnyClientState {
    Tendermint(TmClientState),
    Mock(MockClientState),
}

impl AnyClientState {
    pub fn latest_height(&self) -> Height {
        match self {
            Self::Tendermint(cs) => cs.inner().latest_height,
            Self::Mock(cs) => cs.latest_height(),
        }
    }

    pub fn is_frozen(&self) -> bool {
        match self {
            Self::Tendermint(cs) => cs.inner().is_frozen(),
            Self::Mock(cs) => cs.is_frozen(),
        }
    }
}

impl Protobuf<Any> for AnyClientState {}

impl TryFrom<Any> for AnyClientState {
    type Error = ClientError;

    fn try_from(raw: Any) -> Result<Self, Self::Error> {
        if raw.type_url == TENDERMINT_CLIENT_STATE_TYPE_URL {
            Ok(TmClientState::try_from(raw)?.into())
        } else if raw.type_url == MOCK_CLIENT_STATE_TYPE_URL {
            MockClientState::try_from(raw).map(Into::into)
        } else {
            Err(ClientError::Other {
                description: "failed to deserialize message".to_string(),
            })
        }
    }
}

impl From<AnyClientState> for Any {
    fn from(host_client_state: AnyClientState) -> Self {
        match host_client_state {
            AnyClientState::Tendermint(cs) => cs.into(),
            AnyClientState::Mock(cs) => cs.into(),
        }
    }
}

impl From<ClientStateType> for AnyClientState {
    fn from(client_state: ClientStateType) -> Self {
        AnyClientState::Tendermint(client_state.into())
    }
}

impl From<ConsensusStateType> for AnyConsensusState {
    fn from(consensus_state: ConsensusStateType) -> Self {
        AnyConsensusState::Tendermint(consensus_state.into())
    }
}

#[derive(Debug, Clone, From, PartialEq, Eq, ConsensusState)]
pub enum AnyConsensusState {
    Tendermint(TmConsensusState),
    Mock(MockConsensusState),
}

impl TryFrom<Any> for AnyConsensusState {
    type Error = ClientError;

    fn try_from(raw: Any) -> Result<Self, Self::Error> {
        if raw.type_url == TENDERMINT_CONSENSUS_STATE_TYPE_URL {
            Ok(TmConsensusState::try_from(raw)?.into())
        } else if raw.type_url == MOCK_CONSENSUS_STATE_TYPE_URL {
            MockConsensusState::try_from(raw).map(Into::into)
        } else {
            Err(ClientError::Other {
                description: "failed to deserialize message".to_string(),
            })
        }
    }
}

impl From<AnyConsensusState> for Any {
    fn from(host_consensus_state: AnyConsensusState) -> Self {
        match host_consensus_state {
            AnyConsensusState::Tendermint(cs) => cs.into(),
            AnyConsensusState::Mock(cs) => cs.into(),
        }
    }
}

impl TryFrom<AnyConsensusState> for ConsensusStateType {
    type Error = ClientError;

    fn try_from(value: AnyConsensusState) -> Result<Self, Self::Error> {
        match value {
            AnyConsensusState::Tendermint(cs) => Ok(cs.inner().clone()),
            _ => Err(ClientError::Other {
                description: "failed to convert AnyConsensusState to TmConsensusState".to_string(),
            }),
        }
    }
}

impl TryFrom<AnyConsensusState> for MockConsensusState {
    type Error = ClientError;

    fn try_from(value: AnyConsensusState) -> Result<Self, Self::Error> {
        match value {
            AnyConsensusState::Mock(cs) => Ok(cs),
            _ => Err(ClientError::Other {
                description: "failed to convert AnyConsensusState to MockConsensusState"
                    .to_string(),
            }),
        }
    }
}
