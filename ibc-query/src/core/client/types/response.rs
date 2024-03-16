//! Contains all the RPC method response domain types and their conversions to
//! and from the corresponding gRPC proto types for the client module.

use ibc::core::client::types::{Height, Status};
use ibc::core::host::types::identifiers::ClientId;
use ibc::core::primitives::proto::Any;
use ibc::primitives::prelude::*;
use ibc_proto::ibc::core::client::v1::{
    ConsensusStateWithHeight as RawConsensusStateWithHeight,
    IdentifiedClientState as RawIdentifiedClientState, Params as RawParams,
    QueryClientParamsResponse as RawQueryClientParamsResponse,
    QueryClientStateResponse as RawQueryClientStateResponse,
    QueryClientStatesResponse as RawQueryClientStatesResponse,
    QueryClientStatusResponse as RawQueryClientStatusResponse,
    QueryConsensusStateHeightsResponse as RawQueryConsensusStateHeightsResponse,
    QueryConsensusStateResponse as RawQueryConsensusStateResponse,
    QueryConsensusStatesResponse as RawQueryConsensusStatesResponse,
    QueryUpgradedClientStateResponse as RawQueryUpgradedClientStateResponse,
    QueryUpgradedConsensusStateResponse as RawQueryUpgradedConsensusStateResponse,
};
use serde::{Deserialize, Serialize};

use crate::types::{PageResponse, Proof};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QueryClientStateResponse {
    /// The client identifier.
    pub client_state: Any,
    /// The proof that the client state was retrieved.
    pub proof: Proof,
    /// The height at which the client state was retrieved.
    pub proof_height: Height,
}

impl QueryClientStateResponse {
    pub fn new(client_state: Any, proof: Proof, proof_height: Height) -> Self {
        Self {
            client_state,
            proof,
            proof_height,
        }
    }
}

impl From<QueryClientStateResponse> for RawQueryClientStateResponse {
    fn from(response: QueryClientStateResponse) -> Self {
        Self {
            client_state: Some(response.client_state),
            proof: response.proof,
            proof_height: Some(response.proof_height.into()),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QueryClientStatesResponse {
    pub client_states: Vec<IdentifiedClientState>,
    pub pagination: Option<PageResponse>,
}

impl QueryClientStatesResponse {
    pub fn new(
        client_states: Vec<IdentifiedClientState>,
        pagination: Option<PageResponse>,
    ) -> Self {
        Self {
            client_states,
            pagination,
        }
    }
}

impl From<QueryClientStatesResponse> for RawQueryClientStatesResponse {
    fn from(response: QueryClientStatesResponse) -> Self {
        Self {
            client_states: response
                .client_states
                .into_iter()
                .map(RawIdentifiedClientState::from)
                .collect(),
            pagination: response.pagination.map(|pagination| pagination.into()),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IdentifiedClientState {
    pub client_id: ClientId,
    pub client_state: Any,
}

impl IdentifiedClientState {
    pub fn new(client_id: ClientId, client_state: Any) -> Self {
        Self {
            client_id,
            client_state,
        }
    }
}

impl From<IdentifiedClientState> for RawIdentifiedClientState {
    fn from(ics: IdentifiedClientState) -> Self {
        Self {
            client_id: ics.client_id.to_string(),
            client_state: Some(ics.client_state),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QueryConsensusStateResponse {
    consensus_state: Any,
    proof: Proof,
    proof_height: Height,
}

impl QueryConsensusStateResponse {
    pub fn new(consensus_state: Any, proof: Proof, proof_height: Height) -> Self {
        Self {
            consensus_state,
            proof,
            proof_height,
        }
    }
}

impl From<QueryConsensusStateResponse> for RawQueryConsensusStateResponse {
    fn from(response: QueryConsensusStateResponse) -> Self {
        Self {
            consensus_state: Some(response.consensus_state),
            proof: response.proof,
            proof_height: Some(response.proof_height.into()),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConsensusStateWithHeight {
    height: Height,
    consensus_state: Any,
}

impl ConsensusStateWithHeight {
    pub fn new(height: Height, consensus_state: Any) -> Self {
        Self {
            height,
            consensus_state,
        }
    }
}

impl From<ConsensusStateWithHeight> for RawConsensusStateWithHeight {
    fn from(ics: ConsensusStateWithHeight) -> Self {
        Self {
            height: Some(ics.height.into()),
            consensus_state: Some(ics.consensus_state),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QueryConsensusStatesResponse {
    consensus_states: Vec<ConsensusStateWithHeight>,
    pagination: Option<PageResponse>,
}

impl QueryConsensusStatesResponse {
    pub fn new(
        consensus_states: Vec<ConsensusStateWithHeight>,
        pagination: Option<PageResponse>,
    ) -> Self {
        Self {
            consensus_states,
            pagination,
        }
    }
}

impl From<QueryConsensusStatesResponse> for RawQueryConsensusStatesResponse {
    fn from(response: QueryConsensusStatesResponse) -> Self {
        Self {
            consensus_states: response
                .consensus_states
                .into_iter()
                .map(RawConsensusStateWithHeight::from)
                .collect(),
            pagination: response.pagination.map(|pagination| pagination.into()),
        }
    }
}

pub struct QueryConsensusStateHeightsResponse {
    consensus_state_heights: Vec<Height>,
    pagination: Option<PageResponse>,
}

impl QueryConsensusStateHeightsResponse {
    pub fn new(consensus_state_heights: Vec<Height>, pagination: Option<PageResponse>) -> Self {
        Self {
            consensus_state_heights,
            pagination,
        }
    }
}

impl From<QueryConsensusStateHeightsResponse> for RawQueryConsensusStateHeightsResponse {
    fn from(response: QueryConsensusStateHeightsResponse) -> Self {
        Self {
            consensus_state_heights: response
                .consensus_state_heights
                .into_iter()
                .map(|height| height.into())
                .collect(),
            pagination: response.pagination.map(|pagination| pagination.into()),
        }
    }
}

pub struct QueryClientStatusResponse {
    status: Status,
}

impl QueryClientStatusResponse {
    pub fn new(status: Status) -> Self {
        Self { status }
    }
}

impl From<QueryClientStatusResponse> for RawQueryClientStatusResponse {
    fn from(response: QueryClientStatusResponse) -> Self {
        Self {
            status: response.status.to_string(),
        }
    }
}

pub struct QueryClientParamsResponse {
    allowed_clients: Vec<ClientId>,
}

impl QueryClientParamsResponse {
    pub fn new(allowed_clients: Vec<ClientId>) -> Self {
        Self { allowed_clients }
    }
}

impl From<QueryClientParamsResponse> for RawQueryClientParamsResponse {
    fn from(response: QueryClientParamsResponse) -> Self {
        Self {
            params: Some(RawParams {
                allowed_clients: response
                    .allowed_clients
                    .into_iter()
                    .map(|id| id.to_string())
                    .collect(),
            }),
        }
    }
}

pub struct QueryUpgradedClientStateResponse {
    upgraded_client_state: Any,
}

impl QueryUpgradedClientStateResponse {
    pub fn new(upgraded_client_state: Any) -> Self {
        Self {
            upgraded_client_state,
        }
    }
}

impl From<QueryUpgradedClientStateResponse> for RawQueryUpgradedClientStateResponse {
    fn from(response: QueryUpgradedClientStateResponse) -> Self {
        Self {
            upgraded_client_state: Some(response.upgraded_client_state),
        }
    }
}

pub struct QueryUpgradedConsensusStateResponse {
    upgraded_consensus_state: Any,
}

impl QueryUpgradedConsensusStateResponse {
    pub fn new(upgraded_consensus_state: Any) -> Self {
        Self {
            upgraded_consensus_state,
        }
    }
}

impl From<QueryUpgradedConsensusStateResponse> for RawQueryUpgradedConsensusStateResponse {
    fn from(response: QueryUpgradedConsensusStateResponse) -> Self {
        Self {
            upgraded_consensus_state: Some(response.upgraded_consensus_state),
        }
    }
}
