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

use crate::prelude::*;

use crate::{
    core::{
        ics02_client::error::ClientError,
        ics24_host::{
            identifier::ClientId,
            path::{ClientConsensusStatePath, ClientStatePath, Path, UpgradeClientPath},
        },
        ContextError, ValidationContext,
    },
    hosts::tendermint::upgrade_proposal::UpgradeValidationContext,
    services::core::context::QueryContext,
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
        let request_ref = request.get_ref();

        trace!("Got client state request: {:?}", request_ref);

        let client_id = ClientId::from_str(request_ref.client_id.as_str())?;
        let client_state = self.ibc_context.client_state(&client_id)?;

        let current_height = self.ibc_context.host_height()?;

        let proof = self
            .ibc_context
            .get_proof(
                current_height,
                &Path::ClientState(ClientStatePath::new(&client_id)),
            )
            .ok_or_else(|| {
                Status::not_found(format!(
                    "Client state not found for client {} at height {}",
                    client_id, current_height
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
        let request_ref = request.get_ref();

        trace!("Got client states request: {:?}", request_ref);

        let client_states = self.ibc_context.client_states()?;

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
        let request_ref = request.get_ref();

        trace!("Got consensus state request: {:?}", request_ref);

        let client_id = ClientId::from_str(request_ref.client_id.as_str())?;

        let height = Height::new(request_ref.revision_number, request_ref.revision_height)
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

        let consensus_state = self
            .ibc_context
            .consensus_state(&ClientConsensusStatePath::new(&client_id, &height))?;

        let current_height = self.ibc_context.host_height()?;

        let proof = self
            .ibc_context
            .get_proof(
                current_height,
                &Path::ClientConsensusState(ClientConsensusStatePath::new(&client_id, &height)),
            )
            .ok_or_else(|| {
                Status::not_found(format!(
                    "Consensus state not found for client {} at height {}",
                    client_id, height
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
        let request_ref = request.get_ref();

        trace!("Got consensus states request: {:?}", request_ref);

        let client_id = ClientId::from_str(request_ref.client_id.as_str())?;

        let consensus_states = self.ibc_context.consensus_states(&client_id)?;

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
        let request_ref = request.get_ref();

        trace!("Got consensus state heights request: {:?}", request_ref);

        let client_id = ClientId::from_str(request_ref.client_id.as_str())?;

        let consensus_state_heights = self.ibc_context.consensus_state_heights(&client_id)?;

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
        let request_ref = request.get_ref();

        trace!("Got client status request: {:?}", request_ref);

        let client_id = ClientId::from_str(request_ref.client_id.as_str())?;

        let client_status = self.ibc_context.client_status(&client_id)?;

        Ok(Response::new(QueryClientStatusResponse {
            status: format!("{}", client_status),
        }))
    }

    async fn client_params(
        &self,
        request: Request<QueryClientParamsRequest>,
    ) -> Result<Response<QueryClientParamsResponse>, Status> {
        let request_ref = request.get_ref();

        trace!("Got client params request: {:?}", request_ref);

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
        let request_ref = request.get_ref();

        trace!("Got upgraded client state request: {:?}", request_ref);

        let plan = self
            .upgrade_context
            .upgrade_plan()
            .map_err(ClientError::from)
            .map_err(ContextError::from)?;

        let upgraded_client_state_path = UpgradeClientPath::UpgradedClientState(plan.height);

        let upgraded_client_state = self
            .upgrade_context
            .upgraded_client_state(&upgraded_client_state_path)
            .map_err(ClientError::from)
            .map_err(ContextError::from)?;

        Ok(Response::new(QueryUpgradedClientStateResponse {
            upgraded_client_state: Some(upgraded_client_state.into()),
        }))
    }

    async fn upgraded_consensus_state(
        &self,
        request: Request<QueryUpgradedConsensusStateRequest>,
    ) -> Result<Response<QueryUpgradedConsensusStateResponse>, Status> {
        let request_ref = request.get_ref();

        trace!("Got upgraded consensus state request: {:?}", request_ref);

        let plan = self
            .upgrade_context
            .upgrade_plan()
            .map_err(ClientError::from)
            .map_err(ContextError::from)?;

        let upgraded_consensus_state_path =
            UpgradeClientPath::UpgradedClientConsensusState(plan.height);

        let upgraded_consensus_state = self
            .upgrade_context
            .upgraded_consensus_state(&upgraded_consensus_state_path)
            .map_err(ClientError::from)
            .map_err(ContextError::from)?;

        Ok(Response::new(QueryUpgradedConsensusStateResponse {
            upgraded_consensus_state: Some(upgraded_consensus_state.into()),
        }))
    }
}
