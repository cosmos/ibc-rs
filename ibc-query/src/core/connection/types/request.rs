//! Contains all the RPC method request domain types and their conversions to
//! and from the corresponding gRPC proto types for the connection module.

use ibc::core::client::types::Height;
use ibc::core::host::types::identifiers::{ClientId, ConnectionId};
use ibc::primitives::prelude::*;
use ibc_proto::ibc::core::connection::v1::{
    QueryClientConnectionsRequest as RawQueryClientConnectionsRequest,
    QueryConnectionClientStateRequest as RawQueryConnectionClientStateRequest,
    QueryConnectionConsensusStateRequest as RawQueryConnectionConsensusStateRequest,
    QueryConnectionParamsRequest as RawQueryConnectionParamsRequest,
    QueryConnectionRequest as RawQueryConnectionRequest,
    QueryConnectionsRequest as RawQueryConnectionsRequest,
};

use crate::error::QueryError;
use crate::types::PageRequest;

/// Defines the RPC method request type for querying a connection.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct QueryConnectionRequest {
    pub connection_id: ConnectionId,
    pub query_height: Option<Height>,
}

impl TryFrom<RawQueryConnectionRequest> for QueryConnectionRequest {
    type Error = QueryError;

    fn try_from(request: RawQueryConnectionRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            connection_id: request.connection_id.parse()?,
            query_height: None,
        })
    }
}

/// Defines the RPC method request type for querying connections.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct QueryConnectionsRequest {
    pub pagination: Option<PageRequest>,
}

impl From<RawQueryConnectionsRequest> for QueryConnectionsRequest {
    fn from(request: RawQueryConnectionsRequest) -> Self {
        Self {
            pagination: request.pagination.map(Into::into),
        }
    }
}

/// Defines the RPC method request type for querying connections associated with
/// a client.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct QueryClientConnectionsRequest {
    pub client_id: ClientId,
    pub query_height: Option<Height>,
}

impl TryFrom<RawQueryClientConnectionsRequest> for QueryClientConnectionsRequest {
    type Error = QueryError;

    fn try_from(request: RawQueryClientConnectionsRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            client_id: request.client_id.parse()?,
            query_height: None,
        })
    }
}

/// Defines the RPC method request type for querying the client state associated
/// with a connection.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct QueryConnectionClientStateRequest {
    pub connection_id: ConnectionId,
    pub query_height: Option<Height>,
}

impl TryFrom<RawQueryConnectionClientStateRequest> for QueryConnectionClientStateRequest {
    type Error = QueryError;

    fn try_from(request: RawQueryConnectionClientStateRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            connection_id: request.connection_id.parse()?,
            query_height: None,
        })
    }
}

/// Defines the RPC method request type for querying the consensus state
/// associated with a connection.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct QueryConnectionConsensusStateRequest {
    pub connection_id: ConnectionId,
    pub height: Height,
    pub query_height: Option<Height>,
}

impl TryFrom<RawQueryConnectionConsensusStateRequest> for QueryConnectionConsensusStateRequest {
    type Error = QueryError;

    fn try_from(request: RawQueryConnectionConsensusStateRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            connection_id: request.connection_id.parse()?,
            height: Height::new(request.revision_number, request.revision_height)?,
            query_height: None,
        })
    }
}

/// Defines the RPC method request type for querying the connection parameters.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct QueryConnectionParamsRequest {
    pub query_height: Option<Height>,
}

impl From<RawQueryConnectionParamsRequest> for QueryConnectionParamsRequest {
    fn from(_request: RawQueryConnectionParamsRequest) -> Self {
        Self { query_height: None }
    }
}
