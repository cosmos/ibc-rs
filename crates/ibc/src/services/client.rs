use ibc_proto::{
    google::protobuf::Any,
    ibc::core::client::v1::{
        query_server::Query as ClientQuery, ConsensusStateWithHeight, IdentifiedClientState,
        Params as ClientParams, QueryClientParamsRequest, QueryClientParamsResponse,
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
        ics24_host::{
            identifier::ClientId,
            path::{ClientConsensusStatePath, ClientStatePath, Path, UpgradeClientPath},
        },
        QueryContext, ValidationContext,
    },
    hosts::tendermint::upgrade_proposal::UpgradeValidationContext,
    Height,
};

use core::str::FromStr;
use std::boxed::Box;
use tonic::{Request, Response, Status};
use tracing::trace;

pub struct ClientQueryService<I, U>
where
    I: QueryContext + Send + Sync + 'static,
    U: UpgradeValidationContext + Send + Sync + 'static,
    <I as ValidationContext>::AnyClientState: Into<Any>,
    <I as ValidationContext>::AnyConsensusState: Into<Any>,
    <U as UpgradeValidationContext>::AnyClientState: Into<Any>,
    <U as UpgradeValidationContext>::AnyConsensusState: Into<Any>,
{
    ibc_context: I,
    upgrade_context: U,
}

impl<I, U> ClientQueryService<I, U>
where
    I: QueryContext + Send + Sync + 'static,
    U: UpgradeValidationContext + Send + Sync + 'static,
    <I as ValidationContext>::AnyClientState: Into<Any>,
    <I as ValidationContext>::AnyConsensusState: Into<Any>,
    <U as UpgradeValidationContext>::AnyClientState: Into<Any>,
    <U as UpgradeValidationContext>::AnyConsensusState: Into<Any>,
{
    pub fn new(ibc_context: I, upgrade_context: U) -> Self {
        Self {
            ibc_context,
            upgrade_context,
        }
    }
}

#[tonic::async_trait]
impl<I, U> ClientQuery for ClientQueryService<I, U>
where
    I: QueryContext + Send + Sync + 'static,
    U: UpgradeValidationContext + Send + Sync + 'static,
    <I as ValidationContext>::AnyClientState: Into<Any>,
    <I as ValidationContext>::AnyConsensusState: Into<Any>,
    <U as UpgradeValidationContext>::AnyClientState: Into<Any>,
    <U as UpgradeValidationContext>::AnyConsensusState: Into<Any>,
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
        let client_state = self.ibc_context.client_state(&client_id).map_err(|_| {
            Status::not_found(std::format!(
                "Client state not found for client {}",
                client_id
            ))
        })?;

        let current_height = self
            .ibc_context
            .host_height()
            .map_err(|_| Status::not_found("Current height not found"))?;

        let proof = self
            .ibc_context
            .get_proof(
                current_height,
                &Path::ClientState(ClientStatePath::new(&client_id)),
            )
            .ok_or_else(|| {
                Status::not_found(std::format!(
                    "Client state not found for client {} at height {}",
                    client_id,
                    current_height
                ))
            })?;

        Ok(Response::new(QueryClientStateResponse {
            client_state: Some(client_state.into()),
            proof,
            proof_height: Some(current_height.into()),
        }))
    }

    async fn client_states(
        &self,
        request: Request<QueryClientStatesRequest>,
    ) -> Result<Response<QueryClientStatesResponse>, Status> {
        trace!("Got client states request: {:?}", request);

        let client_states = self
            .ibc_context
            .client_states()
            .map_err(|_| Status::not_found("Client states not found"))?;

        Ok(Response::new(QueryClientStatesResponse {
            client_states: client_states
                .into_iter()
                .map(|(id, state)| IdentifiedClientState {
                    client_id: id.into(),
                    client_state: Some(state.into()),
                })
                .collect(),
            pagination: None,
        }))
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
            .ibc_context
            .consensus_state(&ClientConsensusStatePath::new(&client_id, &height))
            .map_err(|_| {
                Status::not_found(std::format!(
                    "Consensus state not found for client {} at height {}",
                    client_id,
                    height
                ))
            })?;

        let current_height = self
            .ibc_context
            .host_height()
            .map_err(|_| Status::not_found("Current height not found"))?;

        let proof = self
            .ibc_context
            .get_proof(
                current_height,
                &Path::ClientConsensusState(ClientConsensusStatePath::new(&client_id, &height)),
            )
            .ok_or_else(|| {
                Status::not_found(std::format!(
                    "Consensus state not found for client {} at height {}",
                    client_id,
                    height
                ))
            })?;

        Ok(Response::new(QueryConsensusStateResponse {
            consensus_state: Some(consensus_state.into()),
            proof,
            proof_height: Some(current_height.into()),
        }))
    }

    async fn consensus_states(
        &self,
        request: Request<QueryConsensusStatesRequest>,
    ) -> Result<Response<QueryConsensusStatesResponse>, Status> {
        trace!("Got consensus states request: {:?}", request);

        let request_ref = request.get_ref();

        let client_id = ClientId::from_str(request_ref.client_id.as_str()).map_err(|_| {
            Status::invalid_argument(std::format!("Invalid client id: {}", request_ref.client_id))
        })?;

        let consensus_states = self.ibc_context.consensus_states(&client_id).map_err(|_| {
            Status::not_found(std::format!(
                "Consensus states not found for client {}",
                client_id
            ))
        })?;

        Ok(Response::new(QueryConsensusStatesResponse {
            consensus_states: consensus_states
                .into_iter()
                .map(|(height, state)| ConsensusStateWithHeight {
                    height: Some(height.into()),
                    consensus_state: Some(state.into()),
                })
                .collect(),
            pagination: None,
        }))
    }

    async fn consensus_state_heights(
        &self,
        request: Request<QueryConsensusStateHeightsRequest>,
    ) -> Result<Response<QueryConsensusStateHeightsResponse>, Status> {
        trace!("Got consensus state heights request: {:?}", request);

        let request_ref = request.get_ref();

        let client_id = ClientId::from_str(request_ref.client_id.as_str()).map_err(|_| {
            Status::invalid_argument(std::format!("Invalid client id: {}", request_ref.client_id))
        })?;

        let consensus_state_heights = self
            .ibc_context
            .consensus_state_heights(&client_id)
            .map_err(|_| {
                Status::not_found(std::format!(
                    "Consensus state heights not found for client {}",
                    client_id
                ))
            })?;

        Ok(Response::new(QueryConsensusStateHeightsResponse {
            consensus_state_heights: consensus_state_heights
                .into_iter()
                .map(|height| height.into())
                .collect(),
            pagination: None,
        }))
    }

    async fn client_status(
        &self,
        request: Request<QueryClientStatusRequest>,
    ) -> Result<Response<QueryClientStatusResponse>, Status> {
        trace!("Got client status request: {:?}", request);

        let request_ref = request.get_ref();

        let client_id = ClientId::from_str(request_ref.client_id.as_str()).map_err(|_| {
            Status::invalid_argument(std::format!("Invalid client id: {}", request_ref.client_id))
        })?;

        let client_status = self.ibc_context.client_status(&client_id).map_err(|_| {
            Status::not_found(std::format!(
                "Client status not found for client {}",
                client_id
            ))
        })?;

        Ok(Response::new(QueryClientStatusResponse {
            status: std::format!("{}", client_status),
        }))
    }

    async fn client_params(
        &self,
        request: Request<QueryClientParamsRequest>,
    ) -> Result<Response<QueryClientParamsResponse>, Status> {
        trace!("Got client params request: {:?}", request);

        Ok(Response::new(QueryClientParamsResponse {
            params: Some(ClientParams {
                allowed_clients: self
                    .ibc_context
                    .allowed_clients()
                    .into_iter()
                    .map(|x| x.as_str().into())
                    .collect(),
            }),
        }))
    }

    async fn upgraded_client_state(
        &self,
        request: Request<QueryUpgradedClientStateRequest>,
    ) -> Result<Response<QueryUpgradedClientStateResponse>, Status> {
        trace!("Got upgraded client state request: {:?}", request);

        let plan = self
            .upgrade_context
            .upgrade_plan()
            .map_err(|_| Status::not_found("Upgrade plan not found"))?;

        let upgraded_client_state_path = UpgradeClientPath::UpgradedClientState(plan.height);

        let upgraded_client_state = self
            .upgrade_context
            .upgraded_client_state(&upgraded_client_state_path)
            .map_err(|_| Status::not_found("Upgraded client state not found"))?;

        Ok(Response::new(QueryUpgradedClientStateResponse {
            upgraded_client_state: Some(upgraded_client_state.into()),
        }))
    }

    async fn upgraded_consensus_state(
        &self,
        request: Request<QueryUpgradedConsensusStateRequest>,
    ) -> Result<Response<QueryUpgradedConsensusStateResponse>, Status> {
        trace!("Got upgraded consensus state request: {:?}", request);

        let plan = self
            .upgrade_context
            .upgrade_plan()
            .map_err(|_| Status::not_found("Upgrade plan not found"))?;

        let upgraded_consensus_state_path =
            UpgradeClientPath::UpgradedClientConsensusState(plan.height);

        let upgraded_consensus_state = self
            .upgrade_context
            .upgraded_consensus_state(&upgraded_consensus_state_path)
            .map_err(|_| Status::not_found("Upgraded consensus state not found"))?;

        Ok(Response::new(QueryUpgradedConsensusStateResponse {
            upgraded_consensus_state: Some(upgraded_consensus_state.into()),
        }))
    }
}
