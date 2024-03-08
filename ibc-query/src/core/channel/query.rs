//! Provides utility functions for querying IBC channel states.

use alloc::format;
use core::str::FromStr;

use ibc::core::client::context::ClientValidationContext;
use ibc::core::client::types::Height;
use ibc::core::host::types::identifiers::{ChannelId, ConnectionId, PortId, Sequence};
use ibc::core::host::types::path::{
    AckPath, ChannelEndPath, ClientConsensusStatePath, ClientStatePath, CommitmentPath, Path,
    ReceiptPath, SeqRecvPath, SeqSendPath,
};
use ibc::core::host::{ConsensusStateRef, ValidationContext};
use ibc_proto::google::protobuf::Any;
use ibc_proto::ibc::core::channel::v1::{
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
use ibc_proto::ibc::core::client::v1::IdentifiedClientState;

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
    let channel_id = ChannelId::from_str(request.channel_id.as_str())?;

    let port_id = PortId::from_str(request.port_id.as_str())?;

    let channel_end_path = ChannelEndPath::new(&port_id, &channel_id);

    let channel_end = ibc_ctx.channel_end(&channel_end_path)?;

    let current_height = ibc_ctx.host_height()?;
    let proof = ibc_ctx
        .get_proof(current_height, &Path::ChannelEnd(channel_end_path.clone()))
        .ok_or(QueryError::ProofNotFound {
            description: format!(
                "Proof not found for channel end path {:?}",
                channel_end_path
            ),
        })?;

    Ok(QueryChannelResponse {
        channel: Some(channel_end.into()),
        proof,
        proof_height: Some(current_height.into()),
    })
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

    Ok(QueryChannelsResponse {
        channels: channel_ends.into_iter().map(Into::into).collect(),
        height: Some(ibc_ctx.host_height()?.into()),
        // no support for pagination yet
        pagination: None,
    })
}

/// Queries for all channels associated with a given connection
pub fn query_connection_channels<I>(
    ibc_ctx: &I,
    request: &QueryConnectionChannelsRequest,
) -> Result<QueryConnectionChannelsResponse, QueryError>
where
    I: QueryContext,
{
    let connection_id = ConnectionId::from_str(request.connection.as_str())?;

    let all_channel_ends = ibc_ctx.channel_ends()?;

    let connection_channel_ends = all_channel_ends
        .into_iter()
        .filter(|channel_end| {
            channel_end
                .channel_end
                .connection_hops()
                .iter()
                .any(|connection_hop| connection_hop == &connection_id)
        })
        .map(Into::into)
        .collect();

    Ok(QueryConnectionChannelsResponse {
        channels: connection_channel_ends,
        height: Some(ibc_ctx.host_height()?.into()),
        // no support for pagination yet
        pagination: None,
    })
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
    let channel_id = ChannelId::from_str(request.channel_id.as_str())?;

    let port_id = PortId::from_str(request.port_id.as_str())?;

    let channel_end_path = ChannelEndPath::new(&port_id, &channel_id);

    let channel_end = ibc_ctx.channel_end(&channel_end_path)?;

    let connection_end = channel_end
        .connection_hops()
        .first()
        .map(|connection_id| ibc_ctx.connection_end(connection_id))
        .ok_or(QueryError::ProofNotFound {
            description: format!("Channel {} does not have a connection", channel_id),
        })??;

    let client_val_ctx = ibc_ctx.get_client_validation_context();

    let client_state = client_val_ctx.client_state(connection_end.client_id())?;

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

    Ok(QueryChannelClientStateResponse {
        identified_client_state: Some(IdentifiedClientState {
            client_id: connection_end.client_id().as_str().into(),
            client_state: Some(client_state.into()),
        }),
        proof,
        proof_height: Some(current_height.into()),
    })
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
    let channel_id = ChannelId::from_str(request.channel_id.as_str())?;

    let port_id = PortId::from_str(request.port_id.as_str())?;

    let height = Height::new(request.revision_number, request.revision_height)?;

    let channel_end_path = ChannelEndPath::new(&port_id, &channel_id);

    let channel_end = ibc_ctx.channel_end(&channel_end_path)?;

    let connection_end = channel_end
        .connection_hops()
        .first()
        .map(|connection_id| ibc_ctx.connection_end(connection_id))
        .ok_or(QueryError::ProofNotFound {
            description: format!("Channel {} does not have a connection", channel_id),
        })??;

    let consensus_path = ClientConsensusStatePath::new(
        connection_end.client_id().clone(),
        height.revision_number(),
        height.revision_height(),
    );
    let client_val_ctx = ibc_ctx.get_client_validation_context();

    let consensus_state = client_val_ctx.consensus_state(&consensus_path)?;

    let current_height = ibc_ctx.host_height()?;

    let proof = ibc_ctx
        .get_proof(
            current_height,
            &Path::ClientConsensusState(consensus_path.clone()),
        )
        .ok_or(QueryError::ProofNotFound {
            description: format!(
                "Proof not found for client consensus state path: {:?}",
                consensus_path
            ),
        })?;

    Ok(QueryChannelConsensusStateResponse {
        client_id: connection_end.client_id().as_str().into(),
        consensus_state: Some(consensus_state.into()),
        proof,
        proof_height: Some(current_height.into()),
    })
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
    let channel_id = ChannelId::from_str(request.channel_id.as_str())?;

    let port_id = PortId::from_str(request.port_id.as_str())?;

    let sequence = Sequence::from(request.sequence);

    let commitment_path = CommitmentPath::new(&port_id, &channel_id, sequence);

    let packet_commitment_data = ibc_ctx.get_packet_commitment(&commitment_path)?;

    let current_height = ibc_ctx.host_height()?;

    let proof = ibc_ctx
        .get_proof(current_height, &Path::Commitment(commitment_path.clone()))
        .ok_or(QueryError::ProofNotFound {
            description: format!(
                "Proof not found for packet commitment path: {:?}",
                commitment_path
            ),
        })?;

    Ok(QueryPacketCommitmentResponse {
        commitment: packet_commitment_data.into_vec(),
        proof,
        proof_height: Some(current_height.into()),
    })
}

/// Queries for all packet commitments associated with a channel
pub fn query_packet_commitments<I>(
    ibc_ctx: &I,
    request: &QueryPacketCommitmentsRequest,
) -> Result<QueryPacketCommitmentsResponse, QueryError>
where
    I: QueryContext,
{
    let channel_id = ChannelId::from_str(request.channel_id.as_str())?;

    let port_id = PortId::from_str(request.port_id.as_str())?;

    let channel_end_path = ChannelEndPath::new(&port_id, &channel_id);

    let commitments = ibc_ctx
        .packet_commitments(&channel_end_path)?
        .into_iter()
        .map(Into::into)
        .collect();

    Ok(QueryPacketCommitmentsResponse {
        commitments,
        height: Some(ibc_ctx.host_height()?.into()),
        // no support for pagination yet
        pagination: None,
    })
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
    let channel_id = ChannelId::from_str(request.channel_id.as_str())?;

    let port_id = PortId::from_str(request.port_id.as_str())?;

    let sequence = Sequence::from(request.sequence);

    let receipt_path = ReceiptPath::new(&port_id, &channel_id, sequence);

    // Receipt only has one enum
    // Unreceived packets are not stored
    let packet_receipt_data = ibc_ctx.get_packet_receipt(&receipt_path);

    let current_height = ibc_ctx.host_height()?;

    let proof = ibc_ctx
        .get_proof(current_height, &Path::Receipt(receipt_path.clone()))
        .ok_or(QueryError::ProofNotFound {
            description: format!(
                "Proof not found for packet receipt path: {:?}",
                receipt_path
            ),
        })?;

    Ok(QueryPacketReceiptResponse {
        received: packet_receipt_data.is_ok(),
        proof,
        proof_height: Some(current_height.into()),
    })
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
    let channel_id = ChannelId::from_str(request.channel_id.as_str())?;

    let port_id = PortId::from_str(request.port_id.as_str())?;

    let sequence = Sequence::from(request.sequence);

    let acknowledgement_path = AckPath::new(&port_id, &channel_id, sequence);

    let packet_acknowledgement_data = ibc_ctx.get_packet_acknowledgement(&acknowledgement_path)?;

    let current_height = ibc_ctx.host_height()?;

    let proof = ibc_ctx
        .get_proof(current_height, &Path::Ack(acknowledgement_path.clone()))
        .ok_or(QueryError::ProofNotFound {
            description: format!(
                "Proof not found for packet acknowledgement path: {:?}",
                acknowledgement_path
            ),
        })?;

    Ok(QueryPacketAcknowledgementResponse {
        acknowledgement: packet_acknowledgement_data.into_vec(),
        proof,
        proof_height: Some(current_height.into()),
    })
}

/// Queries for all packet acknowledgements associated with a channel
pub fn query_packet_acknowledgements<I>(
    ibc_ctx: &I,
    request: &QueryPacketAcknowledgementsRequest,
) -> Result<QueryPacketAcknowledgementsResponse, QueryError>
where
    I: QueryContext,
{
    let channel_id = ChannelId::from_str(request.channel_id.as_str())?;

    let port_id = PortId::from_str(request.port_id.as_str())?;

    let commitment_sequences = request
        .packet_commitment_sequences
        .iter()
        .copied()
        .map(Sequence::from);

    let channel_end_path = ChannelEndPath::new(&port_id, &channel_id);

    let acknowledgements = ibc_ctx
        .packet_acknowledgements(&channel_end_path, commitment_sequences)?
        .into_iter()
        .map(Into::into)
        .collect();

    Ok(QueryPacketAcknowledgementsResponse {
        acknowledgements,
        height: Some(ibc_ctx.host_height()?.into()),
        // no support for pagination yet
        pagination: None,
    })
}

/// Queries for all unreceived packets associated with a channel
pub fn query_unreceived_packets<I>(
    ibc_ctx: &I,
    request: &QueryUnreceivedPacketsRequest,
) -> Result<QueryUnreceivedPacketsResponse, QueryError>
where
    I: QueryContext,
{
    let channel_id = ChannelId::from_str(request.channel_id.as_str())?;

    let port_id = PortId::from_str(request.port_id.as_str())?;

    let sequences = request
        .packet_commitment_sequences
        .iter()
        .copied()
        .map(Sequence::from);

    let channel_end_path = ChannelEndPath::new(&port_id, &channel_id);

    let unreceived_packets = ibc_ctx.unreceived_packets(&channel_end_path, sequences)?;

    Ok(QueryUnreceivedPacketsResponse {
        sequences: unreceived_packets.into_iter().map(Into::into).collect(),
        height: Some(ibc_ctx.host_height()?.into()),
    })
}

/// Queries for all unreceived acknowledgements associated with a channel
pub fn query_unreceived_acks<I>(
    ibc_ctx: &I,
    request: &QueryUnreceivedAcksRequest,
) -> Result<QueryUnreceivedAcksResponse, QueryError>
where
    I: QueryContext,
{
    let channel_id = ChannelId::from_str(request.channel_id.as_str())?;

    let port_id = PortId::from_str(request.port_id.as_str())?;

    let sequences = request
        .packet_ack_sequences
        .iter()
        .copied()
        .map(Sequence::from);

    let channel_end_path = ChannelEndPath::new(&port_id, &channel_id);

    let unreceived_acks = ibc_ctx.unreceived_acks(&channel_end_path, sequences)?;

    Ok(QueryUnreceivedAcksResponse {
        sequences: unreceived_acks.into_iter().map(Into::into).collect(),
        height: Some(ibc_ctx.host_height()?.into()),
    })
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
    let channel_id = ChannelId::from_str(request.channel_id.as_str())?;

    let port_id = PortId::from_str(request.port_id.as_str())?;

    let next_seq_send_path = SeqSendPath::new(&port_id, &channel_id);

    let next_sequence_send = ibc_ctx.get_next_sequence_send(&next_seq_send_path)?;

    let current_height = ibc_ctx.host_height()?;

    let proof = ibc_ctx
        .get_proof(current_height, &Path::SeqSend(next_seq_send_path))
        .ok_or(QueryError::ProofNotFound {
            description: format!(
                "Next sequence send proof not found for channel {}",
                channel_id
            ),
        })?;

    Ok(QueryNextSequenceSendResponse {
        next_sequence_send: next_sequence_send.into(),
        proof,
        proof_height: Some(current_height.into()),
    })
}

/// Queries for the next sequence receive associated with a channel
pub fn query_next_sequence_receive<I>(
    ibc_ctx: &I,
    request: &QueryNextSequenceReceiveRequest,
) -> Result<QueryNextSequenceReceiveResponse, QueryError>
where
    I: ValidationContext + ProvableContext,
{
    let channel_id = ChannelId::from_str(request.channel_id.as_str())?;

    let port_id = PortId::from_str(request.port_id.as_str())?;

    let next_seq_recv_path = SeqRecvPath::new(&port_id, &channel_id);

    let next_sequence_recv = ibc_ctx.get_next_sequence_recv(&next_seq_recv_path)?;

    let current_height = ibc_ctx.host_height()?;

    let proof = ibc_ctx
        .get_proof(current_height, &Path::SeqRecv(next_seq_recv_path))
        .ok_or(QueryError::ProofNotFound {
            description: format!(
                "Next sequence receive proof not found for channel {}",
                channel_id
            ),
        })?;

    Ok(QueryNextSequenceReceiveResponse {
        next_sequence_receive: next_sequence_recv.into(),
        proof,
        proof_height: Some(current_height.into()),
    })
}
