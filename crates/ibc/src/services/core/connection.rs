//! [`ConnectionQueryService`](ConnectionQueryService) takes a generic `I` to store `ibc_context` that implements [`QueryContext`](QueryContext).
//! `I` must be a type where writes from one thread are readable from another.
//! This means using `Arc<Mutex<_>>` or `Arc<RwLock<_>>` in most cases.

use alloc::str::FromStr;
use std::boxed::Box;

use ibc_proto::google::protobuf::Any;
use ibc_proto::ibc::core::client::v1::IdentifiedClientState;
use ibc_proto::ibc::core::connection::v1::query_server::Query as ConnectionQuery;
use ibc_proto::ibc::core::connection::v1::{
    Params as ConnectionParams, QueryClientConnectionsRequest, QueryClientConnectionsResponse,
    QueryConnectionClientStateRequest, QueryConnectionClientStateResponse,
    QueryConnectionConsensusStateRequest, QueryConnectionConsensusStateResponse,
    QueryConnectionParamsRequest, QueryConnectionParamsResponse, QueryConnectionRequest,
    QueryConnectionResponse, QueryConnectionsRequest, QueryConnectionsResponse,
};
use tonic::{Request, Response, Status};

use crate::core::ics24_host::identifier::{ClientId, ConnectionId};
use crate::core::ics24_host::path::{
    ClientConnectionPath, ClientConsensusStatePath, ClientStatePath, ConnectionPath, Path,
};
use crate::core::ValidationContext;
use crate::prelude::*;
use crate::services::core::context::QueryContext;
use crate::Height;

// TODO(rano): currently the services don't support pagination, so we return all the results.

/// The generic `I` must be a type where writes from one thread are readable from another.
/// This means using `Arc<Mutex<_>>` or `Arc<RwLock<_>>` in most cases.
pub struct ConnectionQueryService<I>
where
    I: QueryContext + Send + Sync + 'static,
    <I as ValidationContext>::AnyClientState: Into<Any>,
    <I as ValidationContext>::AnyConsensusState: Into<Any>,
{
    ibc_context: I,
}

impl<I> ConnectionQueryService<I>
where
    I: QueryContext + Send + Sync + 'static,
    <I as ValidationContext>::AnyClientState: Into<Any>,
    <I as ValidationContext>::AnyConsensusState: Into<Any>,
{
    /// The parameter `ibc_context` must be a type where writes from one thread are readable from another.
    /// This means using `Arc<Mutex<_>>` or `Arc<RwLock<_>>` in most cases.
    pub fn new(ibc_context: I) -> Self {
        Self { ibc_context }
    }
}

#[tonic::async_trait]
impl<I> ConnectionQuery for ConnectionQueryService<I>
where
    I: QueryContext + Send + Sync + 'static,
    <I as ValidationContext>::AnyClientState: Into<Any>,
    <I as ValidationContext>::AnyConsensusState: Into<Any>,
{
    async fn connection(
        &self,
        request: Request<QueryConnectionRequest>,
    ) -> Result<Response<QueryConnectionResponse>, Status> {
        let request_ref = request.get_ref();

        let connection_id = ConnectionId::from_str(request_ref.connection_id.as_str())?;

        let connection_end = self.ibc_context.connection_end(&connection_id)?;

        let current_height = self.ibc_context.host_height()?;

        let proof = self
            .ibc_context
            .get_proof(
                current_height,
                &Path::Connection(ConnectionPath::new(&connection_id)),
            )
            .ok_or_else(|| {
                Status::not_found(format!(
                    "Proof not found for connection path {}",
                    connection_id.as_str()
                ))
            })?;

        Ok(Response::new(QueryConnectionResponse {
            connection: Some(connection_end.into()),
            proof,
            proof_height: Some(current_height.into()),
        }))
    }

    async fn connections(
        &self,
        _request: Request<QueryConnectionsRequest>,
    ) -> Result<Response<QueryConnectionsResponse>, Status> {
        let connections = self.ibc_context.connection_ends()?;

        Ok(Response::new(QueryConnectionsResponse {
            connections: connections.into_iter().map(Into::into).collect(),
            height: Some(self.ibc_context.host_height()?.into()),
            // no support for pagination yet
            pagination: None,
        }))
    }

    async fn client_connections(
        &self,
        request: Request<QueryClientConnectionsRequest>,
    ) -> Result<Response<QueryClientConnectionsResponse>, Status> {
        let request_ref = request.get_ref();

        let client_id = ClientId::from_str(request_ref.client_id.as_str())?;

        let connections = self.ibc_context.client_connection_ends(&client_id)?;

        let current_height = self.ibc_context.host_height()?;

        let proof: Vec<u8> = self
            .ibc_context
            .get_proof(
                current_height,
                &Path::ClientConnection(ClientConnectionPath::new(&client_id)),
            )
            .ok_or_else(|| {
                Status::not_found(format!(
                    "Proof not found for client connection path {}",
                    client_id.as_str()
                ))
            })?;

        Ok(Response::new(QueryClientConnectionsResponse {
            connection_paths: connections.into_iter().map(|x| x.as_str().into()).collect(),
            proof,
            proof_height: Some(current_height.into()),
        }))
    }

    async fn connection_client_state(
        &self,
        request: Request<QueryConnectionClientStateRequest>,
    ) -> Result<Response<QueryConnectionClientStateResponse>, Status> {
        let request_ref = request.get_ref();

        let connection_id = ConnectionId::from_str(request_ref.connection_id.as_str())?;

        let connection_end = self.ibc_context.connection_end(&connection_id)?;

        let client_state = self.ibc_context.client_state(connection_end.client_id())?;

        let current_height = self.ibc_context.host_height()?;

        let proof = self
            .ibc_context
            .get_proof(
                current_height,
                &Path::ClientState(ClientStatePath::new(connection_end.client_id())),
            )
            .ok_or_else(|| {
                Status::not_found(format!(
                    "Proof not found for client state path {}",
                    connection_end.client_id().as_str()
                ))
            })?;

        Ok(Response::new(QueryConnectionClientStateResponse {
            identified_client_state: Some(IdentifiedClientState {
                client_id: connection_end.client_id().as_str().into(),
                client_state: Some(client_state.into()),
            }),
            proof,
            proof_height: Some(current_height.into()),
        }))
    }

    async fn connection_consensus_state(
        &self,
        request: Request<QueryConnectionConsensusStateRequest>,
    ) -> Result<Response<QueryConnectionConsensusStateResponse>, Status> {
        let request_ref = request.get_ref();

        let connection_id = ConnectionId::from_str(request_ref.connection_id.as_str())?;

        let connection_end = self.ibc_context.connection_end(&connection_id)?;

        let height = Height::new(request_ref.revision_number, request_ref.revision_height)
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

        let consensus_path = ClientConsensusStatePath::new(connection_end.client_id(), &height);

        let consensus_state = self.ibc_context.consensus_state(&consensus_path)?;

        let current_height = self.ibc_context.host_height()?;

        let proof = self
            .ibc_context
            .get_proof(current_height, &Path::ClientConsensusState(consensus_path))
            .ok_or_else(|| {
                Status::not_found(format!(
                    "Proof not found for consensus state path {}",
                    connection_end.client_id().as_str()
                ))
            })?;

        Ok(Response::new(QueryConnectionConsensusStateResponse {
            consensus_state: Some(consensus_state.into()),
            client_id: connection_end.client_id().as_str().into(),
            proof,
            proof_height: Some(current_height.into()),
        }))
    }

    async fn connection_params(
        &self,
        _request: Request<QueryConnectionParamsRequest>,
    ) -> Result<Response<QueryConnectionParamsResponse>, Status> {
        Ok(Response::new(QueryConnectionParamsResponse {
            params: Some(ConnectionParams {
                max_expected_time_per_block: self
                    .ibc_context
                    .max_expected_time_per_block()
                    .as_secs(),
            }),
        }))
    }
}
