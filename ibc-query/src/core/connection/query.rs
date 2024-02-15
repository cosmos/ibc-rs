//! Provides utility functions for querying IBC connection states.

use alloc::format;
use alloc::vec::Vec;
use core::str::FromStr;

use ibc::core::client::types::Height;
use ibc::core::host::types::identifiers::{ClientId, ConnectionId};
use ibc::core::host::types::path::{
    ClientConnectionPath, ClientConsensusStatePath, ClientStatePath, ConnectionPath, Path,
};
use ibc::core::host::ValidationContext;
use ibc_proto::google::protobuf::Any;
use ibc_proto::ibc::core::client::v1::IdentifiedClientState;
use ibc_proto::ibc::core::connection::v1::{
    Params as ConnectionParams, QueryClientConnectionsRequest, QueryClientConnectionsResponse,
    QueryConnectionClientStateRequest, QueryConnectionClientStateResponse,
    QueryConnectionConsensusStateRequest, QueryConnectionConsensusStateResponse,
    QueryConnectionParamsRequest, QueryConnectionParamsResponse, QueryConnectionRequest,
    QueryConnectionResponse, QueryConnectionsRequest, QueryConnectionsResponse,
};

use crate::core::context::{ProvableContext, QueryContext};
use crate::error::QueryError;

/// Queries for the connection end of a given connection id.
pub fn query_connection<I>(
    ibc_ctx: &I,
    request: &QueryConnectionRequest,
) -> Result<QueryConnectionResponse, QueryError>
where
    I: ValidationContext + ProvableContext,
{
    let connection_id = ConnectionId::from_str(request.connection_id.as_str())?;

    let connection_end = ibc_ctx.connection_end(&connection_id)?;

    let current_height = ibc_ctx.host_height()?;

    let proof = ibc_ctx
        .get_proof(
            current_height,
            &Path::Connection(ConnectionPath::new(&connection_id)),
        )
        .ok_or(QueryError::ProofNotFound {
            description: format!("Proof not found for connection path: {connection_id:?}"),
        })?;

    Ok(QueryConnectionResponse {
        connection: Some(connection_end.into()),
        proof,
        proof_height: Some(current_height.into()),
    })
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

    Ok(QueryConnectionsResponse {
        connections: connections.into_iter().map(Into::into).collect(),
        height: Some(ibc_ctx.host_height()?.into()),
        // no support for pagination yet
        pagination: None,
    })
}

/// Queries for all the existing connection ends for a given client.
pub fn query_client_connections<I>(
    ibc_ctx: &I,
    request: &QueryClientConnectionsRequest,
) -> Result<QueryClientConnectionsResponse, QueryError>
where
    I: QueryContext,
    <I as ValidationContext>::AnyClientState: Into<Any>,
{
    let client_id = ClientId::from_str(request.client_id.as_str())?;

    let connections = ibc_ctx.client_connection_ends(&client_id)?;

    let current_height = ibc_ctx.host_height()?;

    let proof: Vec<u8> = ibc_ctx
        .get_proof(
            current_height,
            &Path::ClientConnection(ClientConnectionPath::new(client_id.clone())),
        )
        .ok_or(QueryError::ProofNotFound {
            description: format!("Proof not found for client connection path: {client_id:?}"),
        })?;

    Ok(QueryClientConnectionsResponse {
        connection_paths: connections.into_iter().map(|x| x.as_str().into()).collect(),
        proof,
        proof_height: Some(current_height.into()),
    })
}

/// Queries for the client state of a given connection id.
pub fn query_connection_client_state<I>(
    ibc_ctx: &I,
    request: &QueryConnectionClientStateRequest,
) -> Result<QueryConnectionClientStateResponse, QueryError>
where
    I: QueryContext,
    <I as ValidationContext>::AnyClientState: Into<Any>,
{
    let connection_id = ConnectionId::from_str(request.connection_id.as_str())?;

    let connection_end = ibc_ctx.connection_end(&connection_id)?;

    let client_state = ibc_ctx.client_state(connection_end.client_id())?;

    let current_height = ibc_ctx.host_height()?;

    let proof = ibc_ctx
        .get_proof(
            current_height,
            &Path::ClientState(ClientStatePath::new(connection_end.client_id().clone())),
        )
        .ok_or(QueryError::ProofNotFound {
            description: format!(
                "Proof not found for client state path: {:?}",
                connection_end.client_id()
            ),
        })?;

    Ok(QueryConnectionClientStateResponse {
        identified_client_state: Some(IdentifiedClientState {
            client_id: connection_end.client_id().as_str().into(),
            client_state: Some(client_state.into()),
        }),
        proof,
        proof_height: Some(current_height.into()),
    })
}

/// Queries for the consensus state of a given connection id and height.
pub fn query_connection_consensus_state<I>(
    ibc_ctx: &I,
    request: &QueryConnectionConsensusStateRequest,
) -> Result<QueryConnectionConsensusStateResponse, QueryError>
where
    I: ValidationContext + ProvableContext,
    <I as ValidationContext>::AnyConsensusState: Into<Any>,
{
    let connection_id = ConnectionId::from_str(request.connection_id.as_str())?;

    let connection_end = ibc_ctx.connection_end(&connection_id)?;

    let height = Height::new(request.revision_number, request.revision_height)?;

    let consensus_path = ClientConsensusStatePath::new(
        connection_end.client_id().clone(),
        height.revision_number(),
        height.revision_height(),
    );

    let consensus_state = ibc_ctx.consensus_state(&consensus_path)?;

    let current_height = ibc_ctx.host_height()?;

    let proof = ibc_ctx
        .get_proof(current_height, &Path::ClientConsensusState(consensus_path))
        .ok_or(QueryError::ProofNotFound {
            description: format!(
                "Proof not found for consensus state path: {:?}",
                connection_end.client_id()
            ),
        })?;

    Ok(QueryConnectionConsensusStateResponse {
        consensus_state: Some(consensus_state.into()),
        client_id: connection_end.client_id().as_str().into(),
        proof,
        proof_height: Some(current_height.into()),
    })
}

/// Queries for the connection parameters.
pub fn query_connection_params<I>(
    ibc_ctx: &I,
    _request: &QueryConnectionParamsRequest,
) -> Result<QueryConnectionParamsResponse, QueryError>
where
    I: QueryContext,
{
    Ok(QueryConnectionParamsResponse {
        params: Some(ConnectionParams {
            max_expected_time_per_block: ibc_ctx.max_expected_time_per_block().as_secs(),
        }),
    })
}
