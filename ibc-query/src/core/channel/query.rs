//! Provides utility functions for querying IBC channel states.

use ibc::core::client::context::ClientValidationContext;
use ibc::core::host::types::path::{
    AckPath, ChannelEndPath, ClientConsensusStatePath, ClientStatePath, CommitmentPath, Path,
    ReceiptPath, SeqRecvPath, SeqSendPath,
};
use ibc::core::host::{ConsensusStateRef, ValidationContext};
use ibc::primitives::prelude::format;
use ibc_proto::google::protobuf::Any;

use super::{
    QueryChannelClientStateRequest, QueryChannelClientStateResponse,
    QueryChannelConsensusStateRequest, QueryChannelConsensusStateResponse, QueryChannelRequest,
    QueryChannelResponse, QueryChannelsRequest, QueryChannelsResponse,
    QueryConnectionChannelsRequest, QueryConnectionChannelsResponse,
    QueryNextSequenceReceiveRequest, QueryNextSequenceReceiveResponse,
    QueryNextSequenceSendRequest, QueryNextSequenceSendResponse, QueryPacketAcknowledgementRequest,
    QueryPacketAcknowledgementResponse, QueryPacketAcknowledgementsRequest,
    QueryPacketAcknowledgementsResponse, QueryPacketCommitmentRequest,
    QueryPacketCommitmentResponse, QueryPacketCommitmentsRequest, QueryPacketCommitmentsResponse,
    QueryPacketReceiptRequest, QueryPacketReceiptResponse, QueryUnreceivedAcksRequest,
    QueryUnreceivedAcksResponse, QueryUnreceivedPacketsRequest, QueryUnreceivedPacketsResponse,
};
use crate::core::client::IdentifiedClientState;
use crate::core::context::{ProvableContext, QueryContext};
use crate::error::QueryError;

/// Queries for a specific IBC channel by the given channel and port ids and
/// returns the channel end with the associated proof.
pub fn query_channel<I>(
    ibc_ctx: &I,
    request: &QueryChannelRequest,
) -> Result<QueryChannelResponse, QueryError>
where
    I: ValidationContext + ProvableContext,
{
    let channel_end_path = ChannelEndPath::new(&request.port_id, &request.channel_id);

    let channel_end = ibc_ctx.channel_end(&channel_end_path)?;

    let proof_height = match request.query_height {
        Some(height) => height,
        None => ibc_ctx.host_height()?,
    };

    let proof = ibc_ctx
        .get_proof(proof_height, &Path::ChannelEnd(channel_end_path.clone()))
        .ok_or_else(|| {
            QueryError::proof_not_found(format!(
                "Proof not found for channel end path {channel_end_path:?}"
            ))
        })?;

    Ok(QueryChannelResponse::new(channel_end, proof, proof_height))
}

/// Queries for all existing IBC channels and returns the corresponding channel ends
pub fn query_channels<I>(
    ibc_ctx: &I,
    _request: &QueryChannelsRequest,
) -> Result<QueryChannelsResponse, QueryError>
where
    I: QueryContext,
{
    let channel_ends = ibc_ctx.channel_ends()?;

    Ok(QueryChannelsResponse::new(
        channel_ends,
        ibc_ctx.host_height()?,
        None,
    ))
}

/// Queries for all channels associated with a given connection
pub fn query_connection_channels<I>(
    ibc_ctx: &I,
    request: &QueryConnectionChannelsRequest,
) -> Result<QueryConnectionChannelsResponse, QueryError>
where
    I: QueryContext,
{
    let all_channel_ends = ibc_ctx.channel_ends()?;

    let connection_channel_ends = all_channel_ends
        .into_iter()
        .filter(|channel_end| {
            channel_end
                .channel_end
                .connection_hops()
                .iter()
                .any(|connection_hop| connection_hop == &request.connection_id)
        })
        .map(Into::into)
        .collect();

    Ok(QueryConnectionChannelsResponse::new(
        connection_channel_ends,
        ibc_ctx.host_height()?,
        None,
    ))
}

/// Queries for the client state associated with a channel by the given channel
/// and port ids
pub fn query_channel_client_state<I>(
    ibc_ctx: &I,
    request: &QueryChannelClientStateRequest,
) -> Result<QueryChannelClientStateResponse, QueryError>
where
    I: QueryContext,
{
    let channel_end_path = ChannelEndPath::new(&request.port_id, &request.channel_id);

    let channel_end = ibc_ctx.channel_end(&channel_end_path)?;

    let connection_end = channel_end
        .connection_hops()
        .first()
        .map(|connection_id| ibc_ctx.connection_end(connection_id))
        .ok_or_else(|| {
            QueryError::proof_not_found(format!(
                "Channel {} does not have a connection",
                request.channel_id
            ))
        })??;

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

    Ok(QueryChannelClientStateResponse::new(
        IdentifiedClientState::new(connection_end.client_id().clone(), client_state.into()),
        proof,
        proof_height,
    ))
}

/// Queries for the consensus state associated with a channel by the given
/// target height, channel and port ids
pub fn query_channel_consensus_state<I>(
    ibc_ctx: &I,
    request: &QueryChannelConsensusStateRequest,
) -> Result<QueryChannelConsensusStateResponse, QueryError>
where
    I: QueryContext,
    ConsensusStateRef<I>: Into<Any>,
{
    let channel_end_path = ChannelEndPath::new(&request.port_id, &request.channel_id);

    let channel_end = ibc_ctx.channel_end(&channel_end_path)?;

    let connection_end = channel_end
        .connection_hops()
        .first()
        .map(|connection_id| ibc_ctx.connection_end(connection_id))
        .ok_or_else(|| {
            QueryError::proof_not_found(format!(
                "Channel {} does not have a connection",
                request.channel_id
            ))
        })??;

    let consensus_path = ClientConsensusStatePath::new(
        connection_end.client_id().clone(),
        request.consensus_height.revision_number(),
        request.consensus_height.revision_height(),
    );
    let client_val_ctx = ibc_ctx.get_client_validation_context();

    let consensus_state = client_val_ctx.consensus_state(&consensus_path)?;

    let proof_height = match request.query_height {
        Some(height) => height,
        None => ibc_ctx.host_height()?,
    };

    let proof = ibc_ctx
        .get_proof(
            proof_height,
            &Path::ClientConsensusState(consensus_path.clone()),
        )
        .ok_or_else(|| {
            QueryError::proof_not_found(format!(
                "Proof not found for client consensus state path: {consensus_path:?}"
            ))
        })?;

    Ok(QueryChannelConsensusStateResponse::new(
        consensus_state.into(),
        connection_end.client_id().clone(),
        proof,
        proof_height,
    ))
}

/// Queries for the packet commitment associated with a channel by the given
/// sequence, channel and port ids
pub fn query_packet_commitment<I>(
    ibc_ctx: &I,
    request: &QueryPacketCommitmentRequest,
) -> Result<QueryPacketCommitmentResponse, QueryError>
where
    I: ValidationContext + ProvableContext,
{
    let commitment_path =
        CommitmentPath::new(&request.port_id, &request.channel_id, request.sequence);

    let packet_commitment_data = ibc_ctx.get_packet_commitment(&commitment_path)?;

    let proof_height = match request.query_height {
        Some(height) => height,
        None => ibc_ctx.host_height()?,
    };

    let proof = ibc_ctx
        .get_proof(proof_height, &Path::Commitment(commitment_path.clone()))
        .ok_or_else(|| {
            QueryError::proof_not_found(format!(
                "Proof not found for packet commitment path: {commitment_path:?}"
            ))
        })?;

    Ok(QueryPacketCommitmentResponse::new(
        packet_commitment_data,
        proof,
        proof_height,
    ))
}

/// Queries for all packet commitments associated with a channel
pub fn query_packet_commitments<I>(
    ibc_ctx: &I,
    request: &QueryPacketCommitmentsRequest,
) -> Result<QueryPacketCommitmentsResponse, QueryError>
where
    I: QueryContext,
{
    let channel_end_path = ChannelEndPath::new(&request.port_id, &request.channel_id);

    let commitments = ibc_ctx
        .packet_commitments(&channel_end_path)?
        .into_iter()
        .map(Into::into)
        .collect();

    Ok(QueryPacketCommitmentsResponse::new(
        commitments,
        ibc_ctx.host_height()?,
        None,
    ))
}

/// Queries for the packet receipt associated with a channel by the given
/// sequence, channel and port ids
pub fn query_packet_receipt<I>(
    ibc_ctx: &I,
    request: &QueryPacketReceiptRequest,
) -> Result<QueryPacketReceiptResponse, QueryError>
where
    I: ValidationContext + ProvableContext,
{
    let receipt_path = ReceiptPath::new(&request.port_id, &request.channel_id, request.sequence);

    // Receipt only has one enum
    // Unreceived packets are not stored
    let packet_receipt_data = ibc_ctx.get_packet_receipt(&receipt_path);

    let proof_height = match request.query_height {
        Some(height) => height,
        None => ibc_ctx.host_height()?,
    };

    let proof = ibc_ctx
        .get_proof(proof_height, &Path::Receipt(receipt_path.clone()))
        .ok_or_else(|| {
            QueryError::proof_not_found(format!(
                "Proof not found for packet receipt path: {receipt_path:?}"
            ))
        })?;

    Ok(QueryPacketReceiptResponse::new(
        packet_receipt_data.is_ok(),
        proof,
        proof_height,
    ))
}

/// Queries for the packet acknowledgement associated with a channel by the
/// given sequence, channel and port ids
pub fn query_packet_acknowledgement<I>(
    ibc_ctx: &I,
    request: &QueryPacketAcknowledgementRequest,
) -> Result<QueryPacketAcknowledgementResponse, QueryError>
where
    I: ValidationContext + ProvableContext,
{
    let acknowledgement_path =
        AckPath::new(&request.port_id, &request.channel_id, request.sequence);

    let packet_acknowledgement_data = ibc_ctx.get_packet_acknowledgement(&acknowledgement_path)?;

    let proof_height = match request.query_height {
        Some(height) => height,
        None => ibc_ctx.host_height()?,
    };

    let proof = ibc_ctx
        .get_proof(proof_height, &Path::Ack(acknowledgement_path.clone()))
        .ok_or_else(|| {
            QueryError::proof_not_found(format!(
                "Proof not found for packet acknowledgement path: {acknowledgement_path:?}"
            ))
        })?;

    Ok(QueryPacketAcknowledgementResponse::new(
        packet_acknowledgement_data,
        proof,
        proof_height,
    ))
}

/// Queries for all packet acknowledgements associated with a channel
pub fn query_packet_acknowledgements<I>(
    ibc_ctx: &I,
    request: &QueryPacketAcknowledgementsRequest,
) -> Result<QueryPacketAcknowledgementsResponse, QueryError>
where
    I: QueryContext,
{
    let commitment_sequences = request
        .packet_commitment_sequences
        .iter()
        .copied()
        .map(Into::into);

    let channel_end_path = ChannelEndPath::new(&request.port_id, &request.channel_id);

    let acknowledgements = ibc_ctx
        .packet_acknowledgements(&channel_end_path, commitment_sequences)?
        .into_iter()
        .map(Into::into)
        .collect();

    Ok(QueryPacketAcknowledgementsResponse::new(
        acknowledgements,
        ibc_ctx.host_height()?,
        None,
    ))
}

/// Queries for all unreceived packets associated with a channel
pub fn query_unreceived_packets<I>(
    ibc_ctx: &I,
    request: &QueryUnreceivedPacketsRequest,
) -> Result<QueryUnreceivedPacketsResponse, QueryError>
where
    I: QueryContext,
{
    let sequences = request
        .packet_commitment_sequences
        .iter()
        .copied()
        .map(Into::into);

    let channel_end_path = ChannelEndPath::new(&request.port_id, &request.channel_id);

    let unreceived_packets = ibc_ctx.unreceived_packets(&channel_end_path, sequences)?;

    Ok(QueryUnreceivedPacketsResponse::new(
        unreceived_packets,
        ibc_ctx.host_height()?,
    ))
}

/// Queries for all unreceived acknowledgements associated with a channel
pub fn query_unreceived_acks<I>(
    ibc_ctx: &I,
    request: &QueryUnreceivedAcksRequest,
) -> Result<QueryUnreceivedAcksResponse, QueryError>
where
    I: QueryContext,
{
    let sequences = request.packet_ack_sequences.iter().copied().map(Into::into);

    let channel_end_path = ChannelEndPath::new(&request.port_id, &request.channel_id);

    let unreceived_acks = ibc_ctx.unreceived_acks(&channel_end_path, sequences)?;

    Ok(QueryUnreceivedAcksResponse::new(
        unreceived_acks,
        ibc_ctx.host_height()?,
    ))
}

/// Queries for the next sequence to send for the channel specified
/// in the `request`.
pub fn query_next_sequence_send<I>(
    ibc_ctx: &I,
    request: &QueryNextSequenceSendRequest,
) -> Result<QueryNextSequenceSendResponse, QueryError>
where
    I: ValidationContext + ProvableContext,
{
    let next_seq_send_path = SeqSendPath::new(&request.port_id, &request.channel_id);

    let next_sequence_send = ibc_ctx.get_next_sequence_send(&next_seq_send_path)?;

    let proof_height = match request.query_height {
        Some(height) => height,
        None => ibc_ctx.host_height()?,
    };

    let proof = ibc_ctx
        .get_proof(proof_height, &Path::SeqSend(next_seq_send_path))
        .ok_or_else(|| {
            QueryError::proof_not_found(format!(
                "Next sequence send proof not found for channel {}",
                request.channel_id
            ))
        })?;

    Ok(QueryNextSequenceSendResponse::new(
        next_sequence_send,
        proof,
        proof_height,
    ))
}

/// Queries for the next sequence receive associated with a channel
pub fn query_next_sequence_receive<I>(
    ibc_ctx: &I,
    request: &QueryNextSequenceReceiveRequest,
) -> Result<QueryNextSequenceReceiveResponse, QueryError>
where
    I: ValidationContext + ProvableContext,
{
    let next_seq_recv_path = SeqRecvPath::new(&request.port_id, &request.channel_id);

    let next_sequence_recv = ibc_ctx.get_next_sequence_recv(&next_seq_recv_path)?;

    let proof_height = match request.query_height {
        Some(height) => height,
        None => ibc_ctx.host_height()?,
    };

    let proof = ibc_ctx
        .get_proof(proof_height, &Path::SeqRecv(next_seq_recv_path))
        .ok_or_else(|| {
            QueryError::proof_not_found(format!(
                "Next sequence receive proof not found for channel {}",
                request.channel_id
            ))
        })?;

    Ok(QueryNextSequenceReceiveResponse::new(
        next_sequence_recv,
        proof,
        proof_height,
    ))
}
