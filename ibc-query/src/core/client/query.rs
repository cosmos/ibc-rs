//! Provides utility functions for querying IBC client states.

use alloc::format;
use core::str::FromStr;

use ibc::core::client::context::client_state::ClientStateValidation;
use ibc::core::client::types::error::ClientError;
use ibc::core::client::types::Height;
use ibc::core::host::types::identifiers::ClientId;
use ibc::core::host::types::path::{
    ClientConsensusStatePath, ClientStatePath, Path, UpgradeClientPath,
};
use ibc::core::host::ValidationContext;
use ibc::cosmos_host::upgrade_proposal::UpgradeValidationContext;
use ibc_proto::google::protobuf::Any;
use ibc_proto::ibc::core::client::v1::{
    ConsensusStateWithHeight, IdentifiedClientState, QueryClientStateRequest,
    QueryClientStateResponse, QueryClientStatesRequest, QueryClientStatesResponse,
    QueryClientStatusRequest, QueryClientStatusResponse, QueryConsensusStateHeightsRequest,
    QueryConsensusStateHeightsResponse, QueryConsensusStateRequest, QueryConsensusStateResponse,
    QueryConsensusStatesRequest, QueryConsensusStatesResponse, QueryUpgradedClientStateRequest,
    QueryUpgradedClientStateResponse, QueryUpgradedConsensusStateRequest,
    QueryUpgradedConsensusStateResponse,
};

use crate::core::context::{ProvableContext, QueryContext};
use crate::error::QueryError;

/// Queries for the client state of a given client id.
pub fn query_client_state<I>(
    ibc_ctx: &I,
    request: &QueryClientStateRequest,
) -> Result<QueryClientStateResponse, QueryError>
where
    I: ValidationContext + ProvableContext,
    <I as ValidationContext>::AnyClientState: Into<Any>,
{
    let client_id = ClientId::from_str(request.client_id.as_str())?;

    let client_state = ibc_ctx.client_state(&client_id)?;

    let current_height = ibc_ctx.host_height()?;

    let proof = ibc_ctx
        .get_proof(
            current_height,
            &Path::ClientState(ClientStatePath::new(client_id.clone())),
        )
        .ok_or(QueryError::ProofNotFound {
            description: format!("Proof not found for client state path: {client_id:?}"),
        })?;

    Ok(QueryClientStateResponse {
        client_state: Some(client_state.into()),
        proof,
        proof_height: Some(current_height.into()),
    })
}

/// Queries for all the existing client states.
pub fn query_client_states<I>(
    ibc_ctx: &I,
    _request: &QueryClientStatesRequest,
) -> Result<QueryClientStatesResponse, QueryError>
where
    I: QueryContext,
    <I as ValidationContext>::AnyClientState: Into<Any>,
{
    let client_states = ibc_ctx.client_states()?;

    Ok(QueryClientStatesResponse {
        client_states: client_states
            .into_iter()
            .map(|(id, state)| IdentifiedClientState {
                client_id: id.into(),
                client_state: Some(state.into()),
            })
            .collect(),
        // no support for pagination yet
        pagination: None,
    })
}

/// Queries for the consensus state of a given client id and height.
pub fn query_consensus_state<I>(
    ibc_ctx: &I,
    request: &QueryConsensusStateRequest,
) -> Result<QueryConsensusStateResponse, QueryError>
where
    I: QueryContext,
    <I as ValidationContext>::AnyConsensusState: Into<Any>,
{
    let client_id = ClientId::from_str(request.client_id.as_str())?;

    let (height, consensus_state) = if request.latest_height {
        ibc_ctx
            .consensus_states(&client_id)?
            .into_iter()
            .max_by_key(|(h, _)| *h)
            .ok_or(QueryError::ProofNotFound {
                description: format!("No consensus state found for client: {client_id:?}"),
            })?
    } else {
        let height = Height::new(request.revision_number, request.revision_height)?;

        let consensus_state = ibc_ctx.consensus_state(&ClientConsensusStatePath::new(
            client_id.clone(),
            height.revision_number(),
            height.revision_height(),
        ))?;

        (height, consensus_state)
    };

    let current_height = ibc_ctx.host_height()?;

    let proof = ibc_ctx
        .get_proof(
            current_height,
            &Path::ClientConsensusState(ClientConsensusStatePath::new(
                client_id.clone(),
                height.revision_number(),
                height.revision_height(),
            )),
        )
        .ok_or(QueryError::ProofNotFound {
            description: format!("Proof not found for consensus state path: {client_id:?}"),
        })?;

    Ok(QueryConsensusStateResponse {
        consensus_state: Some(consensus_state.into()),
        proof,
        proof_height: Some(current_height.into()),
    })
}

/// Queries for all the consensus states of a given client id.
pub fn query_consensus_states<I>(
    ibc_ctx: &I,
    request: &QueryConsensusStatesRequest,
) -> Result<QueryConsensusStatesResponse, QueryError>
where
    I: QueryContext,
    <I as ValidationContext>::AnyConsensusState: Into<Any>,
{
    let client_id = ClientId::from_str(request.client_id.as_str())?;

    let consensus_states = ibc_ctx.consensus_states(&client_id)?;

    Ok(QueryConsensusStatesResponse {
        consensus_states: consensus_states
            .into_iter()
            .map(|(height, state)| ConsensusStateWithHeight {
                height: Some(height.into()),
                consensus_state: Some(state.into()),
            })
            .collect(),
        // no support for pagination yet
        pagination: None,
    })
}

/// Queries for the heights of all the consensus states of a given client id.
pub fn query_consensus_state_heights<I>(
    ibc_ctx: &I,
    request: &QueryConsensusStateHeightsRequest,
) -> Result<QueryConsensusStateHeightsResponse, QueryError>
where
    I: QueryContext,
{
    let client_id = ClientId::from_str(request.client_id.as_str())?;

    let consensus_state_heights = ibc_ctx.consensus_state_heights(&client_id)?;

    Ok(QueryConsensusStateHeightsResponse {
        consensus_state_heights: consensus_state_heights
            .into_iter()
            .map(|height| height.into())
            .collect(),
        // no support for pagination yet
        pagination: None,
    })
}

/// Queries for the status (Active, Frozen, Expired, Unauthorized) of a given client.
pub fn query_client_status<I>(
    ibc_ctx: &I,
    request: &QueryClientStatusRequest,
) -> Result<QueryClientStatusResponse, QueryError>
where
    I: ValidationContext,
{
    let client_id = ClientId::from_str(request.client_id.as_str())?;

    let client_state = ibc_ctx.client_state(&client_id)?;
    let client_validation_ctx = ibc_ctx.get_client_validation_context();
    let client_status = client_state.status(client_validation_ctx, &client_id)?;

    Ok(QueryClientStatusResponse {
        status: format!("{client_status}"),
    })
}

/// Queries for the upgraded client state.
pub fn query_upgraded_client_state<U>(
    upgrade_ctx: &U,
    _request: &QueryUpgradedClientStateRequest,
) -> Result<QueryUpgradedClientStateResponse, QueryError>
where
    U: UpgradeValidationContext,
    <U as UpgradeValidationContext>::AnyClientState: Into<Any>,
{
    let plan = upgrade_ctx.upgrade_plan().map_err(ClientError::from)?;

    let upgraded_client_state_path = UpgradeClientPath::UpgradedClientState(plan.height);

    let upgraded_client_state = upgrade_ctx
        .upgraded_client_state(&upgraded_client_state_path)
        .map_err(ClientError::from)?;

    Ok(QueryUpgradedClientStateResponse {
        upgraded_client_state: Some(upgraded_client_state.into()),
    })
}

/// Queries for the upgraded consensus state.
pub fn query_upgraded_consensus_state<U>(
    upgrade_ctx: &U,
    _request: &QueryUpgradedConsensusStateRequest,
) -> Result<QueryUpgradedConsensusStateResponse, QueryError>
where
    U: UpgradeValidationContext,
    <U as UpgradeValidationContext>::AnyConsensusState: Into<Any>,
{
    let plan = upgrade_ctx.upgrade_plan().map_err(ClientError::from)?;

    let upgraded_consensus_state_path =
        UpgradeClientPath::UpgradedClientConsensusState(plan.height);

    let upgraded_consensus_state = upgrade_ctx
        .upgraded_consensus_state(&upgraded_consensus_state_path)
        .map_err(ClientError::from)?;

    Ok(QueryUpgradedConsensusStateResponse {
        upgraded_consensus_state: Some(upgraded_consensus_state.into()),
    })
}
