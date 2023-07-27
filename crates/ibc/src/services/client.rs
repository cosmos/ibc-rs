use ibc_proto::{
    google::protobuf::Any,
    ibc::core::client::v1::{
        query_server::Query as ClientQuery, QueryClientParamsRequest, QueryClientParamsResponse,
        QueryClientStateRequest, QueryClientStateResponse, QueryClientStatesRequest,
        QueryClientStatesResponse, QueryClientStatusRequest, QueryClientStatusResponse,
        QueryConsensusStateHeightsRequest, QueryConsensusStateHeightsResponse,
        QueryConsensusStateRequest, QueryConsensusStateResponse, QueryConsensusStatesRequest,
        QueryConsensusStatesResponse, QueryUpgradedClientStateRequest,
        QueryUpgradedClientStateResponse, QueryUpgradedConsensusStateRequest,
        QueryUpgradedConsensusStateResponse,
    },
};

use crate::{
    core::{
        ics24_host::{identifier::ClientId, path::ClientConsensusStatePath},
        ValidationContext,
    },
    Height,
};

use core::str::FromStr;
use std::boxed::Box;
use tonic::{Request, Response, Status};
use tracing::trace;

pub struct ClientQueryServer<T> {
    context: T,
}

impl<T> ClientQueryServer<T> {
    pub fn new(context: T) -> Self {
        Self { context }
    }
}

#[tonic::async_trait]
impl<T> ClientQuery for ClientQueryServer<T>
where
    T: ValidationContext + Send + Sync + 'static,
    <T as ValidationContext>::AnyClientState: Into<Any>,
    <T as ValidationContext>::AnyConsensusState: Into<Any>,
{
    async fn client_state(
        &self,
        request: Request<QueryClientStateRequest>,
    ) -> Result<Response<QueryClientStateResponse>, Status> {
        trace!("Got client state request: {:?}", request);

        let request_ref = request.get_ref();

        let client_id = ClientId::from_str(request_ref.client_id.as_str()).map_err(|_| {
            Status::invalid_argument(std::format!("Invalid client id: {}", request_ref.client_id))
        })?;
        let client_state = self.context.client_state(&client_id).map_err(|_| {
            Status::not_found(std::format!(
                "Client state not found for client {}",
                client_id
            ))
        })?;

        Ok(Response::new(QueryClientStateResponse {
            client_state: Some(client_state.into()),
            proof: Default::default(),
            proof_height: None,
        }))
    }

    async fn client_states(
        &self,
        request: Request<QueryClientStatesRequest>,
    ) -> Result<Response<QueryClientStatesResponse>, Status> {
        trace!("Got client states request: {:?}", request);
        todo!()
    }

    async fn consensus_state(
        &self,
        request: Request<QueryConsensusStateRequest>,
    ) -> Result<Response<QueryConsensusStateResponse>, Status> {
        trace!("Got consensus state request: {:?}", request);

        let request_ref = request.get_ref();

        let client_id = ClientId::from_str(request_ref.client_id.as_str()).map_err(|_| {
            Status::invalid_argument(std::format!("Invalid client id: {}", request_ref.client_id))
        })?;

        let height = Height::new(request_ref.revision_number, request_ref.revision_height)
            .map_err(|_| {
                Status::invalid_argument(std::format!(
                    "Invalid height: {}-{}",
                    request_ref.revision_number,
                    request_ref.revision_height
                ))
            })?;

        let consensus_state = self
            .context
            .consensus_state(&ClientConsensusStatePath::new(&client_id, &height))
            .map_err(|_| {
                Status::not_found(std::format!(
                    "Consensus state not found for client {} at height {}",
                    client_id,
                    height
                ))
            })?;

        Ok(Response::new(QueryConsensusStateResponse {
            consensus_state: Some(consensus_state.into()),
            proof: Default::default(),
            proof_height: None,
        }))
    }

    async fn consensus_states(
        &self,
        request: Request<QueryConsensusStatesRequest>,
    ) -> Result<Response<QueryConsensusStatesResponse>, Status> {
        trace!("Got consensus states request: {:?}", request);
        todo!();
    }

    async fn consensus_state_heights(
        &self,
        _request: Request<QueryConsensusStateHeightsRequest>,
    ) -> Result<Response<QueryConsensusStateHeightsResponse>, Status> {
        todo!()
    }

    async fn client_status(
        &self,
        _request: Request<QueryClientStatusRequest>,
    ) -> Result<Response<QueryClientStatusResponse>, Status> {
        unimplemented!()
    }

    async fn client_params(
        &self,
        _request: Request<QueryClientParamsRequest>,
    ) -> Result<Response<QueryClientParamsResponse>, Status> {
        unimplemented!()
    }

    async fn upgraded_client_state(
        &self,
        _request: Request<QueryUpgradedClientStateRequest>,
    ) -> Result<Response<QueryUpgradedClientStateResponse>, Status> {
        unimplemented!()
    }

    async fn upgraded_consensus_state(
        &self,
        _request: Request<QueryUpgradedConsensusStateRequest>,
    ) -> Result<Response<QueryUpgradedConsensusStateResponse>, Status> {
        unimplemented!()
    }
}
