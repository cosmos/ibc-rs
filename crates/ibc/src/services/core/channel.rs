use std::boxed::Box;

use ibc_proto::google::protobuf::Any;
use ibc_proto::ibc::core::channel::v1::query_server::Query as ChannelQuery;
use ibc_proto::ibc::core::channel::v1::{
    PacketState, QueryChannelClientStateRequest, QueryChannelClientStateResponse,
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
use tonic::{Request, Response, Status};
use tracing::trace;

use crate::core::ics04_channel::packet::Sequence;
use crate::core::ics24_host::identifier::{ChannelId, ConnectionId, PortId};
use crate::core::ics24_host::path::{
    AckPath, ChannelEndPath, ClientConsensusStatePath, ClientStatePath, CommitmentPath, Path,
    ReceiptPath, SeqRecvPath, SeqSendPath,
};
use crate::core::ValidationContext;
use crate::prelude::*;
use crate::services::core::context::QueryContext;
use crate::Height;

pub struct ChannelQueryService<I>
where
    I: QueryContext + Send + Sync + 'static,
    <I as ValidationContext>::AnyClientState: Into<Any>,
    <I as ValidationContext>::AnyConsensusState: Into<Any>,
{
    ibc_context: I,
}

impl<I> ChannelQueryService<I>
where
    I: QueryContext + Send + Sync + 'static,
    <I as ValidationContext>::AnyClientState: Into<Any>,
    <I as ValidationContext>::AnyConsensusState: Into<Any>,
{
    pub fn new(ibc_context: I) -> Self {
        Self { ibc_context }
    }
}

#[tonic::async_trait]
impl<I> ChannelQuery for ChannelQueryService<I>
where
    I: QueryContext + Send + Sync + 'static,
    <I as ValidationContext>::AnyClientState: Into<Any>,
    <I as ValidationContext>::AnyConsensusState: Into<Any>,
{
    async fn channel(
        &self,
        request: Request<QueryChannelRequest>,
    ) -> Result<Response<QueryChannelResponse>, Status> {
        let request_ref = request.get_ref();

        trace!("Got channel request: {:?}", &request_ref);

        let channel_id = ChannelId::from_str(request_ref.channel_id.as_str())?;

        let port_id = PortId::from_str(request_ref.port_id.as_str())?;

        let channel_end_path = ChannelEndPath::new(&port_id, &channel_id);

        let channel_end = self.ibc_context.channel_end(&channel_end_path)?;

        let current_height = self.ibc_context.host_height()?;

        let proof = self
            .ibc_context
            .get_proof(current_height, &Path::ChannelEnd(channel_end_path))
            .ok_or_else(|| {
                Status::not_found(format!(
                    "Channel end proof not found for channel {}",
                    channel_id
                ))
            })?;

        Ok(Response::new(QueryChannelResponse {
            channel: Some(channel_end.into()),
            proof,
            proof_height: Some(current_height.into()),
        }))
    }

    async fn channels(
        &self,
        request: Request<QueryChannelsRequest>,
    ) -> Result<Response<QueryChannelsResponse>, Status> {
        let request_ref = request.get_ref();

        let channel_ends = self.ibc_context.channel_ends()?;

        trace!("Got channels request: {:?}", &request_ref);

        Ok(Response::new(QueryChannelsResponse {
            channels: channel_ends.into_iter().map(Into::into).collect(),
            pagination: None,
            height: Some(self.ibc_context.host_height()?.into()),
        }))
    }

    async fn connection_channels(
        &self,
        request: Request<QueryConnectionChannelsRequest>,
    ) -> Result<Response<QueryConnectionChannelsResponse>, Status> {
        let request_ref = request.get_ref();

        trace!("Got connection channels request: {:?}", &request_ref);

        let connection_id = ConnectionId::from_str(request_ref.connection.as_str())?;

        let all_channel_ends = self.ibc_context.channel_ends()?;

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

        Ok(Response::new(QueryConnectionChannelsResponse {
            channels: connection_channel_ends,
            pagination: None,
            height: Some(self.ibc_context.host_height()?.into()),
        }))
    }

    async fn channel_client_state(
        &self,
        request: Request<QueryChannelClientStateRequest>,
    ) -> Result<Response<QueryChannelClientStateResponse>, Status> {
        let request_ref = request.get_ref();

        trace!("Got channel client state request: {:?}", &request_ref);

        let channel_id = ChannelId::from_str(request_ref.channel_id.as_str())?;

        let port_id = PortId::from_str(request_ref.port_id.as_str())?;

        let channel_end_path = ChannelEndPath::new(&port_id, &channel_id);

        let channel_end = self.ibc_context.channel_end(&channel_end_path)?;

        let connection_end = channel_end
            .connection_hops()
            .first()
            .map(|connection_id| self.ibc_context.connection_end(connection_id))
            .ok_or_else(|| {
                Status::not_found(format!("Channel {} has no connection hops", channel_id))
            })??;

        let client_state = self.ibc_context.client_state(connection_end.client_id())?;

        let current_height = self.ibc_context.host_height()?;

        let proof = self
            .ibc_context
            .get_proof(
                current_height,
                &Path::ClientState(ClientStatePath::new(connection_end.client_id())),
            )
            .ok_or_else(|| {
                Status::not_found(format!(
                    "Client state proof not found for client {}",
                    connection_end.client_id()
                ))
            })?;

        Ok(Response::new(QueryChannelClientStateResponse {
            identified_client_state: Some(IdentifiedClientState {
                client_id: connection_end.client_id().as_str().into(),
                client_state: Some(client_state.into()),
            }),
            proof,
            proof_height: Some(current_height.into()),
        }))
    }

    async fn channel_consensus_state(
        &self,
        request: Request<QueryChannelConsensusStateRequest>,
    ) -> Result<Response<QueryChannelConsensusStateResponse>, Status> {
        let request_ref = request.get_ref();

        trace!("Got channel consensus state request: {:?}", &request_ref);

        let channel_id = ChannelId::from_str(request_ref.channel_id.as_str())?;

        let port_id = PortId::from_str(request_ref.port_id.as_str())?;

        let height = Height::new(request_ref.revision_number, request_ref.revision_height)
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

        let channel_end_path = ChannelEndPath::new(&port_id, &channel_id);

        let channel_end = self.ibc_context.channel_end(&channel_end_path)?;

        let connection_end = channel_end
            .connection_hops()
            .first()
            .map(|connection_id| self.ibc_context.connection_end(connection_id))
            .ok_or_else(|| {
                Status::not_found(format!("Channel {} has no connection hops", channel_id))
            })??;

        let consensus_path = ClientConsensusStatePath::new(connection_end.client_id(), &height);

        let consensus_state = self.ibc_context.consensus_state(&consensus_path)?;

        let current_height = self.ibc_context.host_height()?;

        let proof = self
            .ibc_context
            .get_proof(current_height, &Path::ClientConsensusState(consensus_path))
            .ok_or_else(|| {
                Status::not_found(format!(
                    "Consensus state proof not found for client {}",
                    connection_end.client_id()
                ))
            })?;

        Ok(Response::new(QueryChannelConsensusStateResponse {
            client_id: connection_end.client_id().as_str().into(),
            consensus_state: Some(consensus_state.into()),
            proof,
            proof_height: Some(current_height.into()),
        }))
    }

    async fn packet_commitment(
        &self,
        request: Request<QueryPacketCommitmentRequest>,
    ) -> Result<Response<QueryPacketCommitmentResponse>, Status> {
        let request_ref = request.get_ref();

        trace!("Got packet commitment request: {:?}", &request_ref);

        let channel_id = ChannelId::from_str(request_ref.channel_id.as_str())?;

        let port_id = PortId::from_str(request_ref.port_id.as_str())?;

        let sequence = Sequence::from(request_ref.sequence);

        let commitment_path = CommitmentPath::new(&port_id, &channel_id, sequence);

        let packet_commitment_data = self.ibc_context.get_packet_commitment(&commitment_path)?;

        let current_height = self.ibc_context.host_height()?;

        let proof = self
            .ibc_context
            .get_proof(current_height, &Path::Commitment(commitment_path))
            .ok_or_else(|| {
                Status::not_found(format!(
                    "Packet commitment proof not found for channel {}",
                    channel_id
                ))
            })?;

        Ok(Response::new(QueryPacketCommitmentResponse {
            commitment: packet_commitment_data.into_vec(),
            proof,
            proof_height: Some(current_height.into()),
        }))
    }

    async fn packet_commitments(
        &self,
        request: Request<QueryPacketCommitmentsRequest>,
    ) -> Result<Response<QueryPacketCommitmentsResponse>, Status> {
        let request_ref = request.get_ref();

        trace!("Got packet commitments request: {:?}", &request_ref);

        let channel_id = ChannelId::from_str(request_ref.channel_id.as_str())?;

        let port_id = PortId::from_str(request_ref.port_id.as_str())?;

        let channel_end_path = ChannelEndPath::new(&port_id, &channel_id);

        let commitments = self
            .ibc_context
            .packet_commitments(&channel_end_path)?
            .into_iter()
            .map(|path| {
                self.ibc_context
                    .get_packet_commitment(&path)
                    .map(|commitment| PacketState {
                        port_id: path.port_id.as_str().into(),
                        channel_id: path.channel_id.as_str().into(),
                        sequence: path.sequence.into(),
                        data: commitment.into_vec(),
                    })
            })
            .collect::<Result<_, _>>()?;

        Ok(Response::new(QueryPacketCommitmentsResponse {
            commitments,
            pagination: None,
            height: Some(self.ibc_context.host_height()?.into()),
        }))
    }

    async fn packet_receipt(
        &self,
        request: Request<QueryPacketReceiptRequest>,
    ) -> Result<Response<QueryPacketReceiptResponse>, Status> {
        let request_ref = request.get_ref();

        trace!("Got packet receipt request: {:?}", &request_ref);

        let channel_id = ChannelId::from_str(request_ref.channel_id.as_str())?;

        let port_id = PortId::from_str(request_ref.port_id.as_str())?;

        let sequence = Sequence::from(request_ref.sequence);

        let receipt_path = ReceiptPath::new(&port_id, &channel_id, sequence);

        // Receipt only has one enum
        // Unreceived packets are not stored
        let packet_receipt_data = self.ibc_context.get_packet_receipt(&receipt_path);

        let current_height = self.ibc_context.host_height()?;

        let proof = self
            .ibc_context
            .get_proof(current_height, &Path::Receipt(receipt_path))
            .ok_or_else(|| {
                Status::not_found(format!(
                    "Packet receipt proof not found for channel {}",
                    channel_id
                ))
            })?;

        Ok(Response::new(QueryPacketReceiptResponse {
            received: packet_receipt_data.is_ok(),
            proof,
            proof_height: Some(current_height.into()),
        }))
    }

    async fn packet_acknowledgement(
        &self,
        request: Request<QueryPacketAcknowledgementRequest>,
    ) -> Result<Response<QueryPacketAcknowledgementResponse>, Status> {
        let request_ref = request.get_ref();

        trace!("Got packet acknowledgement request: {:?}", &request_ref);

        let channel_id = ChannelId::from_str(request_ref.channel_id.as_str())?;

        let port_id = PortId::from_str(request_ref.port_id.as_str())?;

        let sequence = Sequence::from(request_ref.sequence);

        let acknowledgement_path = AckPath::new(&port_id, &channel_id, sequence);

        let packet_acknowledgement_data = self
            .ibc_context
            .get_packet_acknowledgement(&acknowledgement_path)?;

        let current_height = self.ibc_context.host_height()?;

        let proof = self
            .ibc_context
            .get_proof(current_height, &Path::Ack(acknowledgement_path))
            .ok_or_else(|| {
                Status::not_found(format!(
                    "Packet acknowledgement proof not found for channel {}",
                    channel_id
                ))
            })?;

        Ok(Response::new(QueryPacketAcknowledgementResponse {
            acknowledgement: packet_acknowledgement_data.into_vec(),
            proof,
            proof_height: Some(current_height.into()),
        }))
    }

    /// Returns all the acknowledgements if sequences is omitted.
    async fn packet_acknowledgements(
        &self,
        request: Request<QueryPacketAcknowledgementsRequest>,
    ) -> Result<Response<QueryPacketAcknowledgementsResponse>, Status> {
        let request_ref = request.get_ref();

        trace!("Got packet acknowledgements request: {:?}", &request_ref);

        let channel_id = ChannelId::from_str(request_ref.channel_id.as_str())?;

        let port_id = PortId::from_str(request_ref.port_id.as_str())?;

        let commitment_sequences = request_ref
            .packet_commitment_sequences
            .iter()
            .copied()
            .map(Sequence::from);

        let channel_end_path = ChannelEndPath::new(&port_id, &channel_id);

        let acknowledgements = self
            .ibc_context
            .packet_acknowledgements(&channel_end_path, commitment_sequences)?
            .into_iter()
            .map(|path| {
                self.ibc_context
                    .get_packet_acknowledgement(&path)
                    .map(|acknowledgement| PacketState {
                        port_id: path.port_id.as_str().into(),
                        channel_id: path.channel_id.as_str().into(),
                        sequence: path.sequence.into(),
                        data: acknowledgement.into_vec(),
                    })
            })
            .collect::<Result<_, _>>()?;

        Ok(Response::new(QueryPacketAcknowledgementsResponse {
            acknowledgements,
            pagination: None,
            height: Some(self.ibc_context.host_height()?.into()),
        }))
    }

    async fn unreceived_packets(
        &self,
        request: Request<QueryUnreceivedPacketsRequest>,
    ) -> Result<Response<QueryUnreceivedPacketsResponse>, Status> {
        let request_ref = request.get_ref();

        trace!("Got unreceived packets request: {:?}", &request_ref);

        let channel_id = ChannelId::from_str(request_ref.channel_id.as_str())?;

        let port_id = PortId::from_str(request_ref.port_id.as_str())?;

        let sequences = request_ref
            .packet_commitment_sequences
            .iter()
            .copied()
            .map(Sequence::from);

        let channel_end_path = ChannelEndPath::new(&port_id, &channel_id);

        let unreceived_packets = self
            .ibc_context
            .unreceived_packets(&channel_end_path, sequences)?;

        Ok(Response::new(QueryUnreceivedPacketsResponse {
            sequences: unreceived_packets.into_iter().map(Into::into).collect(),
            height: Some(self.ibc_context.host_height()?.into()),
        }))
    }

    /// Returns all the unreceived acknowledgements if sequences is omitted.
    async fn unreceived_acks(
        &self,
        request: Request<QueryUnreceivedAcksRequest>,
    ) -> Result<Response<QueryUnreceivedAcksResponse>, Status> {
        let request_ref = request.get_ref();

        trace!("Got unreceived acks request: {:?}", &request_ref);

        let channel_id = ChannelId::from_str(request_ref.channel_id.as_str())?;

        let port_id = PortId::from_str(request_ref.port_id.as_str())?;

        let sequences = request_ref
            .packet_ack_sequences
            .iter()
            .copied()
            .map(Sequence::from);

        let channel_end_path = ChannelEndPath::new(&port_id, &channel_id);

        let unreceived_acks = self
            .ibc_context
            .unreceived_acks(&channel_end_path, sequences)?;

        Ok(Response::new(QueryUnreceivedAcksResponse {
            sequences: unreceived_acks.into_iter().map(Into::into).collect(),
            height: Some(self.ibc_context.host_height()?.into()),
        }))
    }

    async fn next_sequence_receive(
        &self,
        request: Request<QueryNextSequenceReceiveRequest>,
    ) -> Result<Response<QueryNextSequenceReceiveResponse>, Status> {
        let request_ref = request.get_ref();

        trace!("Got next sequence receive request: {:?}", &request_ref);

        let channel_id = ChannelId::from_str(request_ref.channel_id.as_str())?;

        let port_id = PortId::from_str(request_ref.port_id.as_str())?;

        let next_seq_recv_path = SeqRecvPath::new(&port_id, &channel_id);

        let next_sequence_recv = self
            .ibc_context
            .get_next_sequence_recv(&next_seq_recv_path)?;

        let current_height = self.ibc_context.host_height()?;

        let proof = self
            .ibc_context
            .get_proof(current_height, &Path::SeqRecv(next_seq_recv_path))
            .ok_or_else(|| {
                Status::not_found(format!(
                    "Next sequence receive proof not found for channel {}",
                    channel_id
                ))
            })?;

        Ok(Response::new(QueryNextSequenceReceiveResponse {
            next_sequence_receive: next_sequence_recv.into(),
            proof,
            proof_height: Some(current_height.into()),
        }))
    }

    async fn next_sequence_send(
        &self,
        request: Request<QueryNextSequenceSendRequest>,
    ) -> Result<Response<QueryNextSequenceSendResponse>, Status> {
        let request_ref = request.get_ref();

        trace!("Got next sequence send request: {:?}", &request_ref);

        let channel_id = ChannelId::from_str(request_ref.channel_id.as_str())?;

        let port_id = PortId::from_str(request_ref.port_id.as_str())?;

        let next_seq_send_path = SeqSendPath::new(&port_id, &channel_id);

        let next_sequence_send = self
            .ibc_context
            .get_next_sequence_send(&next_seq_send_path)?;

        let current_height = self.ibc_context.host_height()?;

        let proof = self
            .ibc_context
            .get_proof(current_height, &Path::SeqSend(next_seq_send_path))
            .ok_or_else(|| {
                Status::not_found(format!(
                    "Next sequence send proof not found for channel {}",
                    channel_id
                ))
            })?;

        Ok(Response::new(QueryNextSequenceSendResponse {
            next_sequence_send: next_sequence_send.into(),
            proof,
            proof_height: Some(current_height.into()),
        }))
    }
}
