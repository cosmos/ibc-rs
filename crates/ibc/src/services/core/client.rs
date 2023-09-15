//! [`ClientQueryService`](ClientQueryService) takes generics `I` and `U` to store `ibc_context` and `upgrade_context` that implement [`QueryContext`](QueryContext) and [`UpgradeValidationContext`](UpgradeValidationContext) respectively.
//! `I` must be a type where writes from one thread are readable from another.
//! This means using `Arc<Mutex<_>>` or `Arc<RwLock<_>>` in most cases.

use core::str::FromStr;
use std::boxed::Box;

use ibc_proto::google::protobuf::Any;
use ibc_proto::ibc::core::client::v1::query_server::Query as ClientQuery;
use ibc_proto::ibc::core::client::v1::{
    ConsensusStateWithHeight, IdentifiedClientState, QueryClientParamsRequest,
    QueryClientParamsResponse, QueryClientStateRequest, QueryClientStateResponse,
    QueryClientStatesRequest, QueryClientStatesResponse, QueryClientStatusRequest,
    QueryClientStatusResponse, QueryConsensusStateHeightsRequest,
    QueryConsensusStateHeightsResponse, QueryConsensusStateRequest, QueryConsensusStateResponse,
    QueryConsensusStatesRequest, QueryConsensusStatesResponse, QueryUpgradedClientStateRequest,
    QueryUpgradedClientStateResponse, QueryUpgradedConsensusStateRequest,
    QueryUpgradedConsensusStateResponse,
};
use tonic::{Request, Response, Status};

use crate::core::ics02_client::client_state::ClientStateValidation;
use crate::core::ics02_client::error::ClientError;
use crate::core::ics24_host::identifier::ClientId;
use crate::core::ics24_host::path::{
    ClientConsensusStatePath, ClientStatePath, Path, UpgradeClientPath,
};
use crate::core::{ContextError, ValidationContext};
use crate::hosts::tendermint::upgrade_proposal::UpgradeValidationContext;
use crate::prelude::*;
use crate::services::core::context::QueryContext;
use crate::Height;

// TODO(rano): currently the services don't support pagination, so we return all the results.

/// Generics `I` and `U` must be a type where writes from one thread are readable from another.
/// This means using `Arc<Mutex<_>>` or `Arc<RwLock<_>>` in most cases.
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
    /// Parameters `ibc_context` and `upgrade_context` must be a type where writes from one thread are readable from another.
    /// This means using `Arc<Mutex<_>>` or `Arc<RwLock<_>>` in most cases.
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
                    "Proof unavailable for client {} at height {}",
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
        _request: Request<QueryClientStatesRequest>,
    ) -> Result<Response<QueryClientStatesResponse>, Status> {
        let client_states = self.ibc_context.client_states()?;

        Ok(Response::new(QueryClientStatesResponse {
            client_states: client_states
                .into_iter()
                .map(|(id, state)| IdentifiedClientState {
                    client_id: id.into(),
                    client_state: Some(state.into()),
                })
                .collect(),
            // no support for pagination yet
            pagination: None,
        }))
    }

    async fn consensus_state(
        &self,
        request: Request<QueryConsensusStateRequest>,
    ) -> Result<Response<QueryConsensusStateResponse>, Status> {
        let request_ref = request.get_ref();

        let client_id = ClientId::from_str(request_ref.client_id.as_str())?;

        let (height, consensus_state) = if request_ref.latest_height {
            self.ibc_context
                .consensus_states(&client_id)?
                .into_iter()
                .max_by_key(|(h, _)| *h)
                .ok_or_else(|| {
                    Status::not_found(format!(
                        "Consensus state not found for client {}",
                        client_id
                    ))
                })?
        } else {
            let height = Height::new(request_ref.revision_number, request_ref.revision_height)
                .map_err(|e| Status::invalid_argument(e.to_string()))?;
            let consensus_state = self
                .ibc_context
                .consensus_state(&ClientConsensusStatePath::new(&client_id, &height))?;

            (height, consensus_state)
        };

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
            // no support for pagination yet
            pagination: None,
        }))
    }

    async fn consensus_state_heights(
        &self,
        request: Request<QueryConsensusStateHeightsRequest>,
    ) -> Result<Response<QueryConsensusStateHeightsResponse>, Status> {
        let request_ref = request.get_ref();

        let client_id = ClientId::from_str(request_ref.client_id.as_str())?;

        let consensus_state_heights = self.ibc_context.consensus_state_heights(&client_id)?;

        Ok(Response::new(QueryConsensusStateHeightsResponse {
            consensus_state_heights: consensus_state_heights
                .into_iter()
                .map(|height| height.into())
                .collect(),
            // no support for pagination yet
            pagination: None,
        }))
    }

    async fn client_status(
        &self,
        request: Request<QueryClientStatusRequest>,
    ) -> Result<Response<QueryClientStatusResponse>, Status> {
        let request_ref = request.get_ref();

        let client_id = ClientId::from_str(request_ref.client_id.as_str())?;

        let client_state = self.ibc_context.client_state(&client_id)?;
        let client_validation_ctx = self.ibc_context.get_client_validation_context();
        let client_status = client_state
            .status(client_validation_ctx, &client_id)
            .map_err(ContextError::from)?;

        Ok(Response::new(QueryClientStatusResponse {
            status: format!("{}", client_status),
        }))
    }

    async fn client_params(
        &self,
        _request: Request<QueryClientParamsRequest>,
    ) -> Result<Response<QueryClientParamsResponse>, Status> {
        Err(Status::unimplemented(
            "Querying ClientParams is not supported yet",
        ))
    }

    async fn upgraded_client_state(
        &self,
        _request: Request<QueryUpgradedClientStateRequest>,
    ) -> Result<Response<QueryUpgradedClientStateResponse>, Status> {
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
        _request: Request<QueryUpgradedConsensusStateRequest>,
    ) -> Result<Response<QueryUpgradedConsensusStateResponse>, Status> {
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
