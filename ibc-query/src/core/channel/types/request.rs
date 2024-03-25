//! Contains all the RPC method request domain types and their conversions to
//! and from the corresponding gRPC proto types for the channel module.

use ibc::core::client::types::Height;
use ibc::core::host::types::identifiers::{ChannelId, ConnectionId, PortId, Sequence};
use ibc::primitives::prelude::*;
use ibc_proto::ibc::core::channel::v1::{
    QueryChannelClientStateRequest as RawQueryChannelClientStateRequest,
    QueryChannelConsensusStateRequest as RawQueryChannelConsensusStateRequest,
    QueryChannelRequest as RawQueryChannelRequest, QueryChannelsRequest as RawQueryChannelsRequest,
    QueryConnectionChannelsRequest as RawQueryConnectionChannelsRequest,
    QueryNextSequenceReceiveRequest as RawQueryNextSequenceReceiveRequest,
    QueryNextSequenceSendRequest as RawQueryNextSequenceSendRequest,
    QueryPacketAcknowledgementRequest as RawQueryPacketAcknowledgementRequest,
    QueryPacketAcknowledgementsRequest as RawQueryPacketAcknowledgementsRequest,
    QueryPacketCommitmentRequest as RawQueryPacketCommitmentRequest,
    QueryPacketCommitmentsRequest as RawQueryPacketCommitmentsRequest,
    QueryPacketReceiptRequest as RawQueryPacketReceiptRequest,
    QueryUnreceivedAcksRequest as RawQueryUnreceivedAcksRequest,
    QueryUnreceivedPacketsRequest as RawQueryUnreceivedPacketsRequest,
};

use crate::error::QueryError;
use crate::types::PageRequest;

/// Defines the RPC method request type for querying a channel
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct QueryChannelRequest {
    pub port_id: PortId,
    pub channel_id: ChannelId,
    pub query_height: Option<Height>,
}

impl TryFrom<RawQueryChannelRequest> for QueryChannelRequest {
    type Error = QueryError;

    fn try_from(request: RawQueryChannelRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            port_id: request.port_id.parse()?,
            channel_id: request.channel_id.parse()?,
            query_height: None,
        })
    }
}

/// Defines the RPC method request type for querying all channels
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct QueryChannelsRequest {
    pub pagination: Option<PageRequest>,
}

impl From<RawQueryChannelsRequest> for QueryChannelsRequest {
    fn from(request: RawQueryChannelsRequest) -> Self {
        Self {
            pagination: request.pagination.map(Into::into),
        }
    }
}

/// Defines the RPC method request type for querying all channels associated
/// with a connection identifier
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct QueryConnectionChannelsRequest {
    pub connection_id: ConnectionId,
    pub pagination: Option<PageRequest>,
}

impl TryFrom<RawQueryConnectionChannelsRequest> for QueryConnectionChannelsRequest {
    type Error = QueryError;

    fn try_from(request: RawQueryConnectionChannelsRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            connection_id: request.connection.parse()?,
            pagination: request.pagination.map(Into::into),
        })
    }
}

/// Defines the RPC method request type for querying the client state associated
/// with a channel
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct QueryChannelClientStateRequest {
    pub port_id: PortId,
    pub channel_id: ChannelId,
    pub query_height: Option<Height>,
}

impl TryFrom<RawQueryChannelClientStateRequest> for QueryChannelClientStateRequest {
    type Error = QueryError;

    fn try_from(request: RawQueryChannelClientStateRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            port_id: request.port_id.parse()?,
            channel_id: request.channel_id.parse()?,
            query_height: None,
        })
    }
}

/// Defines the RPC method request type for querying the consensus state
/// associated with a channel
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct QueryChannelConsensusStateRequest {
    pub port_id: PortId,
    pub channel_id: ChannelId,
    pub consensus_height: Height,
    pub query_height: Option<Height>,
}

impl TryFrom<RawQueryChannelConsensusStateRequest> for QueryChannelConsensusStateRequest {
    type Error = QueryError;

    fn try_from(request: RawQueryChannelConsensusStateRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            port_id: request.port_id.parse()?,
            channel_id: request.channel_id.parse()?,
            consensus_height: Height::new(request.revision_number, request.revision_height)?,
            query_height: None,
        })
    }
}

/// Defines the RPC method request type for querying the packet commitment
/// associated with the specified channel
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct QueryPacketCommitmentRequest {
    pub port_id: PortId,
    pub channel_id: ChannelId,
    pub sequence: Sequence,
    pub query_height: Option<Height>,
}

impl TryFrom<RawQueryPacketCommitmentRequest> for QueryPacketCommitmentRequest {
    type Error = QueryError;

    fn try_from(request: RawQueryPacketCommitmentRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            port_id: request.port_id.parse()?,
            channel_id: request.channel_id.parse()?,
            sequence: request.sequence.into(),
            query_height: None,
        })
    }
}

/// Defines the RPC method request type for querying all packet commitments
/// associated with the specified channel
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct QueryPacketCommitmentsRequest {
    pub port_id: PortId,
    pub channel_id: ChannelId,
    pub pagination: Option<PageRequest>,
}

impl TryFrom<RawQueryPacketCommitmentsRequest> for QueryPacketCommitmentsRequest {
    type Error = QueryError;

    fn try_from(request: RawQueryPacketCommitmentsRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            port_id: request.port_id.parse()?,
            channel_id: request.channel_id.parse()?,
            pagination: request.pagination.map(Into::into),
        })
    }
}

impl From<QueryPacketCommitmentsRequest> for RawQueryPacketCommitmentsRequest {
    fn from(request: QueryPacketCommitmentsRequest) -> Self {
        Self {
            port_id: request.port_id.to_string(),
            channel_id: request.channel_id.to_string(),
            pagination: request.pagination.map(Into::into),
        }
    }
}

/// Defines the RPC method request type for querying the packet receipt
/// associated with the specified channel and sequence number
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct QueryPacketReceiptRequest {
    pub port_id: PortId,
    pub channel_id: ChannelId,
    pub sequence: Sequence,
    pub query_height: Option<Height>,
}

impl TryFrom<RawQueryPacketReceiptRequest> for QueryPacketReceiptRequest {
    type Error = QueryError;

    fn try_from(request: RawQueryPacketReceiptRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            port_id: request.port_id.parse()?,
            channel_id: request.channel_id.parse()?,
            sequence: request.sequence.into(),
            query_height: None,
        })
    }
}

/// Defines the RPC method request type for querying the packet acknowledgement
/// associated with the specified channel and sequence number
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct QueryPacketAcknowledgementRequest {
    pub port_id: PortId,
    pub channel_id: ChannelId,
    pub sequence: Sequence,
    pub query_height: Option<Height>,
}

impl TryFrom<RawQueryPacketAcknowledgementRequest> for QueryPacketAcknowledgementRequest {
    type Error = QueryError;

    fn try_from(request: RawQueryPacketAcknowledgementRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            port_id: request.port_id.parse()?,
            channel_id: request.channel_id.parse()?,
            sequence: request.sequence.into(),
            query_height: None,
        })
    }
}

/// Defines the RPC method request type for querying the packet acknowledgements
/// associated with the specified channel
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct QueryPacketAcknowledgementsRequest {
    pub port_id: PortId,
    pub channel_id: ChannelId,
    pub packet_commitment_sequences: Vec<Sequence>,
    pub pagination: Option<PageRequest>,
}

impl TryFrom<RawQueryPacketAcknowledgementsRequest> for QueryPacketAcknowledgementsRequest {
    type Error = QueryError;

    fn try_from(request: RawQueryPacketAcknowledgementsRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            port_id: request.port_id.parse()?,
            channel_id: request.channel_id.parse()?,
            packet_commitment_sequences: request
                .packet_commitment_sequences
                .into_iter()
                .map(Into::into)
                .collect(),
            pagination: request.pagination.map(Into::into),
        })
    }
}

impl From<QueryPacketAcknowledgementsRequest> for RawQueryPacketAcknowledgementsRequest {
    fn from(request: QueryPacketAcknowledgementsRequest) -> Self {
        Self {
            port_id: request.port_id.to_string(),
            channel_id: request.channel_id.to_string(),
            packet_commitment_sequences: request
                .packet_commitment_sequences
                .into_iter()
                .map(Into::into)
                .collect(),
            pagination: request.pagination.map(Into::into),
        }
    }
}

/// Defines the RPC method request type for querying the unreceived packets
/// associated with the specified channel
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct QueryUnreceivedPacketsRequest {
    pub port_id: PortId,
    pub channel_id: ChannelId,
    pub packet_commitment_sequences: Vec<Sequence>,
}

impl TryFrom<RawQueryUnreceivedPacketsRequest> for QueryUnreceivedPacketsRequest {
    type Error = QueryError;

    fn try_from(request: RawQueryUnreceivedPacketsRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            port_id: request.port_id.parse()?,
            channel_id: request.channel_id.parse()?,
            packet_commitment_sequences: request
                .packet_commitment_sequences
                .into_iter()
                .map(Into::into)
                .collect(),
        })
    }
}

impl From<QueryUnreceivedPacketsRequest> for RawQueryUnreceivedPacketsRequest {
    fn from(request: QueryUnreceivedPacketsRequest) -> Self {
        Self {
            port_id: request.port_id.to_string(),
            channel_id: request.channel_id.to_string(),
            packet_commitment_sequences: request
                .packet_commitment_sequences
                .into_iter()
                .map(Into::into)
                .collect(),
        }
    }
}

/// gRPC query to fetch the unreceived acknowledgements sequences associated with
/// the specified channel.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct QueryUnreceivedAcksRequest {
    pub port_id: PortId,
    pub channel_id: ChannelId,
    pub packet_ack_sequences: Vec<Sequence>,
}

impl TryFrom<RawQueryUnreceivedAcksRequest> for QueryUnreceivedAcksRequest {
    type Error = QueryError;

    fn try_from(request: RawQueryUnreceivedAcksRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            port_id: request.port_id.parse()?,
            channel_id: request.channel_id.parse()?,
            packet_ack_sequences: request
                .packet_ack_sequences
                .into_iter()
                .map(Into::into)
                .collect(),
        })
    }
}

impl From<QueryUnreceivedAcksRequest> for RawQueryUnreceivedAcksRequest {
    fn from(request: QueryUnreceivedAcksRequest) -> Self {
        Self {
            port_id: request.port_id.to_string(),
            channel_id: request.channel_id.to_string(),
            packet_ack_sequences: request
                .packet_ack_sequences
                .into_iter()
                .map(Into::into)
                .collect(),
        }
    }
}

/// Defines the RPC method request type for querying the next sequence receive
/// number for the specified channel
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct QueryNextSequenceReceiveRequest {
    pub port_id: PortId,
    pub channel_id: ChannelId,
    pub query_height: Option<Height>,
}

impl TryFrom<RawQueryNextSequenceReceiveRequest> for QueryNextSequenceReceiveRequest {
    type Error = QueryError;

    fn try_from(request: RawQueryNextSequenceReceiveRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            port_id: request.port_id.parse()?,
            channel_id: request.channel_id.parse()?,
            query_height: None,
        })
    }
}
/// Defines the RPC method request type for querying the next sequence send
/// number for the specified channel
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct QueryNextSequenceSendRequest {
    pub port_id: PortId,
    pub channel_id: ChannelId,
    pub query_height: Option<Height>,
}

impl TryFrom<RawQueryNextSequenceSendRequest> for QueryNextSequenceSendRequest {
    type Error = QueryError;

    fn try_from(request: RawQueryNextSequenceSendRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            port_id: request.port_id.parse()?,
            channel_id: request.channel_id.parse()?,
            query_height: None,
        })
    }
}
