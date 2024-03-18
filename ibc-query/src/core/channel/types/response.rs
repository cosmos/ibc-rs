//! Contains all the RPC method response domain types and their conversions to
//! and from the corresponding gRPC proto types for the channel module.

use ibc::core::channel::types::channel::{ChannelEnd, IdentifiedChannelEnd};
use ibc::core::channel::types::commitment::{AcknowledgementCommitment, PacketCommitment};
use ibc::core::channel::types::packet::PacketState;
use ibc::core::client::types::Height;
use ibc::core::host::types::identifiers::{ClientId, Sequence};
use ibc::core::primitives::proto::Any;
use ibc::primitives::prelude::*;
use ibc_proto::ibc::core::channel::v1::{
    QueryChannelClientStateResponse as RawQueryChannelClientStateResponse,
    QueryChannelConsensusStateResponse as RawQueryChannelConsensusStateResponse,
    QueryChannelResponse as RawQueryChannelResponse,
    QueryChannelsResponse as RawQueryChannelsResponse,
    QueryConnectionChannelsResponse as RawQueryConnectionChannelsResponse,
    QueryNextSequenceReceiveResponse as RawQueryNextSequenceReceiveResponse,
    QueryNextSequenceSendResponse as RawQueryNextSequenceSendResponse,
    QueryPacketAcknowledgementResponse as RawQueryPacketAcknowledgementResponse,
    QueryPacketAcknowledgementsResponse as RawQueryPacketAcknowledgementsResponse,
    QueryPacketCommitmentResponse as RawQueryPacketCommitmentResponse,
    QueryPacketCommitmentsResponse as RawQueryPacketCommitmentsResponse,
    QueryPacketReceiptResponse as RawQueryPacketReceiptResponse,
    QueryUnreceivedAcksResponse as RawQueryUnreceivedAcksResponse,
    QueryUnreceivedPacketsResponse as RawQueryUnreceivedPacketsResponse,
};

use crate::core::client::IdentifiedClientState;
use crate::types::{PageResponse, Proof};

/// Defines the RPC method response type when querying a channel.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct QueryChannelResponse {
    pub channel: ChannelEnd,
    pub proof: Proof,
    pub proof_height: Height,
}

impl QueryChannelResponse {
    pub fn new(channel: ChannelEnd, proof: Proof, proof_height: Height) -> Self {
        Self {
            channel,
            proof,
            proof_height,
        }
    }
}

impl From<QueryChannelResponse> for RawQueryChannelResponse {
    fn from(response: QueryChannelResponse) -> Self {
        Self {
            channel: Some(response.channel.into()),
            proof: response.proof,
            proof_height: Some(response.proof_height.into()),
        }
    }
}

/// Defines the RPC method response type when querying a list of channels.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct QueryChannelsResponse {
    pub channels: Vec<IdentifiedChannelEnd>,
    pub query_height: Height,
    pub pagination: Option<PageResponse>,
}

impl QueryChannelsResponse {
    pub fn new(
        channels: Vec<IdentifiedChannelEnd>,
        query_height: Height,
        pagination: Option<PageResponse>,
    ) -> Self {
        Self {
            channels,
            query_height,
            pagination,
        }
    }
}

impl From<QueryChannelsResponse> for RawQueryChannelsResponse {
    fn from(response: QueryChannelsResponse) -> Self {
        Self {
            channels: response.channels.into_iter().map(Into::into).collect(),
            height: Some(response.query_height.into()),
            pagination: response.pagination.map(|pagination| pagination.into()),
        }
    }
}

/// Defines the RPC method response type when querying a list of channels associated with a connection.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct QueryConnectionChannelsResponse {
    pub channels: Vec<IdentifiedChannelEnd>,
    pub query_height: Height,
    pub pagination: Option<PageResponse>,
}

impl QueryConnectionChannelsResponse {
    pub fn new(
        channels: Vec<IdentifiedChannelEnd>,
        query_height: Height,
        pagination: Option<PageResponse>,
    ) -> Self {
        Self {
            channels,
            query_height,
            pagination,
        }
    }
}

impl From<QueryConnectionChannelsResponse> for RawQueryConnectionChannelsResponse {
    fn from(response: QueryConnectionChannelsResponse) -> Self {
        Self {
            channels: response.channels.into_iter().map(Into::into).collect(),
            height: Some(response.query_height.into()),
            pagination: response.pagination.map(|pagination| pagination.into()),
        }
    }
}

/// Defines the RPC method response type when querying a channel client state.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct QueryChannelClientStateResponse {
    pub identified_client_state: IdentifiedClientState,
    pub proof: Proof,
    pub proof_height: Height,
}

impl QueryChannelClientStateResponse {
    pub fn new(
        identified_client_state: IdentifiedClientState,
        proof: Proof,
        proof_height: Height,
    ) -> Self {
        Self {
            identified_client_state,
            proof,
            proof_height,
        }
    }
}

impl From<QueryChannelClientStateResponse> for RawQueryChannelClientStateResponse {
    fn from(response: QueryChannelClientStateResponse) -> Self {
        Self {
            identified_client_state: Some(response.identified_client_state.into()),
            proof: response.proof,
            proof_height: Some(response.proof_height.into()),
        }
    }
}

/// Defines the RPC method response when for querying a channel consensus state.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct QueryChannelConsensusStateResponse {
    pub consensus_state: Any,
    pub client_id: ClientId,
    pub proof: Proof,
    pub proof_height: Height,
}

impl QueryChannelConsensusStateResponse {
    pub fn new(
        consensus_state: Any,
        client_id: ClientId,
        proof: Proof,
        proof_height: Height,
    ) -> Self {
        Self {
            consensus_state,
            client_id,
            proof,
            proof_height,
        }
    }
}

impl From<QueryChannelConsensusStateResponse> for RawQueryChannelConsensusStateResponse {
    fn from(response: QueryChannelConsensusStateResponse) -> Self {
        Self {
            consensus_state: Some(response.consensus_state),
            client_id: response.client_id.to_string(),
            proof: response.proof,
            proof_height: Some(response.proof_height.into()),
        }
    }
}

/// Defines the RPC method response type when querying a packet commitment.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct QueryPacketCommitmentResponse {
    pub packet_commitment: PacketCommitment,
    pub proof: Proof,
    pub proof_height: Height,
}

impl QueryPacketCommitmentResponse {
    pub fn new(packet_commitment: PacketCommitment, proof: Proof, proof_height: Height) -> Self {
        Self {
            packet_commitment,
            proof,
            proof_height,
        }
    }
}

impl From<QueryPacketCommitmentResponse> for RawQueryPacketCommitmentResponse {
    fn from(response: QueryPacketCommitmentResponse) -> Self {
        Self {
            commitment: response.packet_commitment.into_vec(),
            proof: response.proof,
            proof_height: Some(response.proof_height.into()),
        }
    }
}

/// Defines the RPC method response type when querying a list of packet commitments.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct QueryPacketCommitmentsResponse {
    pub commitments: Vec<PacketState>,
    pub height: Height,
    pub pagination: Option<PageResponse>,
}

impl QueryPacketCommitmentsResponse {
    pub fn new(
        commitments: Vec<PacketState>,
        height: Height,
        pagination: Option<PageResponse>,
    ) -> Self {
        Self {
            commitments,
            height,
            pagination,
        }
    }
}

impl From<QueryPacketCommitmentsResponse> for RawQueryPacketCommitmentsResponse {
    fn from(response: QueryPacketCommitmentsResponse) -> Self {
        Self {
            commitments: response.commitments.into_iter().map(Into::into).collect(),
            height: Some(response.height.into()),
            pagination: response.pagination.map(|pagination| pagination.into()),
        }
    }
}

/// Defines the RPC method response type when querying a packet receipt.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct QueryPacketReceiptResponse {
    pub received: bool,
    pub proof: Proof,
    pub proof_height: Height,
}

impl QueryPacketReceiptResponse {
    pub fn new(received: bool, proof: Proof, proof_height: Height) -> Self {
        Self {
            received,
            proof,
            proof_height,
        }
    }
}

impl From<QueryPacketReceiptResponse> for RawQueryPacketReceiptResponse {
    fn from(response: QueryPacketReceiptResponse) -> Self {
        Self {
            received: response.received,
            proof: response.proof,
            proof_height: Some(response.proof_height.into()),
        }
    }
}

/// Defines the RPC method response type when querying a packet acknowledgement.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct QueryPacketAcknowledgementResponse {
    pub acknowledgement: AcknowledgementCommitment,
    pub proof: Proof,
    pub proof_height: Height,
}

impl QueryPacketAcknowledgementResponse {
    pub fn new(
        acknowledgement: AcknowledgementCommitment,
        proof: Proof,
        proof_height: Height,
    ) -> Self {
        Self {
            acknowledgement,
            proof,
            proof_height,
        }
    }
}

impl From<QueryPacketAcknowledgementResponse> for RawQueryPacketAcknowledgementResponse {
    fn from(response: QueryPacketAcknowledgementResponse) -> Self {
        Self {
            acknowledgement: response.acknowledgement.into_vec(),
            proof: response.proof,
            proof_height: Some(response.proof_height.into()),
        }
    }
}

/// Defines the RPC method response type when querying a list of packet
/// acknowledgements.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct QueryPacketAcknowledgementsResponse {
    pub acknowledgements: Vec<PacketState>,
    pub height: Height,
    pub pagination: Option<PageResponse>,
}

impl QueryPacketAcknowledgementsResponse {
    pub fn new(
        acknowledgements: Vec<PacketState>,
        height: Height,
        pagination: Option<PageResponse>,
    ) -> Self {
        Self {
            acknowledgements,
            height,
            pagination,
        }
    }
}

impl From<QueryPacketAcknowledgementsResponse> for RawQueryPacketAcknowledgementsResponse {
    fn from(response: QueryPacketAcknowledgementsResponse) -> Self {
        Self {
            acknowledgements: response
                .acknowledgements
                .into_iter()
                .map(Into::into)
                .collect(),
            height: Some(response.height.into()),
            pagination: response.pagination.map(|pagination| pagination.into()),
        }
    }
}

/// Defines the RPC method response type when querying a list of unreceived acks.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct QueryUnreceivedAcksResponse {
    pub sequences: Vec<Sequence>,
    pub height: Height,
}

impl QueryUnreceivedAcksResponse {
    pub fn new(sequences: Vec<Sequence>, height: Height) -> Self {
        Self { sequences, height }
    }
}

impl From<QueryUnreceivedAcksResponse> for RawQueryUnreceivedAcksResponse {
    fn from(response: QueryUnreceivedAcksResponse) -> Self {
        Self {
            sequences: response.sequences.into_iter().map(Into::into).collect(),
            height: Some(response.height.into()),
        }
    }
}

/// Defines the RPC method response type when querying a list of unreceived packets.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct QueryUnreceivedPacketsResponse {
    pub sequences: Vec<Sequence>,
    pub height: Height,
}

impl QueryUnreceivedPacketsResponse {
    pub fn new(sequences: Vec<Sequence>, height: Height) -> Self {
        Self { sequences, height }
    }
}

impl From<QueryUnreceivedPacketsResponse> for RawQueryUnreceivedPacketsResponse {
    fn from(response: QueryUnreceivedPacketsResponse) -> Self {
        Self {
            sequences: response.sequences.into_iter().map(Into::into).collect(),
            height: Some(response.height.into()),
        }
    }
}

/// Defines the RPC method response type when querying the next sequence to be received on a channel.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct QueryNextSequenceReceiveResponse {
    pub next_sequence_receive: Sequence,
    pub proof: Proof,
    pub proof_height: Height,
}

impl QueryNextSequenceReceiveResponse {
    pub fn new(next_sequence_receive: Sequence, proof: Proof, proof_height: Height) -> Self {
        Self {
            next_sequence_receive,
            proof,
            proof_height,
        }
    }
}

impl From<QueryNextSequenceReceiveResponse> for RawQueryNextSequenceReceiveResponse {
    fn from(response: QueryNextSequenceReceiveResponse) -> Self {
        Self {
            next_sequence_receive: response.next_sequence_receive.into(),
            proof: response.proof,
            proof_height: Some(response.proof_height.into()),
        }
    }
}

/// Defines the RPC method response type when querying the next sequence to be
/// sent on a channel.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct QueryNextSequenceSendResponse {
    pub next_sequence_send: Sequence,
    pub proof: Proof,
    pub proof_height: Height,
}

impl QueryNextSequenceSendResponse {
    pub fn new(next_sequence_send: Sequence, proof: Proof, proof_height: Height) -> Self {
        Self {
            next_sequence_send,
            proof,
            proof_height,
        }
    }
}

impl From<QueryNextSequenceSendResponse> for RawQueryNextSequenceSendResponse {
    fn from(response: QueryNextSequenceSendResponse) -> Self {
        Self {
            next_sequence_send: response.next_sequence_send.into(),
            proof: response.proof,
            proof_height: Some(response.proof_height.into()),
        }
    }
}
