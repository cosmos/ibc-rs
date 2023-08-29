use ibc_proto::{
    google::protobuf::Any,
    ibc::core::{
        channel::v1::{
            query_server::Query as ChannelQuery, PacketState, QueryChannelClientStateRequest,
            QueryChannelClientStateResponse, QueryChannelConsensusStateRequest,
            QueryChannelConsensusStateResponse, QueryChannelRequest, QueryChannelResponse,
            QueryChannelsRequest, QueryChannelsResponse, QueryConnectionChannelsRequest,
            QueryConnectionChannelsResponse, QueryNextSequenceReceiveRequest,
            QueryNextSequenceReceiveResponse, QueryNextSequenceSendRequest,
            QueryNextSequenceSendResponse, QueryPacketAcknowledgementRequest,
            QueryPacketAcknowledgementResponse, QueryPacketAcknowledgementsRequest,
            QueryPacketAcknowledgementsResponse, QueryPacketCommitmentRequest,
            QueryPacketCommitmentResponse, QueryPacketCommitmentsRequest,
            QueryPacketCommitmentsResponse, QueryPacketReceiptRequest, QueryPacketReceiptResponse,
            QueryUnreceivedAcksRequest, QueryUnreceivedAcksResponse, QueryUnreceivedPacketsRequest,
            QueryUnreceivedPacketsResponse,
        },
        client::v1::IdentifiedClientState,
    },
};

use crate::{
    core::{
        ics04_channel::packet::Sequence,
        ics24_host::{
            identifier::{ChannelId, ConnectionId, PortId},
            path::{
                AckPath, ChannelEndPath, ClientConsensusStatePath, ClientStatePath, CommitmentPath,
                Path, ReceiptPath, SeqRecvPath, SeqSendPath,
            },
        },
        QueryContext, ValidationContext,
    },
    Height,
};

use core::str::FromStr;
use std::boxed::Box;
use tonic::{Request, Response, Status};
use tracing::trace;

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
        trace!("Got channel request: {:?}", request);
        let request_ref = request.get_ref();

        let channel_id = ChannelId::from_str(request_ref.channel_id.as_str()).map_err(|_| {
            Status::invalid_argument(std::format!(
                "Invalid channel id: {}",
                request_ref.channel_id
            ))
        })?;

        let port_id = PortId::from_str(request_ref.port_id.as_str()).map_err(|_| {
            Status::invalid_argument(std::format!("Invalid port id: {}", request_ref.port_id))
        })?;

        let channel_end_path = ChannelEndPath::new(&port_id, &channel_id);

        let channel_end = self
            .ibc_context
            .channel_end(&channel_end_path)
            .map_err(|_| {
                Status::not_found(std::format!(
                    "Channel end not found for channel {}",
                    channel_id
                ))
            })?;

        let current_height = self.ibc_context.host_height().map_err(|_| {
            Status::not_found(std::format!(
                "Current height not found for channel {}",
                channel_id
            ))
        })?;

        let proof = self
            .ibc_context
            .get_proof(current_height, &Path::ChannelEnd(channel_end_path))
            .ok_or_else(|| {
                Status::not_found(std::format!(
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
    /// Channels queries all the IBC channels of a chain.
    async fn channels(
        &self,
        request: Request<QueryChannelsRequest>,
    ) -> Result<Response<QueryChannelsResponse>, Status> {
        trace!("Got channels request: {:?}", request);

        let channel_ends = self
            .ibc_context
            .channel_ends()
            .map_err(|_| Status::not_found("Channel ends not found"))?;

        Ok(Response::new(QueryChannelsResponse {
            channels: channel_ends.into_iter().map(Into::into).collect(),
            pagination: None,
            height: Some(
                self.ibc_context
                    .host_height()
                    .map_err(|_| Status::not_found("Current height not found"))?
                    .into(),
            ),
        }))
    }
    /// ConnectionChannels queries all the channels associated with a connection
    /// end.
    async fn connection_channels(
        &self,
        request: Request<QueryConnectionChannelsRequest>,
    ) -> Result<Response<QueryConnectionChannelsResponse>, Status> {
        trace!("Got connection channels request: {:?}", request);

        let request_ref = request.get_ref();

        let connection_id =
            ConnectionId::from_str(request_ref.connection.as_str()).map_err(|_| {
                Status::invalid_argument(std::format!(
                    "Invalid connection id: {}",
                    request_ref.connection
                ))
            })?;

        let channel_ends = self
            .ibc_context
            .connection_channel_ends(&connection_id)
            .map_err(|_| {
                Status::not_found(std::format!(
                    "Connection channels not found for connection {}",
                    connection_id
                ))
            })?;

        Ok(Response::new(QueryConnectionChannelsResponse {
            channels: channel_ends.into_iter().map(Into::into).collect(),
            pagination: None,
            height: Some(
                self.ibc_context
                    .host_height()
                    .map_err(|_| {
                        Status::not_found(std::format!(
                            "Current height not found for connection {}",
                            connection_id
                        ))
                    })?
                    .into(),
            ),
        }))
    }
    /// ChannelClientState queries for the client state for the channel associated
    /// with the provided channel identifiers.
    async fn channel_client_state(
        &self,
        request: Request<QueryChannelClientStateRequest>,
    ) -> Result<Response<QueryChannelClientStateResponse>, Status> {
        trace!("Got channel client state request: {:?}", request);

        let request_ref = request.get_ref();

        let channel_id = ChannelId::from_str(request_ref.channel_id.as_str()).map_err(|_| {
            Status::invalid_argument(std::format!(
                "Invalid channel id: {}",
                request_ref.channel_id
            ))
        })?;

        let port_id = PortId::from_str(request_ref.port_id.as_str()).map_err(|_| {
            Status::invalid_argument(std::format!("Invalid port id: {}", request_ref.port_id))
        })?;

        let channel_end_path = ChannelEndPath::new(&port_id, &channel_id);

        let channel_end = self
            .ibc_context
            .channel_end(&channel_end_path)
            .map_err(|_| {
                Status::not_found(std::format!(
                    "Channel end not found for channel {}",
                    channel_id
                ))
            })?;

        let connection_end = channel_end
            .connection_hops()
            .first()
            .map(|connection_id| {
                self.ibc_context.connection_end(connection_id).map_err(|_| {
                    Status::not_found(std::format!(
                        "Connection end not found for connection {}",
                        connection_id
                    ))
                })
            })
            .ok_or_else(|| {
                Status::not_found(std::format!(
                    "Channel {} has no connection hops",
                    channel_id
                ))
            })??;

        let client_state = self
            .ibc_context
            .client_state(connection_end.client_id())
            .map_err(|_| {
                Status::not_found(std::format!(
                    "Client state not found for client {}",
                    connection_end.client_id()
                ))
            })?;

        let current_height = self.ibc_context.host_height().map_err(|_| {
            Status::not_found(std::format!(
                "Current height not found for client {}",
                connection_end.client_id()
            ))
        })?;

        let proof = self
            .ibc_context
            .get_proof(
                current_height,
                &Path::ClientState(ClientStatePath::new(connection_end.client_id())),
            )
            .ok_or_else(|| {
                Status::not_found(std::format!(
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
    /// ChannelConsensusState queries for the consensus state for the channel
    /// associated with the provided channel identifiers.
    async fn channel_consensus_state(
        &self,
        request: Request<QueryChannelConsensusStateRequest>,
    ) -> Result<Response<QueryChannelConsensusStateResponse>, Status> {
        trace!("Got channel consensus state request: {:?}", request);

        let request_ref = request.get_ref();

        let channel_id = ChannelId::from_str(request_ref.channel_id.as_str()).map_err(|_| {
            Status::invalid_argument(std::format!(
                "Invalid channel id: {}",
                request_ref.channel_id
            ))
        })?;

        let port_id = PortId::from_str(request_ref.port_id.as_str()).map_err(|_| {
            Status::invalid_argument(std::format!("Invalid port id: {}", request_ref.port_id))
        })?;

        let channel_end_path = ChannelEndPath::new(&port_id, &channel_id);

        let channel_end = self
            .ibc_context
            .channel_end(&channel_end_path)
            .map_err(|_| {
                Status::not_found(std::format!(
                    "Channel end not found for channel {}",
                    channel_id
                ))
            })?;

        let connection_end = channel_end
            .connection_hops()
            .first()
            .map(|connection_id| {
                self.ibc_context.connection_end(connection_id).map_err(|_| {
                    Status::not_found(std::format!(
                        "Connection end not found for connection {}",
                        connection_id
                    ))
                })
            })
            .ok_or_else(|| {
                Status::not_found(std::format!(
                    "Channel {} has no connection hops",
                    channel_id
                ))
            })??;

        let consensus_path = ClientConsensusStatePath::new(
            connection_end.client_id(),
            &Height::new(request_ref.revision_number, request_ref.revision_height).map_err(
                |_| {
                    Status::invalid_argument(std::format!(
                        "Invalid height: {}-{}",
                        request_ref.revision_number,
                        request_ref.revision_height
                    ))
                },
            )?,
        );

        let consensus_state = self
            .ibc_context
            .consensus_state(&consensus_path)
            .map_err(|_| {
                Status::not_found(std::format!(
                    "Consensus state not found for client {} and revision {}",
                    connection_end.client_id(),
                    request_ref.revision_number
                ))
            })?;

        let current_height = self.ibc_context.host_height().map_err(|_| {
            Status::not_found(std::format!(
                "Current height not found for client {}",
                connection_end.client_id()
            ))
        })?;

        let proof = self
            .ibc_context
            .get_proof(current_height, &Path::ClientConsensusState(consensus_path))
            .ok_or_else(|| {
                Status::not_found(std::format!(
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
    /// PacketCommitment queries a stored packet commitment hash.
    async fn packet_commitment(
        &self,
        request: Request<QueryPacketCommitmentRequest>,
    ) -> Result<Response<QueryPacketCommitmentResponse>, Status> {
        trace!("Got packet commitment request: {:?}", request);

        let request_ref = request.get_ref();

        let channel_id = ChannelId::from_str(request_ref.channel_id.as_str()).map_err(|_| {
            Status::invalid_argument(std::format!(
                "Invalid channel id: {}",
                request_ref.channel_id
            ))
        })?;

        let port_id = PortId::from_str(request_ref.port_id.as_str()).map_err(|_| {
            Status::invalid_argument(std::format!("Invalid port id: {}", request_ref.port_id))
        })?;

        let sequence = Sequence::from(request_ref.sequence);

        let commitment_path = CommitmentPath::new(&port_id, &channel_id, sequence);

        let packet_commitment_data = self
            .ibc_context
            .get_packet_commitment(&commitment_path)
            .map_err(|_| {
                Status::not_found(std::format!(
                    "Packet commitment not found for channel {} and sequence {}",
                    channel_id,
                    sequence
                ))
            })?;

        let current_height = self.ibc_context.host_height().map_err(|_| {
            Status::not_found(std::format!(
                "Current height not found for channel {}",
                channel_id
            ))
        })?;

        let proof = self
            .ibc_context
            .get_proof(current_height, &Path::Commitment(commitment_path))
            .ok_or_else(|| {
                Status::not_found(std::format!(
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

    /// PacketCommitments returns all the packet commitments hashes associated
    /// with a channel.
    async fn packet_commitments(
        &self,
        request: Request<QueryPacketCommitmentsRequest>,
    ) -> Result<Response<QueryPacketCommitmentsResponse>, Status> {
        trace!("Got packet commitments request: {:?}", request);

        let request_ref = request.get_ref();

        let channel_id = ChannelId::from_str(request_ref.channel_id.as_str()).map_err(|_| {
            Status::invalid_argument(std::format!(
                "Invalid channel id: {}",
                request_ref.channel_id
            ))
        })?;

        let port_id = PortId::from_str(request_ref.port_id.as_str()).map_err(|_| {
            Status::invalid_argument(std::format!("Invalid port id: {}", request_ref.port_id))
        })?;

        let channel_end_path = ChannelEndPath::new(&port_id, &channel_id);

        let commitments = self
            .ibc_context
            .packet_commitments(&channel_end_path)
            .map_err(|_| {
                Status::not_found(std::format!(
                    "Packet commitments not found for channel {}",
                    channel_id
                ))
            })?
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
                    .map_err(|_| {
                        Status::not_found(std::format!(
                            "Packet commitment not found for channel {} and sequence {}",
                            channel_id,
                            path.sequence
                        ))
                    })
            })
            .collect::<Result<_, _>>()?;

        Ok(Response::new(QueryPacketCommitmentsResponse {
            commitments,
            pagination: None,
            height: Some(
                self.ibc_context
                    .host_height()
                    .map_err(|_| {
                        Status::not_found(std::format!(
                            "Current height not found for channel {}",
                            channel_id
                        ))
                    })?
                    .into(),
            ),
        }))
    }

    /// PacketReceipt queries if a given packet sequence has been received on the
    /// queried chain
    async fn packet_receipt(
        &self,
        request: Request<QueryPacketReceiptRequest>,
    ) -> Result<Response<QueryPacketReceiptResponse>, Status> {
        let request_ref = request.get_ref();

        let channel_id = ChannelId::from_str(request_ref.channel_id.as_str()).map_err(|_| {
            Status::invalid_argument(std::format!(
                "Invalid channel id: {}",
                request_ref.channel_id
            ))
        })?;

        let port_id = PortId::from_str(request_ref.port_id.as_str()).map_err(|_| {
            Status::invalid_argument(std::format!("Invalid port id: {}", request_ref.port_id))
        })?;

        let sequence = Sequence::from(request_ref.sequence);

        let receipt_path = ReceiptPath::new(&port_id, &channel_id, sequence);

        // Receipt only has one enum
        // Unreceived packets are not stored
        let packet_receipt_data = self.ibc_context.get_packet_receipt(&receipt_path);

        let current_height = self.ibc_context.host_height().map_err(|_| {
            Status::not_found(std::format!(
                "Current height not found for channel {}",
                channel_id
            ))
        })?;

        let proof = self
            .ibc_context
            .get_proof(current_height, &Path::Receipt(receipt_path))
            .ok_or_else(|| {
                Status::not_found(std::format!(
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

    /// PacketAcknowledgement queries a stored packet acknowledgement hash.
    async fn packet_acknowledgement(
        &self,
        request: Request<QueryPacketAcknowledgementRequest>,
    ) -> Result<Response<QueryPacketAcknowledgementResponse>, Status> {
        let request_ref = request.get_ref();

        let channel_id = ChannelId::from_str(request_ref.channel_id.as_str()).map_err(|_| {
            Status::invalid_argument(std::format!(
                "Invalid channel id: {}",
                request_ref.channel_id
            ))
        })?;

        let port_id = PortId::from_str(request_ref.port_id.as_str()).map_err(|_| {
            Status::invalid_argument(std::format!("Invalid port id: {}", request_ref.port_id))
        })?;

        let sequence = Sequence::from(request_ref.sequence);

        let acknowledgement_path = AckPath::new(&port_id, &channel_id, sequence);

        let packet_acknowledgement_data = self
            .ibc_context
            .get_packet_acknowledgement(&acknowledgement_path)
            .map_err(|_| {
                Status::not_found(std::format!(
                    "Packet acknowledgement not found for channel {} and sequence {}",
                    channel_id,
                    sequence
                ))
            })?;

        let current_height = self.ibc_context.host_height().map_err(|_| {
            Status::not_found(std::format!(
                "Current height not found for channel {}",
                channel_id
            ))
        })?;

        let proof = self
            .ibc_context
            .get_proof(current_height, &Path::Ack(acknowledgement_path))
            .ok_or_else(|| {
                Status::not_found(std::format!(
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

    /// PacketAcknowledgements returns all the packet acknowledgements associated
    /// with a channel.
    async fn packet_acknowledgements(
        &self,
        request: Request<QueryPacketAcknowledgementsRequest>,
    ) -> Result<Response<QueryPacketAcknowledgementsResponse>, Status> {
        let request_ref = request.get_ref();

        let channel_id = ChannelId::from_str(request_ref.channel_id.as_str()).map_err(|_| {
            Status::invalid_argument(std::format!(
                "Invalid channel id: {}",
                request_ref.channel_id
            ))
        })?;

        let port_id = PortId::from_str(request_ref.port_id.as_str()).map_err(|_| {
            Status::invalid_argument(std::format!("Invalid port id: {}", request_ref.port_id))
        })?;

        let commitment_sequences = request_ref
            .packet_commitment_sequences
            .iter()
            .copied()
            .map(Sequence::from);

        let channel_end_path = ChannelEndPath::new(&port_id, &channel_id);

        let acknowledgements = self
            .ibc_context
            .packet_acknowledgements(&channel_end_path, commitment_sequences)
            .map_err(|_| {
                Status::not_found(std::format!(
                    "Packet acknowledgements not found for channel {}",
                    channel_id
                ))
            })?
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
                    .map_err(|_| {
                        Status::not_found(std::format!(
                            "Packet acknowledgement not found for channel {} and sequence {}",
                            channel_id,
                            path.sequence
                        ))
                    })
            })
            .collect::<Result<_, _>>()?;

        Ok(Response::new(QueryPacketAcknowledgementsResponse {
            acknowledgements,
            pagination: None,
            height: Some(
                self.ibc_context
                    .host_height()
                    .map_err(|_| {
                        Status::not_found(std::format!(
                            "Current height not found for channel {}",
                            channel_id
                        ))
                    })?
                    .into(),
            ),
        }))
    }

    /// UnreceivedPackets returns all the unreceived IBC packets associated with
    /// a channel and sequences.
    ///
    /// QUESTION. Currently only works for unordered channels; ordered channels
    /// don't use receipts. However, ibc-go does it this way. Investigate if
    /// this query only ever makes sense on unordered channels.
    async fn unreceived_packets(
        &self,
        request: Request<QueryUnreceivedPacketsRequest>,
    ) -> Result<Response<QueryUnreceivedPacketsResponse>, Status> {
        let request_ref = request.get_ref();

        let channel_id = ChannelId::from_str(request_ref.channel_id.as_str()).map_err(|_| {
            Status::invalid_argument(std::format!(
                "Invalid channel id: {}",
                request_ref.channel_id
            ))
        })?;

        let port_id = PortId::from_str(request_ref.port_id.as_str()).map_err(|_| {
            Status::invalid_argument(std::format!("Invalid port id: {}", request_ref.port_id))
        })?;

        let sequences = request_ref
            .packet_commitment_sequences
            .iter()
            .copied()
            .map(Sequence::from);

        let channel_end_path = ChannelEndPath::new(&port_id, &channel_id);

        let unreceived_packets = self
            .ibc_context
            .unreceived_packets(&channel_end_path, sequences)
            .map_err(|_| {
                Status::not_found(std::format!(
                    "Unreceived packets not found for channel {}",
                    channel_id
                ))
            })?;

        Ok(Response::new(QueryUnreceivedPacketsResponse {
            sequences: unreceived_packets.into_iter().map(Into::into).collect(),
            height: Some(
                self.ibc_context
                    .host_height()
                    .map_err(|_| {
                        Status::not_found(std::format!(
                            "Current height not found for channel {}",
                            channel_id
                        ))
                    })?
                    .into(),
            ),
        }))
    }

    /// UnreceivedAcks returns all the unreceived IBC acknowledgements associated
    /// with a channel and sequences.
    async fn unreceived_acks(
        &self,
        _request: Request<QueryUnreceivedAcksRequest>,
    ) -> Result<Response<QueryUnreceivedAcksResponse>, Status> {
        let request_ref = _request.get_ref();

        let channel_id = ChannelId::from_str(request_ref.channel_id.as_str()).map_err(|_| {
            Status::invalid_argument(std::format!(
                "Invalid channel id: {}",
                request_ref.channel_id
            ))
        })?;

        let port_id = PortId::from_str(request_ref.port_id.as_str()).map_err(|_| {
            Status::invalid_argument(std::format!("Invalid port id: {}", request_ref.port_id))
        })?;

        let sequences = request_ref
            .packet_ack_sequences
            .iter()
            .copied()
            .map(Sequence::from);

        let channel_end_path = ChannelEndPath::new(&port_id, &channel_id);

        let unreceived_acks = self
            .ibc_context
            .unreceived_acks(&channel_end_path, sequences)
            .map_err(|_| {
                Status::not_found(std::format!(
                    "Unreceived acks not found for channel {}",
                    channel_id
                ))
            })?;

        Ok(Response::new(QueryUnreceivedAcksResponse {
            sequences: unreceived_acks.into_iter().map(Into::into).collect(),
            height: Some(
                self.ibc_context
                    .host_height()
                    .map_err(|_| {
                        Status::not_found(std::format!(
                            "Current height not found for channel {}",
                            channel_id
                        ))
                    })?
                    .into(),
            ),
        }))
    }

    /// NextSequenceReceive returns the next receive sequence for a given channel.
    async fn next_sequence_receive(
        &self,
        request: Request<QueryNextSequenceReceiveRequest>,
    ) -> Result<Response<QueryNextSequenceReceiveResponse>, Status> {
        let request_ref = request.get_ref();

        let channel_id = ChannelId::from_str(request_ref.channel_id.as_str()).map_err(|_| {
            Status::invalid_argument(std::format!(
                "Invalid channel id: {}",
                request_ref.channel_id
            ))
        })?;

        let port_id = PortId::from_str(request_ref.port_id.as_str()).map_err(|_| {
            Status::invalid_argument(std::format!("Invalid port id: {}", request_ref.port_id))
        })?;

        let next_seq_recv_path = SeqRecvPath::new(&port_id, &channel_id);

        let next_sequence_recv = self
            .ibc_context
            .get_next_sequence_recv(&next_seq_recv_path)
            .map_err(|_| {
                Status::not_found(std::format!(
                    "Next sequence receive not found for channel {}",
                    channel_id
                ))
            })?;

        let current_height = self.ibc_context.host_height().map_err(|_| {
            Status::not_found(std::format!(
                "Current height not found for channel {}",
                channel_id
            ))
        })?;

        let proof = self
            .ibc_context
            .get_proof(current_height, &Path::SeqRecv(next_seq_recv_path))
            .ok_or_else(|| {
                Status::not_found(std::format!(
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

    // NextSequenceSend returns the next send sequence for a given channel.
    async fn next_sequence_send(
        &self,
        request: Request<QueryNextSequenceSendRequest>,
    ) -> Result<Response<QueryNextSequenceSendResponse>, Status> {
        let request_ref = request.get_ref();

        let channel_id = ChannelId::from_str(request_ref.channel_id.as_str()).map_err(|_| {
            Status::invalid_argument(std::format!(
                "Invalid channel id: {}",
                request_ref.channel_id
            ))
        })?;

        let port_id = PortId::from_str(request_ref.port_id.as_str()).map_err(|_| {
            Status::invalid_argument(std::format!("Invalid port id: {}", request_ref.port_id))
        })?;

        let next_seq_send_path = SeqSendPath::new(&port_id, &channel_id);

        let next_sequence_send = self
            .ibc_context
            .get_next_sequence_send(&next_seq_send_path)
            .map_err(|_| {
                Status::not_found(std::format!(
                    "Next sequence send not found for channel {}",
                    channel_id
                ))
            })?;

        let current_height = self.ibc_context.host_height().map_err(|_| {
            Status::not_found(std::format!(
                "Current height not found for channel {}",
                channel_id
            ))
        })?;

        let proof = self
            .ibc_context
            .get_proof(current_height, &Path::SeqSend(next_seq_send_path))
            .ok_or_else(|| {
                Status::not_found(std::format!(
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
