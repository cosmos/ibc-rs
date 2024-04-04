//! Contains all the RPC method request domain types and their conversions to
//! and from the corresponding gRPC proto types for the client module.

use ibc::core::client::types::Height;
use ibc::core::host::types::identifiers::ClientId;
use ibc::primitives::prelude::*;
use ibc_proto::ibc::core::client::v1::{
    QueryClientParamsRequest as RawQueryClientParamsRequest,
    QueryClientStateRequest as RawQueryClientStateRequest,
    QueryClientStatesRequest as RawQueryClientStatesRequest,
    QueryClientStatusRequest as RawQueryClientStatusRequest,
    QueryConsensusStateHeightsRequest as RawQueryConsensusStateHeightsRequest,
    QueryConsensusStateRequest as RawQueryConsensusStateRequest,
    QueryConsensusStatesRequest as RawQueryConsensusStatesRequest,
    QueryUpgradedClientStateRequest as RawUpgradedClientStateRequest,
    QueryUpgradedConsensusStateRequest as RawUpgradedConsensusStateRequest,
};
use ibc_proto::Protobuf;

use crate::error::QueryError;
use crate::types::PageRequest;

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct QueryClientStateRequest {
    /// The client identifier.
    pub client_id: ClientId,
    /// The height at which to query the client state. If not provided, the
    /// latest height should be used.
    pub query_height: Option<Height>,
}

impl TryFrom<RawQueryClientStateRequest> for QueryClientStateRequest {
    type Error = QueryError;

    fn try_from(request: RawQueryClientStateRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            client_id: request.client_id.parse()?,
            query_height: None,
        })
    }
}

/// Defines the RPC method request type for querying all client states.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct QueryClientStatesRequest {
    pub pagination: Option<PageRequest>,
}

impl From<RawQueryClientStatesRequest> for QueryClientStatesRequest {
    fn from(request: RawQueryClientStatesRequest) -> Self {
        Self {
            pagination: request.pagination.map(Into::into),
        }
    }
}

/// Defines the RPC method request type for querying the consensus state of a
/// client.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct QueryConsensusStateRequest {
    /// The client identifier.
    pub client_id: ClientId,
    /// The consensus state height to be queried. If not provided, the latest
    /// height
    pub consensus_height: Option<Height>,
    /// The height at which to query the consensus state. If not provided, the
    /// latest height should be used.
    pub query_height: Option<Height>,
}

impl TryFrom<RawQueryConsensusStateRequest> for QueryConsensusStateRequest {
    type Error = QueryError;

    fn try_from(request: RawQueryConsensusStateRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            client_id: request.client_id.parse()?,
            consensus_height: (!request.latest_height)
                .then(|| Height::new(request.revision_number, request.revision_height))
                .transpose()?,
            query_height: None,
        })
    }
}

/// Defines the RPC method request type for querying all consensus states.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct QueryConsensusStatesRequest {
    pub client_id: ClientId,
    pub pagination: Option<PageRequest>,
}

impl Protobuf<RawQueryConsensusStatesRequest> for QueryConsensusStatesRequest {}

impl TryFrom<RawQueryConsensusStatesRequest> for QueryConsensusStatesRequest {
    type Error = QueryError;

    fn try_from(request: RawQueryConsensusStatesRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            client_id: request.client_id.parse()?,
            pagination: request.pagination.map(Into::into),
        })
    }
}

impl From<QueryConsensusStatesRequest> for RawQueryConsensusStatesRequest {
    fn from(request: QueryConsensusStatesRequest) -> Self {
        Self {
            client_id: request.client_id.to_string(),
            pagination: request.pagination.map(Into::into),
        }
    }
}

/// Defines the RPC method request type for querying the consensus state
/// heights.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct QueryConsensusStateHeightsRequest {
    pub client_id: ClientId,
    pub pagination: Option<PageRequest>,
}

impl Protobuf<RawQueryConsensusStateHeightsRequest> for QueryConsensusStateHeightsRequest {}

impl TryFrom<RawQueryConsensusStateHeightsRequest> for QueryConsensusStateHeightsRequest {
    type Error = QueryError;

    fn try_from(request: RawQueryConsensusStateHeightsRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            client_id: request.client_id.parse()?,
            pagination: request.pagination.map(Into::into),
        })
    }
}

impl From<QueryConsensusStateHeightsRequest> for RawQueryConsensusStateHeightsRequest {
    fn from(request: QueryConsensusStateHeightsRequest) -> Self {
        Self {
            client_id: request.client_id.to_string(),
            pagination: request.pagination.map(Into::into),
        }
    }
}

/// Defines the RPC method request type for querying the host consensus state.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct QueryHostConsensusStateRequest {
    pub query_height: Option<Height>,
}

/// Defines the RPC method request type for querying the status of a client.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct QueryClientStatusRequest {
    pub client_id: ClientId,
    pub query_height: Option<Height>,
}

impl TryFrom<RawQueryClientStatusRequest> for QueryClientStatusRequest {
    type Error = QueryError;

    fn try_from(request: RawQueryClientStatusRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            client_id: request.client_id.parse()?,
            query_height: None,
        })
    }
}

/// Defines the RPC method request type for querying the parameters of a client.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct QueryClientParamsRequest {
    pub query_height: Option<Height>,
}

impl From<RawQueryClientParamsRequest> for QueryClientParamsRequest {
    fn from(_request: RawQueryClientParamsRequest) -> Self {
        Self { query_height: None }
    }
}

/// Defines the RPC method request type for querying the upgraded client state.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct QueryUpgradedClientStateRequest {
    /// Height at which the chain is scheduled to halt for upgrade
    pub upgrade_height: Option<Height>,
    /// The height at which to query the upgraded client state. If not provided,
    /// the latest height should be used.
    pub query_height: Option<Height>,
}

impl From<RawUpgradedClientStateRequest> for QueryUpgradedClientStateRequest {
    fn from(_request: RawUpgradedClientStateRequest) -> Self {
        Self {
            upgrade_height: None,
            query_height: None,
        }
    }
}

/// Defines the RPC method request type for querying the upgraded consensus
/// state.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct QueryUpgradedConsensusStateRequest {
    /// Height at which the chain is scheduled to halt for upgrade.
    pub upgrade_height: Option<Height>,
    /// The height at which to query the upgraded consensus state. If not
    /// provided, the latest height should be used.
    pub query_height: Option<Height>,
}

impl From<RawUpgradedConsensusStateRequest> for QueryUpgradedConsensusStateRequest {
    fn from(_request: RawUpgradedConsensusStateRequest) -> Self {
        Self {
            upgrade_height: None,
            query_height: None,
        }
    }
}
