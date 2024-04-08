//! Provides utility functions for querying IBC client states.

use ibc::core::client::context::client_state::ClientStateValidation;
use ibc::core::client::context::ClientValidationContext;
use ibc::core::client::types::error::ClientError;
use ibc::core::host::types::path::{
    ClientConsensusStatePath, ClientStatePath, Path, UpgradeClientPath,
};
use ibc::core::host::{ConsensusStateRef, ValidationContext};
use ibc::cosmos_host::upgrade_proposal::{UpgradeValidationContext, UpgradedConsensusStateRef};
use ibc::primitives::prelude::format;
use ibc::primitives::proto::Any;

use super::{
    ConsensusStateWithHeight, IdentifiedClientState, QueryClientStateResponse,
    QueryClientStatesRequest, QueryClientStatesResponse, QueryClientStatusRequest,
    QueryClientStatusResponse, QueryConsensusStateHeightsRequest,
    QueryConsensusStateHeightsResponse, QueryConsensusStateRequest, QueryConsensusStateResponse,
    QueryConsensusStatesRequest, QueryConsensusStatesResponse, QueryUpgradedClientStateRequest,
    QueryUpgradedClientStateResponse, QueryUpgradedConsensusStateRequest,
    QueryUpgradedConsensusStateResponse,
};
use crate::core::client::QueryClientStateRequest;
use crate::core::context::{ProvableContext, QueryContext};
use crate::error::QueryError;

/// Queries for the client state of a given client id.
pub fn query_client_state<I>(
    ibc_ctx: &I,
    request: &QueryClientStateRequest,
) -> Result<QueryClientStateResponse, QueryError>
where
    I: QueryContext,
{
    let client_id = request.client_id.clone();

    let client_val_ctx = ibc_ctx.get_client_validation_context();

    let client_state = client_val_ctx.client_state(&client_id)?;

    let proof_height = match request.query_height {
        Some(height) => height,
        None => ibc_ctx.host_height()?,
    };

    let proof = ibc_ctx
        .get_proof(
            proof_height,
            &Path::ClientState(ClientStatePath::new(client_id.clone())),
        )
        .ok_or_else(|| {
            QueryError::proof_not_found(format!(
                "Proof not found for client state path: {client_id:?}"
            ))
        })?;

    Ok(QueryClientStateResponse::new(
        client_state.into(),
        proof,
        proof_height,
    ))
}

/// Queries for all the existing client states.
pub fn query_client_states<I>(
    ibc_ctx: &I,
    _request: &QueryClientStatesRequest,
) -> Result<QueryClientStatesResponse, QueryError>
where
    I: QueryContext,
{
    let client_states = ibc_ctx.client_states()?;

    Ok(QueryClientStatesResponse::new(
        client_states
            .into_iter()
            .map(|(id, state)| IdentifiedClientState::new(id, state.into()))
            .collect(),
        // no support for pagination yet
        None,
    ))
}

/// Queries for the consensus state of a given client id and height.
pub fn query_consensus_state<I>(
    ibc_ctx: &I,
    request: &QueryConsensusStateRequest,
) -> Result<QueryConsensusStateResponse, QueryError>
where
    I: QueryContext,
    ConsensusStateRef<I>: Into<Any>,
{
    let client_id = request.client_id.clone();

    let (height, consensus_state) = if let Some(height) = request.consensus_height {
        let client_val_ctx = ibc_ctx.get_client_validation_context();

        let consensus_state = client_val_ctx.consensus_state(&ClientConsensusStatePath::new(
            client_id.clone(),
            height.revision_number(),
            height.revision_height(),
        ))?;

        (height, consensus_state)
    } else {
        ibc_ctx
            .consensus_states(&client_id)?
            .into_iter()
            .max_by_key(|&(h, _)| h)
            .ok_or_else(|| {
                QueryError::proof_not_found(format!(
                    "No consensus state found for client: {client_id:?}"
                ))
            })?
    };

    let proof_height = match request.query_height {
        Some(height) => height,
        None => ibc_ctx.host_height()?,
    };

    let proof = ibc_ctx
        .get_proof(
            proof_height,
            &Path::ClientConsensusState(ClientConsensusStatePath::new(
                client_id.clone(),
                height.revision_number(),
                height.revision_height(),
            )),
        )
        .ok_or_else(|| {
            QueryError::proof_not_found(format!(
                "Proof not found for consensus state path: {client_id:?}"
            ))
        })?;

    Ok(QueryConsensusStateResponse::new(
        consensus_state.into(),
        proof,
        proof_height,
    ))
}

/// Queries for all the consensus states of a given client id.
pub fn query_consensus_states<I>(
    ibc_ctx: &I,
    request: &QueryConsensusStatesRequest,
) -> Result<QueryConsensusStatesResponse, QueryError>
where
    I: QueryContext,
    ConsensusStateRef<I>: Into<Any>,
{
    let consensus_states = ibc_ctx.consensus_states(&request.client_id)?;

    Ok(QueryConsensusStatesResponse::new(
        consensus_states
            .into_iter()
            .map(|(height, state)| ConsensusStateWithHeight::new(height, state.into()))
            .collect(),
        // no support for pagination yet,
        None,
    ))
}

/// Queries for the heights of all the consensus states of a given client id.
pub fn query_consensus_state_heights<I>(
    ibc_ctx: &I,
    request: &QueryConsensusStateHeightsRequest,
) -> Result<QueryConsensusStateHeightsResponse, QueryError>
where
    I: QueryContext,
{
    let consensus_state_heights = ibc_ctx.consensus_state_heights(&request.client_id)?;

    Ok(QueryConsensusStateHeightsResponse::new(
        consensus_state_heights,
        // no support for pagination yet
        None,
    ))
}

/// Queries for the status (Active, Frozen, Expired, Unauthorized) of a given client.
pub fn query_client_status<I>(
    ibc_ctx: &I,
    request: &QueryClientStatusRequest,
) -> Result<QueryClientStatusResponse, QueryError>
where
    I: ValidationContext,
{
    let client_val_ctx = ibc_ctx.get_client_validation_context();
    let client_state = client_val_ctx.client_state(&request.client_id)?;
    let client_validation_ctx = ibc_ctx.get_client_validation_context();
    let client_status = client_state.status(client_validation_ctx, &request.client_id)?;

    Ok(QueryClientStatusResponse::new(client_status))
}

/// Queries for the upgraded client state.
pub fn query_upgraded_client_state<I, U>(
    ibc_ctx: &I,
    upgrade_ctx: &U,
    request: &QueryUpgradedClientStateRequest,
) -> Result<QueryUpgradedClientStateResponse, QueryError>
where
    I: ValidationContext,
    U: UpgradeValidationContext + ProvableContext,
{
    let upgrade_revision_height = match request.upgrade_height {
        Some(height) => height.revision_height(),
        None => {
            upgrade_ctx
                .upgrade_plan()
                .map_err(ClientError::from)?
                .height
        }
    };

    let upgraded_client_state_path =
        UpgradeClientPath::UpgradedClientState(upgrade_revision_height);

    let upgraded_client_state = upgrade_ctx
        .upgraded_client_state(&upgraded_client_state_path)
        .map_err(ClientError::from)?;

    let proof_height = match request.query_height {
        Some(height) => height,
        None => ibc_ctx.host_height()?,
    };

    let proof = upgrade_ctx
        .get_proof(
            proof_height,
            &Path::UpgradeClient(upgraded_client_state_path),
        )
        .ok_or_else(|| {
            QueryError::proof_not_found(format!(
                "Proof not found for upgraded client state at: {proof_height:?}"
            ))
        })?;

    Ok(QueryUpgradedClientStateResponse::new(
        upgraded_client_state.into(),
        proof,
        proof_height,
    ))
}

/// Queries for the upgraded consensus state.
pub fn query_upgraded_consensus_state<I, U>(
    ibc_ctx: &I,
    upgrade_ctx: &U,
    request: &QueryUpgradedConsensusStateRequest,
) -> Result<QueryUpgradedConsensusStateResponse, QueryError>
where
    I: ValidationContext,
    U: UpgradeValidationContext + ProvableContext,
    UpgradedConsensusStateRef<U>: Into<Any>,
{
    let upgrade_revision_height = match request.upgrade_height {
        Some(height) => height.revision_height(),
        None => {
            upgrade_ctx
                .upgrade_plan()
                .map_err(ClientError::from)?
                .height
        }
    };

    let upgraded_consensus_state_path =
        UpgradeClientPath::UpgradedClientConsensusState(upgrade_revision_height);

    let upgraded_consensus_state = upgrade_ctx
        .upgraded_consensus_state(&upgraded_consensus_state_path)
        .map_err(ClientError::from)?;

    let proof_height = match request.query_height {
        Some(height) => height,
        None => ibc_ctx.host_height()?,
    };

    let proof = upgrade_ctx
        .get_proof(
            proof_height,
            &Path::UpgradeClient(upgraded_consensus_state_path),
        )
        .ok_or_else(|| {
            QueryError::proof_not_found(format!(
                "Proof not found for upgraded consensus state at: {proof_height:?}"
            ))
        })?;

    Ok(QueryUpgradedConsensusStateResponse::new(
        upgraded_consensus_state.into(),
        proof,
        proof_height,
    ))
}
