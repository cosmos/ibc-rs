use crate::{
    core::{
        ics04_channel::{
            channel::Order, msgs::acknowledgement::Acknowledgement, packet::Sequence,
            timeout::TimeoutHeight,
        },
        ics24_host::identifier::{ChannelId, ConnectionId, PortId},
    },
    prelude::*,
    timestamp::Timestamp,
};
use derive_more::From;
use subtle_encoding::hex;
use tendermint::abci::tag::Tag;

use crate::core::ics04_channel::error::Error;

///! This module holds all the abci event attributes for IBC events emitted
///! during packet-related datagrams.

const PKT_SEQ_ATTRIBUTE_KEY: &str = "packet_sequence";
const PKT_DATA_ATTRIBUTE_KEY: &str = "packet_data";
const PKT_DATA_HEX_ATTRIBUTE_KEY: &str = "packet_data_hex";
const PKT_SRC_PORT_ATTRIBUTE_KEY: &str = "packet_src_port";
const PKT_SRC_CHANNEL_ATTRIBUTE_KEY: &str = "packet_src_channel";
const PKT_DST_PORT_ATTRIBUTE_KEY: &str = "packet_dst_port";
const PKT_DST_CHANNEL_ATTRIBUTE_KEY: &str = "packet_dst_channel";
const PKT_CHANNEL_ORDERING_ATTRIBUTE_KEY: &str = "packet_channel_ordering";
const PKT_TIMEOUT_HEIGHT_ATTRIBUTE_KEY: &str = "packet_timeout_height";
const PKT_TIMEOUT_TIMESTAMP_ATTRIBUTE_KEY: &str = "packet_timeout_timestamp";
const PKT_ACK_ATTRIBUTE_KEY: &str = "packet_ack";
const PKT_ACK_HEX_ATTRIBUTE_KEY: &str = "packet_ack_hex";
const PKT_CONNECTION_ID_ATTRIBUTE_KEY: &str = "packet_connection";

#[derive(Debug, From)]
pub struct PacketDataAttribute {
    pub packet_data: Vec<u8>,
}

impl TryFrom<PacketDataAttribute> for Vec<Tag> {
    type Error = Error;

    fn try_from(attr: PacketDataAttribute) -> Result<Self, Self::Error> {
        let tags = vec![
            Tag {
                key: PKT_DATA_ATTRIBUTE_KEY.parse().unwrap(),
                value: String::from_utf8(attr.packet_data.clone())
                    // Note: this attribute forces us to assume that Packet data
                    // is valid UTF-8, even though the standard doesn't require
                    // it. It has been deprecated in ibc-go. It will be removed
                    // in the future.
                    .map_err(|_| Error::non_utf8_packet_data())?
                    .parse()
                    .unwrap(),
            },
            Tag {
                key: PKT_DATA_HEX_ATTRIBUTE_KEY.parse().unwrap(),
                value: String::from_utf8(hex::encode(attr.packet_data))
                    .unwrap()
                    .parse()
                    .unwrap(),
            },
        ];

        Ok(tags)
    }
}

#[derive(Debug, From)]
pub struct TimeoutHeightAttribute {
    pub timeout_height: TimeoutHeight,
}

impl From<TimeoutHeightAttribute> for Tag {
    fn from(attr: TimeoutHeightAttribute) -> Self {
        Tag {
            key: PKT_TIMEOUT_HEIGHT_ATTRIBUTE_KEY.parse().unwrap(),
            value: match attr.timeout_height {
                TimeoutHeight::Never => "0-0".to_string(),
                TimeoutHeight::At(height) => height.to_string(),
            }
            .parse()
            .unwrap(),
        }
    }
}

#[derive(Debug, From)]
pub struct TimeoutTimestampAttribute {
    pub timeout_timestamp: Timestamp,
}

impl From<TimeoutTimestampAttribute> for Tag {
    fn from(attr: TimeoutTimestampAttribute) -> Self {
        Tag {
            key: PKT_TIMEOUT_TIMESTAMP_ATTRIBUTE_KEY.parse().unwrap(),
            value: attr
                .timeout_timestamp
                .nanoseconds()
                .to_string()
                .parse()
                .unwrap(),
        }
    }
}

#[derive(Debug, From)]
pub struct SequenceAttribute {
    pub sequence: Sequence,
}

impl From<SequenceAttribute> for Tag {
    fn from(attr: SequenceAttribute) -> Self {
        Tag {
            key: PKT_SEQ_ATTRIBUTE_KEY.parse().unwrap(),
            value: u64::from(attr.sequence).to_string().parse().unwrap(),
        }
    }
}

#[derive(Debug, From)]
pub struct SrcPortIdAttribute {
    pub src_port_id: PortId,
}

impl From<SrcPortIdAttribute> for Tag {
    fn from(attr: SrcPortIdAttribute) -> Self {
        Tag {
            key: PKT_SRC_PORT_ATTRIBUTE_KEY.parse().unwrap(),
            value: attr.src_port_id.as_str().parse().unwrap(),
        }
    }
}

#[derive(Debug, From)]
pub struct SrcChannelIdAttribute {
    pub src_channel_id: ChannelId,
}

impl From<SrcChannelIdAttribute> for Tag {
    fn from(attr: SrcChannelIdAttribute) -> Self {
        Tag {
            key: PKT_SRC_CHANNEL_ATTRIBUTE_KEY.parse().unwrap(),
            value: attr.src_channel_id.as_str().parse().unwrap(),
        }
    }
}

#[derive(Debug, From)]
pub struct DstPortIdAttribute {
    pub dst_port_id: PortId,
}

impl From<DstPortIdAttribute> for Tag {
    fn from(attr: DstPortIdAttribute) -> Self {
        Tag {
            key: PKT_DST_PORT_ATTRIBUTE_KEY.parse().unwrap(),
            value: attr.dst_port_id.as_str().parse().unwrap(),
        }
    }
}

#[derive(Debug, From)]
pub struct DstChannelIdAttribute {
    pub dst_channel_id: ChannelId,
}

impl From<DstChannelIdAttribute> for Tag {
    fn from(attr: DstChannelIdAttribute) -> Self {
        Tag {
            key: PKT_DST_CHANNEL_ATTRIBUTE_KEY.parse().unwrap(),
            value: attr.dst_channel_id.as_str().parse().unwrap(),
        }
    }
}

#[derive(Debug, From)]
pub struct ChannelOrderingAttribute {
    pub order: Order,
}

impl From<ChannelOrderingAttribute> for Tag {
    fn from(attr: ChannelOrderingAttribute) -> Self {
        Tag {
            key: PKT_CHANNEL_ORDERING_ATTRIBUTE_KEY.parse().unwrap(),
            value: attr.order.as_str().parse().unwrap(),
        }
    }
}

#[derive(Debug, From)]
pub struct PacketConnectionIdAttribute {
    pub connection_id: ConnectionId,
}

impl From<PacketConnectionIdAttribute> for Tag {
    fn from(attr: PacketConnectionIdAttribute) -> Self {
        Tag {
            key: PKT_CONNECTION_ID_ATTRIBUTE_KEY.parse().unwrap(),
            value: attr.connection_id.as_str().parse().unwrap(),
        }
    }
}

#[derive(Debug, From)]
pub struct AcknowledgementAttribute {
    pub acknowledgement: Acknowledgement,
}

impl TryFrom<AcknowledgementAttribute> for Vec<Tag> {
    type Error = Error;

    fn try_from(attr: AcknowledgementAttribute) -> Result<Self, Self::Error> {
        let tags = vec![
            Tag {
                key: PKT_ACK_ATTRIBUTE_KEY.parse().unwrap(),
                value: String::from_utf8(attr.acknowledgement.as_ref().into())
                    // Note: this attribute forces us to assume that Packet data
                    // is valid UTF-8, even though the standard doesn't require
                    // it. It has been deprecated in ibc-go. It will be removed
                    // in the future.
                    .map_err(|_| Error::non_utf8_packet_data())?
                    .parse()
                    .unwrap(),
            },
            Tag {
                key: PKT_ACK_HEX_ATTRIBUTE_KEY.parse().unwrap(),
                value: String::from_utf8(hex::encode(attr.acknowledgement))
                    .unwrap()
                    .parse()
                    .unwrap(),
            },
        ];

        Ok(tags)
    }
}
