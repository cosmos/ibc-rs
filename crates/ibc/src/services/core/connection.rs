use ibc_proto::{
    google::protobuf::Any,
    ibc::core::{
        client::v1::IdentifiedClientState,
        connection::v1::{
            query_server::Query as ConnectionQuery, Params as ConnectionParams,
            QueryClientConnectionsRequest, QueryClientConnectionsResponse,
            QueryConnectionClientStateRequest, QueryConnectionClientStateResponse,
            QueryConnectionConsensusStateRequest, QueryConnectionConsensusStateResponse,
            QueryConnectionParamsRequest, QueryConnectionParamsResponse, QueryConnectionRequest,
            QueryConnectionResponse, QueryConnectionsRequest, QueryConnectionsResponse,
        },
    },
};

use crate::{
    core::{
        ics24_host::{
            identifier::{ClientId, ConnectionId},
            path::{
                ClientConnectionPath, ClientConsensusStatePath, ClientStatePath, ConnectionPath,
                Path,
            },
        },
        ValidationContext,
    },
    services::core::context::QueryContext,
    Height,
};

use crate::prelude::*;

use std::boxed::Box;
use tonic::{Request, Response, Status};
use tracing::trace;

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

        trace!("Got connection request: {:?}", request_ref);

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
        request: Request<QueryConnectionsRequest>,
    ) -> Result<Response<QueryConnectionsResponse>, Status> {
        let request_ref = request.get_ref();

        trace!("Got connections request: {:?}", request_ref);

        let connections = self.ibc_context.connection_ends()?;

        Ok(Response::new(QueryConnectionsResponse {
            connections: connections.into_iter().map(Into::into).collect(),
            pagination: None,
            height: Some(self.ibc_context.host_height()?.into()),
        }))
    }

    async fn client_connections(
        &self,
        request: Request<QueryClientConnectionsRequest>,
    ) -> Result<Response<QueryClientConnectionsResponse>, Status> {
        let request_ref = request.get_ref();

        trace!("Got client connections request: {:?}", request_ref);

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

        trace!("Got connection client state request: {:?}", request_ref);

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

        trace!("Got connection consensus state request: {:?}", request_ref);

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
        request: Request<QueryConnectionParamsRequest>,
    ) -> Result<Response<QueryConnectionParamsResponse>, Status> {
        let request_ref = request.get_ref();

        trace!("Got connection params request: {:?}", request_ref);

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
