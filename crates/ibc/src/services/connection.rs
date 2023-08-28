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
            path::ClientConsensusStatePath,
        },
        QueryContext, ValidationContext,
    },
    Height,
};

use core::str::FromStr;
use std::boxed::Box;
use tonic::{Request, Response, Status};
use tracing::trace;

pub struct ConnectionQueryServer<T> {
    pub context: T,
}

impl<T> ConnectionQueryServer<T> {
    pub fn new(context: T) -> Self {
        Self { context }
    }
}

#[tonic::async_trait]
impl<T> ConnectionQuery for ConnectionQueryServer<T>
where
    T: QueryContext + Send + Sync + 'static,
    <T as ValidationContext>::AnyClientState: Into<Any>,
    <T as ValidationContext>::AnyConsensusState: Into<Any>,
{
    async fn connection(
        &self,
        request: Request<QueryConnectionRequest>,
    ) -> Result<Response<QueryConnectionResponse>, Status> {
        let request_ref = request.get_ref();

        let connection_id =
            ConnectionId::from_str(request_ref.connection_id.as_str()).map_err(|_| {
                Status::invalid_argument(std::format!(
                    "Invalid connection id: {}",
                    request_ref.connection_id
                ))
            })?;

        let connection_end = self.context.connection_end(&connection_id).map_err(|_| {
            Status::not_found(std::format!(
                "Connection end not found for connection {}",
                connection_id
            ))
        })?;

        Ok(Response::new(QueryConnectionResponse {
            connection: Some(connection_end.into()),
            proof: Default::default(),
            proof_height: None,
        }))
    }

    async fn connections(
        &self,
        request: Request<QueryConnectionsRequest>,
    ) -> Result<Response<QueryConnectionsResponse>, Status> {
        trace!("Got connections request: {:?}", request);

        let connections = self
            .context
            .connection_ends()
            .map_err(|_| Status::not_found("Connections not found"))?;

        Ok(Response::new(QueryConnectionsResponse {
            connections: connections.into_iter().map(Into::into).collect(),
            pagination: None,
            height: Some(
                self.context
                    .host_height()
                    .map_err(|_| Status::not_found("Host chain height not found"))?
                    .into(),
            ),
        }))
    }

    async fn client_connections(
        &self,
        request: Request<QueryClientConnectionsRequest>,
    ) -> Result<Response<QueryClientConnectionsResponse>, Status> {
        trace!("Got client connections request: {:?}", request);

        let request_ref = request.get_ref();

        let client_id = ClientId::from_str(request_ref.client_id.as_str()).map_err(|_| {
            Status::invalid_argument(std::format!("Invalid client id: {}", request_ref.client_id))
        })?;

        let connections = self
            .context
            .client_connection_ends(&client_id)
            .map_err(|_| Status::not_found("Connections not found"))?;

        Ok(Response::new(QueryClientConnectionsResponse {
            connection_paths: connections.into_iter().map(|x| x.as_str().into()).collect(),
            proof: Default::default(),
            proof_height: None,
        }))
    }

    async fn connection_client_state(
        &self,
        request: Request<QueryConnectionClientStateRequest>,
    ) -> Result<Response<QueryConnectionClientStateResponse>, Status> {
        let request_ref = request.get_ref();

        let connection_id =
            ConnectionId::from_str(request_ref.connection_id.as_str()).map_err(|_| {
                Status::invalid_argument(std::format!(
                    "Invalid connection id: {}",
                    request_ref.connection_id
                ))
            })?;

        let connection_end = self.context.connection_end(&connection_id).map_err(|_| {
            Status::not_found(std::format!(
                "Connection end not found for connection {}",
                connection_id
            ))
        })?;

        let client_state = self
            .context
            .client_state(connection_end.client_id())
            .map_err(|_| {
                Status::not_found(std::format!(
                    "Client state not found for connection {}",
                    connection_id
                ))
            })?;

        Ok(Response::new(QueryConnectionClientStateResponse {
            identified_client_state: Some(IdentifiedClientState {
                client_id: connection_end.client_id().as_str().into(),
                client_state: Some(client_state.into()),
            }),
            proof: Default::default(),
            proof_height: None,
        }))
    }

    async fn connection_consensus_state(
        &self,
        request: Request<QueryConnectionConsensusStateRequest>,
    ) -> Result<Response<QueryConnectionConsensusStateResponse>, Status> {
        let request_ref = request.get_ref();

        let connection_id =
            ConnectionId::from_str(request_ref.connection_id.as_str()).map_err(|_| {
                Status::invalid_argument(std::format!(
                    "Invalid connection id: {}",
                    request_ref.connection_id
                ))
            })?;

        let connection_end = self.context.connection_end(&connection_id).map_err(|_| {
            Status::not_found(std::format!(
                "Connection end not found for connection {}",
                connection_id
            ))
        })?;

        let consensus_path = ClientConsensusStatePath::new(
            connection_end.client_id(),
            &Height::new(request_ref.revision_number, request_ref.revision_height).map_err(
                |_| {
                    Status::invalid_argument(std::format!(
                        "Invalid height: {}-{}",
                        request_ref.revision_number,
                        request_ref.revision_height
                    ))
                },
            )?,
        );

        let consensus_state = self.context.consensus_state(&consensus_path).map_err(|_| {
            Status::not_found(std::format!(
                "Consensus state not found for connection {}",
                connection_id
            ))
        })?;

        Ok(Response::new(QueryConnectionConsensusStateResponse {
            consensus_state: Some(consensus_state.into()),
            client_id: connection_end.client_id().as_str().into(),
            proof: Default::default(),
            proof_height: None,
        }))
    }

    async fn connection_params(
        &self,
        request: Request<QueryConnectionParamsRequest>,
    ) -> Result<Response<QueryConnectionParamsResponse>, Status> {
        trace!("Got connection params request: {:?}", request);

        Ok(Response::new(QueryConnectionParamsResponse {
            params: Some(ConnectionParams {
                max_expected_time_per_block: self.context.max_expected_time_per_block().as_secs(),
            }),
        }))
    }
}
