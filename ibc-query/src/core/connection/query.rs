//! Provides utility functions for querying IBC connection states.

use ibc::core::client::context::ClientValidationContext;
use ibc::core::host::types::path::{
    ClientConnectionPath, ClientConsensusStatePath, ClientStatePath, ConnectionPath, Path,
};
use ibc::core::host::{ConsensusStateRef, ValidationContext};
use ibc::primitives::prelude::format;
use ibc::primitives::proto::Any;

use super::{
    QueryClientConnectionsRequest, QueryClientConnectionsResponse,
    QueryConnectionClientStateRequest, QueryConnectionClientStateResponse,
    QueryConnectionConsensusStateRequest, QueryConnectionConsensusStateResponse,
    QueryConnectionParamsRequest, QueryConnectionParamsResponse, QueryConnectionRequest,
    QueryConnectionResponse, QueryConnectionsRequest, QueryConnectionsResponse,
};
use crate::core::client::IdentifiedClientState;
use crate::core::context::{ProvableContext, QueryContext};
use crate::error::QueryError;
use crate::types::Proof;

/// Queries for the connection end of a given connection id.
pub fn query_connection<I>(
    ibc_ctx: &I,
    request: &QueryConnectionRequest,
) -> Result<QueryConnectionResponse, QueryError>
where
    I: ValidationContext + ProvableContext,
{
    let connection_end = ibc_ctx.connection_end(&request.connection_id)?;

    let proof_height = match request.query_height {
        Some(height) => height,
        None => ibc_ctx.host_height()?,
    };

    let proof = ibc_ctx
        .get_proof(
            proof_height,
            &Path::Connection(ConnectionPath::new(&request.connection_id)),
        )
        .ok_or_else(|| {
            QueryError::proof_not_found(format!(
                "Proof not found for connection path: {:?}",
                request.connection_id
            ))
        })?;

    Ok(QueryConnectionResponse::new(
        connection_end,
        proof,
        proof_height,
    ))
}

/// Queries for all the existing connection ends.
pub fn query_connections<I>(
    ibc_ctx: &I,
    _request: &QueryConnectionsRequest,
) -> Result<QueryConnectionsResponse, QueryError>
where
    I: QueryContext,
{
    let connections = ibc_ctx.connection_ends()?;

    Ok(QueryConnectionsResponse::new(
        connections,
        ibc_ctx.host_height()?,
        None,
    ))
}

/// Queries for all the existing connection ends for a given client.
pub fn query_client_connections<I>(
    ibc_ctx: &I,
    request: &QueryClientConnectionsRequest,
) -> Result<QueryClientConnectionsResponse, QueryError>
where
    I: QueryContext,
{
    let connections = ibc_ctx.client_connection_ends(&request.client_id)?;

    let proof_height = match request.query_height {
        Some(height) => height,
        None => ibc_ctx.host_height()?,
    };

    let proof: Proof = ibc_ctx
        .get_proof(
            proof_height,
            &Path::ClientConnection(ClientConnectionPath::new(request.client_id.clone())),
        )
        .ok_or_else(|| {
            QueryError::proof_not_found(format!(
                "Proof not found for client connection path: {:?}",
                request.client_id
            ))
        })?;

    Ok(QueryClientConnectionsResponse::new(
        connections,
        proof,
        proof_height,
    ))
}

/// Queries for the client state of a given connection id.
pub fn query_connection_client_state<I>(
    ibc_ctx: &I,
    request: &QueryConnectionClientStateRequest,
) -> Result<QueryConnectionClientStateResponse, QueryError>
where
    I: QueryContext,
{
    let connection_end = ibc_ctx.connection_end(&request.connection_id)?;

    let client_val_ctx = ibc_ctx.get_client_validation_context();

    let client_state = client_val_ctx.client_state(connection_end.client_id())?;

    let proof_height = match request.query_height {
        Some(height) => height,
        None => ibc_ctx.host_height()?,
    };

    let proof = ibc_ctx
        .get_proof(
            proof_height,
            &Path::ClientState(ClientStatePath::new(connection_end.client_id().clone())),
        )
        .ok_or_else(|| {
            QueryError::proof_not_found(format!(
                "Proof not found for client state path: {:?}",
                connection_end.client_id()
            ))
        })?;

    Ok(QueryConnectionClientStateResponse::new(
        IdentifiedClientState::new(connection_end.client_id().clone(), client_state.into()),
        proof,
        proof_height,
    ))
}

/// Queries for the consensus state of a given connection id and height.
pub fn query_connection_consensus_state<I>(
    ibc_ctx: &I,
    request: &QueryConnectionConsensusStateRequest,
) -> Result<QueryConnectionConsensusStateResponse, QueryError>
where
    I: ValidationContext + ProvableContext,
    ConsensusStateRef<I>: Into<Any>,
{
    let connection_end = ibc_ctx.connection_end(&request.connection_id)?;

    let consensus_path = ClientConsensusStatePath::new(
        connection_end.client_id().clone(),
        request.height.revision_number(),
        request.height.revision_height(),
    );

    let client_val_ctx = ibc_ctx.get_client_validation_context();

    let consensus_state = client_val_ctx.consensus_state(&consensus_path)?;

    let proof_height = match request.query_height {
        Some(height) => height,
        None => ibc_ctx.host_height()?,
    };

    let proof = ibc_ctx
        .get_proof(proof_height, &Path::ClientConsensusState(consensus_path))
        .ok_or_else(|| {
            QueryError::proof_not_found(format!(
                "Proof not found for consensus state path: {:?}",
                connection_end.client_id()
            ))
        })?;

    Ok(QueryConnectionConsensusStateResponse::new(
        consensus_state.into(),
        connection_end.client_id().clone(),
        proof,
        proof_height,
    ))
}

/// Queries for the connection parameters.
pub fn query_connection_params<I>(
    ibc_ctx: &I,
    _request: &QueryConnectionParamsRequest,
) -> Result<QueryConnectionParamsResponse, QueryError>
where
    I: QueryContext,
{
    Ok(QueryConnectionParamsResponse::new(
        ibc_ctx.max_expected_time_per_block().as_secs(),
    ))
}
