//! Contains all the RPC method response domain types and their conversions to
//! and from the corresponding gRPC proto types for the connection module.

use ibc::core::client::types::Height;
use ibc::core::connection::types::{ConnectionEnd, IdentifiedConnectionEnd};
use ibc::core::host::types::identifiers::{ClientId, ConnectionId};
use ibc::core::primitives::proto::Any;
use ibc::primitives::prelude::*;
use ibc::primitives::proto::Protobuf;
use ibc_proto::ibc::core::connection::v1::{
    Params as RawParams, QueryClientConnectionsResponse as RawQueryClientConnectionsResponse,
    QueryConnectionClientStateResponse as RawQueryConnectionClientStateResponse,
    QueryConnectionConsensusStateResponse as RawQueryConnectionConsensusStateResponse,
    QueryConnectionParamsResponse as RawQueryConnectionParamsResponse,
    QueryConnectionResponse as RawQueryConnectionResponse,
    QueryConnectionsResponse as RawQueryConnectionsResponse,
};

use crate::core::client::IdentifiedClientState;
use crate::error::QueryError;
use crate::types::{PageResponse, Proof};

/// Defines the RPC method response type when querying a connection.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct QueryConnectionResponse {
    pub conn_end: ConnectionEnd,
    pub proof: Proof,
    pub proof_height: Height,
}

impl QueryConnectionResponse {
    pub fn new(connection: ConnectionEnd, proof: Proof, proof_height: Height) -> Self {
        Self {
            conn_end: connection,
            proof,
            proof_height,
        }
    }
}

impl Protobuf<RawQueryConnectionResponse> for QueryConnectionResponse {}

impl TryFrom<RawQueryConnectionResponse> for QueryConnectionResponse {
    type Error = QueryError;

    fn try_from(value: RawQueryConnectionResponse) -> Result<Self, Self::Error> {
        Ok(Self {
            conn_end: value
                .connection
                .ok_or_else(|| QueryError::missing_field("connection"))?
                .try_into()?,
            proof: value.proof,
            proof_height: value
                .proof_height
                .ok_or_else(|| QueryError::missing_field("proof_height"))?
                .try_into()?,
        })
    }
}

impl From<QueryConnectionResponse> for RawQueryConnectionResponse {
    fn from(response: QueryConnectionResponse) -> Self {
        Self {
            connection: Some(response.conn_end.into()),
            proof: response.proof,
            proof_height: Some(response.proof_height.into()),
        }
    }
}

/// Defines the RPC method response type when querying all the existing connection ends.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct QueryConnectionsResponse {
    pub connections: Vec<IdentifiedConnectionEnd>,
    pub query_height: Height,
    pub pagination: Option<PageResponse>,
}

impl QueryConnectionsResponse {
    pub fn new(
        connections: Vec<IdentifiedConnectionEnd>,
        query_height: Height,
        pagination: Option<PageResponse>,
    ) -> Self {
        Self {
            connections,
            query_height,
            pagination,
        }
    }
}

impl Protobuf<RawQueryConnectionsResponse> for QueryConnectionsResponse {}

impl TryFrom<RawQueryConnectionsResponse> for QueryConnectionsResponse {
    type Error = QueryError;

    fn try_from(value: RawQueryConnectionsResponse) -> Result<Self, Self::Error> {
        Ok(Self {
            connections: value
                .connections
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
            query_height: value
                .height
                .ok_or_else(|| QueryError::missing_field("height"))?
                .try_into()?,
            pagination: value.pagination.map(Into::into),
        })
    }
}

impl From<QueryConnectionsResponse> for RawQueryConnectionsResponse {
    fn from(response: QueryConnectionsResponse) -> Self {
        Self {
            connections: response.connections.into_iter().map(Into::into).collect(),
            height: Some(response.query_height.into()),
            pagination: response.pagination.map(Into::into),
        }
    }
}

/// Defines the RPC method response type when querying client state associated
/// with a connection.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct QueryConnectionClientStateResponse {
    pub identified_client_state: IdentifiedClientState,
    pub proof: Proof,
    pub proof_height: Height,
}

impl QueryConnectionClientStateResponse {
    pub fn new(
        identified_client_state: IdentifiedClientState,
        proof: Proof,
        proof_height: Height,
    ) -> Self {
        Self {
            identified_client_state,
            proof,
            proof_height,
        }
    }
}

impl Protobuf<RawQueryConnectionClientStateResponse> for QueryConnectionClientStateResponse {}

impl TryFrom<RawQueryConnectionClientStateResponse> for QueryConnectionClientStateResponse {
    type Error = QueryError;

    fn try_from(value: RawQueryConnectionClientStateResponse) -> Result<Self, Self::Error> {
        Ok(Self {
            identified_client_state: value
                .identified_client_state
                .ok_or_else(|| QueryError::missing_field("identified_client_state"))?
                .try_into()?,
            proof: value.proof,
            proof_height: value
                .proof_height
                .ok_or_else(|| QueryError::missing_field("proof_height"))?
                .try_into()?,
        })
    }
}

impl From<QueryConnectionClientStateResponse> for RawQueryConnectionClientStateResponse {
    fn from(response: QueryConnectionClientStateResponse) -> Self {
        Self {
            identified_client_state: Some(response.identified_client_state.into()),
            proof: response.proof,
            proof_height: Some(response.proof_height.into()),
        }
    }
}

/// Defines the RPC method response type when querying all the existing
/// connection ends for a given client.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct QueryClientConnectionsResponse {
    pub connection_paths: Vec<ConnectionId>,
    pub proof: Proof,
    pub proof_height: Height,
}

impl QueryClientConnectionsResponse {
    pub fn new(connection_paths: Vec<ConnectionId>, proof: Proof, proof_height: Height) -> Self {
        Self {
            connection_paths,
            proof,
            proof_height,
        }
    }
}

impl Protobuf<RawQueryClientConnectionsResponse> for QueryClientConnectionsResponse {}

impl TryFrom<RawQueryClientConnectionsResponse> for QueryClientConnectionsResponse {
    type Error = QueryError;

    fn try_from(value: RawQueryClientConnectionsResponse) -> Result<Self, Self::Error> {
        Ok(Self {
            connection_paths: value
                .connection_paths
                .into_iter()
                .map(|id| id.parse())
                .collect::<Result<_, _>>()?,
            proof: value.proof,
            proof_height: value
                .proof_height
                .ok_or_else(|| QueryError::missing_field("proof_height"))?
                .try_into()?,
        })
    }
}

impl From<QueryClientConnectionsResponse> for RawQueryClientConnectionsResponse {
    fn from(response: QueryClientConnectionsResponse) -> Self {
        Self {
            connection_paths: response
                .connection_paths
                .into_iter()
                .map(|id| id.to_string())
                .collect(),
            proof: response.proof,
            proof_height: Some(response.proof_height.into()),
        }
    }
}

/// Defines the RPC method response type when querying the consensus state for a
/// connection.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct QueryConnectionConsensusStateResponse {
    pub consensus_state: Any,
    pub client_id: ClientId,
    pub proof: Proof,
    pub proof_height: Height,
}

impl QueryConnectionConsensusStateResponse {
    pub fn new(
        consensus_state: Any,
        client_id: ClientId,
        proof: Proof,
        proof_height: Height,
    ) -> Self {
        Self {
            consensus_state,
            client_id,
            proof,
            proof_height,
        }
    }
}

impl Protobuf<RawQueryConnectionConsensusStateResponse> for QueryConnectionConsensusStateResponse {}

impl TryFrom<RawQueryConnectionConsensusStateResponse> for QueryConnectionConsensusStateResponse {
    type Error = QueryError;

    fn try_from(value: RawQueryConnectionConsensusStateResponse) -> Result<Self, Self::Error> {
        Ok(Self {
            consensus_state: value
                .consensus_state
                .ok_or_else(|| QueryError::missing_field("consensus_state"))?,
            client_id: value.client_id.parse()?,
            proof: value.proof,
            proof_height: value
                .proof_height
                .ok_or_else(|| QueryError::missing_field("proof_height"))?
                .try_into()?,
        })
    }
}

impl From<QueryConnectionConsensusStateResponse> for RawQueryConnectionConsensusStateResponse {
    fn from(response: QueryConnectionConsensusStateResponse) -> Self {
        Self {
            consensus_state: Some(response.consensus_state),
            client_id: response.client_id.to_string(),
            proof: response.proof,
            proof_height: Some(response.proof_height.into()),
        }
    }
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct QueryConnectionParamsResponse {
    pub max_expected_time_per_block: u64,
}

impl QueryConnectionParamsResponse {
    pub fn new(max_expected_time_per_block: u64) -> Self {
        Self {
            max_expected_time_per_block,
        }
    }
}

impl Protobuf<RawQueryConnectionParamsResponse> for QueryConnectionParamsResponse {}

impl TryFrom<RawQueryConnectionParamsResponse> for QueryConnectionParamsResponse {
    type Error = QueryError;

    fn try_from(value: RawQueryConnectionParamsResponse) -> Result<Self, Self::Error> {
        Ok(Self {
            max_expected_time_per_block: value
                .params
                .ok_or_else(|| QueryError::missing_field("params"))?
                .max_expected_time_per_block,
        })
    }
}

impl From<QueryConnectionParamsResponse> for RawQueryConnectionParamsResponse {
    fn from(response: QueryConnectionParamsResponse) -> Self {
        Self {
            params: Some(RawParams {
                max_expected_time_per_block: response.max_expected_time_per_block,
            }),
        }
    }
}
