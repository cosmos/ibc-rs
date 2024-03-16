//! Contains all the RPC method request domain types and their conversions to
//! and from the corresponding gRPC proto types for the connection module.

use alloc::string::ToString;

use ibc::core::client::types::Height;
use ibc::core::host::types::identifiers::{ClientId, ConnectionId};
use ibc_proto::ibc::core::channel::v1::QueryConnectionChannelsRequest as RawQueryConnectionChannelsRequest;
use ibc_proto::ibc::core::connection::v1::{
    QueryClientConnectionsRequest as RawQueryClientConnectionsRequest,
    QueryConnectionClientStateRequest as RawQueryConnectionClientStateRequest,
    QueryConnectionConsensusStateRequest as RawQueryConnectionConsensusStateRequest,
    QueryConnectionParamsRequest as RawQueryConnectionParamsRequest,
    QueryConnectionRequest as RawQueryConnectionRequest,
    QueryConnectionsRequest as RawQueryConnectionsRequest,
};
use serde::{Deserialize, Serialize};

use crate::error::QueryError;
use crate::types::PageRequest;

/// Defines the RPC method request type for querying connections.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QueryConnectionsRequest {
    pub pagination: Option<PageRequest>,
}

impl From<RawQueryConnectionsRequest> for QueryConnectionsRequest {
    fn from(request: RawQueryConnectionsRequest) -> Self {
        Self {
            pagination: request.pagination.map(|pagination| pagination.into()),
        }
    }
}

/// Defines the RPC method request type for querying connections associated with
/// a client.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QueryClientConnectionsRequest {
    pub client_id: ClientId,
}

impl TryFrom<RawQueryClientConnectionsRequest> for QueryClientConnectionsRequest {
    type Error = QueryError;

    fn try_from(request: RawQueryClientConnectionsRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            client_id: request.client_id.parse()?,
        })
    }
}

/// Defines the RPC method request type for querying a connection.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QueryConnectionRequest {
    pub connection_id: ConnectionId,
    pub height: Option<Height>,
}

impl TryFrom<RawQueryConnectionRequest> for QueryConnectionRequest {
    type Error = QueryError;

    fn try_from(request: RawQueryConnectionRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            connection_id: request.connection_id.parse()?,
            height: None,
        })
    }
}

/// Defines the RPC method request type for querying the channels associated
/// with a connection.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QueryConnectionChannelsRequest {
    pub connection_id: ConnectionId,
    pub pagination: Option<PageRequest>,
}

impl TryFrom<RawQueryConnectionChannelsRequest> for QueryConnectionChannelsRequest {
    type Error = QueryError;

    fn try_from(request: RawQueryConnectionChannelsRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            connection_id: request.connection.parse()?,
            pagination: request.pagination.map(|pagination| pagination.into()),
        })
    }
}

impl From<QueryConnectionChannelsRequest> for RawQueryConnectionChannelsRequest {
    fn from(request: QueryConnectionChannelsRequest) -> Self {
        RawQueryConnectionChannelsRequest {
            connection: request.connection_id.to_string(),
            pagination: request.pagination.map(|pagination| pagination.into()),
        }
    }
}

/// Defines the RPC method request type for querying the client state associated
/// with a connection.
#[derive(Clone, Debug, Serialize, Deserialize)]
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
#[derive(Clone, Debug, Serialize, Deserialize)]
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
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QueryConnectionParamsRequest {
    pub query_height: Option<Height>,
}

impl From<RawQueryConnectionParamsRequest> for QueryConnectionParamsRequest {
    fn from(_request: RawQueryConnectionParamsRequest) -> Self {
        Self { query_height: None }
    }
}
