use ibc_proto::{
    google::protobuf::Any,
    ibc::core::{
        client::v1::IdentifiedClientState,
        connection::v1::{
            query_server::Query as ConnectionQuery, QueryClientConnectionsRequest,
            QueryClientConnectionsResponse, QueryConnectionClientStateRequest,
            QueryConnectionClientStateResponse, QueryConnectionConsensusStateRequest,
            QueryConnectionConsensusStateResponse, QueryConnectionParamsRequest,
            QueryConnectionParamsResponse, QueryConnectionRequest, QueryConnectionResponse,
            QueryConnectionsRequest, QueryConnectionsResponse,
        },
    },
};

use crate::{
    core::{
        ics24_host::{identifier::ConnectionId, path::ClientConsensusStatePath},
        ValidationContext,
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
    T: ValidationContext + Send + Sync + 'static,
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
        _request: Request<QueryConnectionsRequest>,
    ) -> Result<Response<QueryConnectionsResponse>, Status> {
        todo!()
    }

    async fn client_connections(
        &self,
        request: Request<QueryClientConnectionsRequest>,
    ) -> Result<Response<QueryClientConnectionsResponse>, Status> {
        trace!("Got client connections request: {:?}", request);
        todo!()
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
        _request: Request<QueryConnectionParamsRequest>,
    ) -> Result<Response<QueryConnectionParamsResponse>, Status> {
        todo!()
    }
}
