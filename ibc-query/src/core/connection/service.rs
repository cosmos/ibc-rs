//! [`ConnectionQueryService`](ConnectionQueryService) takes a generic `I` to
//! store `ibc_context` that implements [`QueryContext`](QueryContext). `I` must
//! be a type where writes from one thread are readable from another. This means
//! using `Arc<Mutex<_>>` or `Arc<RwLock<_>>` in most cases.

use ibc::core::host::ConsensusStateRef;
use ibc::core::primitives::prelude::*;
use ibc_proto::google::protobuf::Any;
use ibc_proto::ibc::core::connection::v1::query_server::Query as ConnectionQuery;
use ibc_proto::ibc::core::connection::v1::{
    QueryClientConnectionsRequest, QueryClientConnectionsResponse,
    QueryConnectionClientStateRequest, QueryConnectionClientStateResponse,
    QueryConnectionConsensusStateRequest, QueryConnectionConsensusStateResponse,
    QueryConnectionParamsRequest, QueryConnectionParamsResponse, QueryConnectionRequest,
    QueryConnectionResponse, QueryConnectionsRequest, QueryConnectionsResponse,
};
use tonic::{Request, Response, Status};

use super::{
    query_client_connections, query_connection, query_connection_client_state,
    query_connection_consensus_state, query_connection_params, query_connections,
};
use crate::core::context::QueryContext;
use crate::utils::{IntoDomain, IntoResponse, TryIntoDomain};

// TODO(rano): currently the services don't support pagination, so we return all the results.

/// The generic `I` must be a type where writes from one thread are readable
/// from another. This means using `Arc<Mutex<_>>` or `Arc<RwLock<_>>` in most
/// cases.
pub struct ConnectionQueryService<I>
where
    I: QueryContext + Send + Sync + 'static,
    ConsensusStateRef<I>: Into<Any>,
{
    ibc_context: I,
}

impl<I> ConnectionQueryService<I>
where
    I: QueryContext + Send + Sync + 'static,
    ConsensusStateRef<I>: Into<Any>,
{
    /// The parameter `ibc_context` must be a type where writes from one thread
    /// are readable from another. This means using `Arc<Mutex<_>>` or
    /// `Arc<RwLock<_>>` in most cases.
    pub fn new(ibc_context: I) -> Self {
        Self { ibc_context }
    }
}

#[tonic::async_trait]
impl<I> ConnectionQuery for ConnectionQueryService<I>
where
    I: QueryContext + Send + Sync + 'static,
    ConsensusStateRef<I>: Into<Any>,
{
    async fn connection(
        &self,
        request: Request<QueryConnectionRequest>,
    ) -> Result<Response<QueryConnectionResponse>, Status> {
        query_connection(&self.ibc_context, &request.try_into_domain()?)?.into_response()
    }

    async fn connections(
        &self,
        request: Request<QueryConnectionsRequest>,
    ) -> Result<Response<QueryConnectionsResponse>, Status> {
        query_connections(&self.ibc_context, &request.into_domain())?.into_response()
    }

    async fn client_connections(
        &self,
        request: Request<QueryClientConnectionsRequest>,
    ) -> Result<Response<QueryClientConnectionsResponse>, Status> {
        query_client_connections(&self.ibc_context, &request.try_into_domain()?)?.into_response()
    }

    async fn connection_client_state(
        &self,
        request: Request<QueryConnectionClientStateRequest>,
    ) -> Result<Response<QueryConnectionClientStateResponse>, Status> {
        query_connection_client_state(&self.ibc_context, &request.try_into_domain()?)?
            .into_response()
    }

    async fn connection_consensus_state(
        &self,
        request: Request<QueryConnectionConsensusStateRequest>,
    ) -> Result<Response<QueryConnectionConsensusStateResponse>, Status> {
        query_connection_consensus_state(&self.ibc_context, &request.try_into_domain()?)?
            .into_response()
    }

    async fn connection_params(
        &self,
        request: Request<QueryConnectionParamsRequest>,
    ) -> Result<Response<QueryConnectionParamsResponse>, Status> {
        query_connection_params(&self.ibc_context, &request.into_domain())?.into_response()
    }
}
