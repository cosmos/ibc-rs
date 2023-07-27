use ibc_proto::{
    google::protobuf::Any,
    ibc::core::{
        channel::v1::{
            query_server::Query as ChannelQuery, QueryChannelClientStateRequest,
            QueryChannelClientStateResponse, QueryChannelConsensusStateRequest,
            QueryChannelConsensusStateResponse, QueryChannelRequest, QueryChannelResponse,
            QueryChannelsRequest, QueryChannelsResponse, QueryConnectionChannelsRequest,
            QueryConnectionChannelsResponse, QueryNextSequenceReceiveRequest,
            QueryNextSequenceReceiveResponse, QueryPacketAcknowledgementRequest,
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
            identifier::{ChannelId, PortId},
            path::{
                AckPath, ChannelEndPath, ClientConsensusStatePath, CommitmentPath, ReceiptPath,
                SeqRecvPath,
            },
        },
        ValidationContext,
    },
    Height,
};

use core::str::FromStr;
use std::boxed::Box;
use tonic::{Request, Response, Status};
use tracing::trace;

pub struct ChannelQueryServer<T> {
    context: T,
}

impl<T> ChannelQueryServer<T> {
    pub fn new(context: T) -> Self {
        Self { context }
    }
}

#[tonic::async_trait]
impl<T> ChannelQuery for ChannelQueryServer<T>
where
    T: ValidationContext + Send + Sync + 'static,
    <T as ValidationContext>::AnyClientState: Into<Any>,
    <T as ValidationContext>::AnyConsensusState: Into<Any>,
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

        let channel_end = self.context.channel_end(&channel_end_path).map_err(|_| {
            Status::not_found(std::format!(
                "Channel end not found for channel {}",
                channel_id
            ))
        })?;

        Ok(Response::new(QueryChannelResponse {
            channel: Some(channel_end.into()),
            proof: Default::default(),
            proof_height: None,
        }))
    }
    /// Channels queries all the IBC channels of a chain.
    async fn channels(
        &self,
        _request: Request<QueryChannelsRequest>,
    ) -> Result<Response<QueryChannelsResponse>, Status> {
        todo!()
    }
    /// ConnectionChannels queries all the channels associated with a connection
    /// end.
    async fn connection_channels(
        &self,
        _request: Request<QueryConnectionChannelsRequest>,
    ) -> Result<Response<QueryConnectionChannelsResponse>, Status> {
        todo!()
    }
    /// ChannelClientState queries for the client state for the channel associated
    /// with the provided channel identifiers.
    async fn channel_client_state(
        &self,
        request: Request<QueryChannelClientStateRequest>,
    ) -> Result<Response<QueryChannelClientStateResponse>, Status> {
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

        let channel_end = self.context.channel_end(&channel_end_path).map_err(|_| {
            Status::not_found(std::format!(
                "Channel end not found for channel {}",
                channel_id
            ))
        })?;

        let connection_end = channel_end
            .connection_hops()
            .first()
            .map(|connection_id| {
                self.context.connection_end(connection_id).map_err(|_| {
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
            .context
            .client_state(connection_end.client_id())
            .map_err(|_| {
                Status::not_found(std::format!(
                    "Client state not found for client {}",
                    connection_end.client_id()
                ))
            })?;

        Ok(Response::new(QueryChannelClientStateResponse {
            identified_client_state: Some(IdentifiedClientState {
                client_id: connection_end.client_id().as_str().into(),
                client_state: Some(client_state.into()),
            }),
            proof: Default::default(),
            proof_height: None,
        }))
    }
    /// ChannelConsensusState queries for the consensus state for the channel
    /// associated with the provided channel identifiers.
    async fn channel_consensus_state(
        &self,
        request: Request<QueryChannelConsensusStateRequest>,
    ) -> Result<Response<QueryChannelConsensusStateResponse>, Status> {
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

        let channel_end = self.context.channel_end(&channel_end_path).map_err(|_| {
            Status::not_found(std::format!(
                "Channel end not found for channel {}",
                channel_id
            ))
        })?;

        let connection_end = channel_end
            .connection_hops()
            .first()
            .map(|connection_id| {
                self.context.connection_end(connection_id).map_err(|_| {
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

        let consensus_state = self.context.consensus_state(&consensus_path).map_err(|_| {
            Status::not_found(std::format!(
                "Consensus state not found for client {} and revision {}",
                connection_end.client_id(),
                request_ref.revision_number
            ))
        })?;

        Ok(Response::new(QueryChannelConsensusStateResponse {
            client_id: connection_end.client_id().as_str().into(),
            consensus_state: Some(consensus_state.into()),
            proof: Default::default(),
            proof_height: None,
        }))
    }
    /// PacketCommitment queries a stored packet commitment hash.
    async fn packet_commitment(
        &self,
        request: Request<QueryPacketCommitmentRequest>,
    ) -> Result<Response<QueryPacketCommitmentResponse>, Status> {
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
            .context
            .get_packet_commitment(&commitment_path)
            .map_err(|_| {
                Status::not_found(std::format!(
                    "Packet commitment not found for channel {} and sequence {}",
                    channel_id,
                    sequence
                ))
            })?;

        Ok(Response::new(QueryPacketCommitmentResponse {
            commitment: packet_commitment_data.into_vec(),
            proof: Default::default(),
            proof_height: None,
        }))
    }
    /// PacketCommitments returns all the packet commitments hashes associated
    /// with a channel.
    async fn packet_commitments(
        &self,
        _request: Request<QueryPacketCommitmentsRequest>,
    ) -> Result<Response<QueryPacketCommitmentsResponse>, Status> {
        todo!()
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
        let packet_receipt_data = self.context.get_packet_receipt(&receipt_path);

        Ok(Response::new(QueryPacketReceiptResponse {
            received: packet_receipt_data.is_ok(),
            proof: Default::default(),
            proof_height: None,
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
            .context
            .get_packet_acknowledgement(&acknowledgement_path)
            .map_err(|_| {
                Status::not_found(std::format!(
                    "Packet acknowledgement not found for channel {} and sequence {}",
                    channel_id,
                    sequence
                ))
            })?;

        Ok(Response::new(QueryPacketAcknowledgementResponse {
            acknowledgement: packet_acknowledgement_data.into_vec(),
            proof: Default::default(),
            proof_height: None,
        }))
    }

    /// PacketAcknowledgements returns all the packet acknowledgements associated
    /// with a channel.
    async fn packet_acknowledgements(
        &self,
        _request: Request<QueryPacketAcknowledgementsRequest>,
    ) -> Result<Response<QueryPacketAcknowledgementsResponse>, Status> {
        todo!()
    }

    /// UnreceivedPackets returns all the unreceived IBC packets associated with
    /// a channel and sequences.
    ///
    /// QUESTION. Currently only works for unordered channels; ordered channels
    /// don't use receipts. However, ibc-go does it this way. Investigate if
    /// this query only ever makes sense on unordered channels.
    async fn unreceived_packets(
        &self,
        _request: Request<QueryUnreceivedPacketsRequest>,
    ) -> Result<Response<QueryUnreceivedPacketsResponse>, Status> {
        todo!()
    }

    /// UnreceivedAcks returns all the unreceived IBC acknowledgements associated
    /// with a channel and sequences.
    async fn unreceived_acks(
        &self,
        _request: Request<QueryUnreceivedAcksRequest>,
    ) -> Result<Response<QueryUnreceivedAcksResponse>, Status> {
        todo!()
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
            .context
            .get_next_sequence_recv(&next_seq_recv_path)
            .map_err(|_| {
                Status::not_found(std::format!(
                    "Next sequence receive not found for channel {}",
                    channel_id
                ))
            })?;

        Ok(Response::new(QueryNextSequenceReceiveResponse {
            next_sequence_receive: next_sequence_recv.into(),
            proof: Default::default(),
            proof_height: None,
        }))
    }
}
