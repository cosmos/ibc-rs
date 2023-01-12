use crate::{
    core::{
        ics04_channel::{
            channel::Order, error::ChannelError, msgs::acknowledgement::Acknowledgement,
            packet::Sequence, timeout::TimeoutHeight,
        },
        ics24_host::identifier::{ChannelId, ConnectionId, PortId},
    },
    prelude::*,
    timestamp::Timestamp,
};
use derive_more::From;
use subtle_encoding::hex;
use tendermint::abci;

use core::str;

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

#[cfg_attr(
    feature = "parity-scale-codec",
    derive(
        parity_scale_codec::Encode,
        parity_scale_codec::Decode,
        scale_info::TypeInfo
    )
)]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, From, PartialEq, Eq)]
pub struct PacketDataAttribute {
    pub packet_data: Vec<u8>,
}

impl TryFrom<PacketDataAttribute> for Vec<abci::EventAttribute> {
    type Error = ChannelError;

    fn try_from(attr: PacketDataAttribute) -> Result<Self, Self::Error> {
        let tags = vec![
            (
                PKT_DATA_ATTRIBUTE_KEY,
                str::from_utf8(&attr.packet_data).map_err(|_| ChannelError::NonUtf8PacketData)?,
            )
                .into(),
            (
                PKT_DATA_HEX_ATTRIBUTE_KEY,
                String::from_utf8(hex::encode(attr.packet_data)).unwrap(),
            )
                .into(),
        ];

        Ok(tags)
    }
}

#[cfg_attr(
    feature = "parity-scale-codec",
    derive(
        parity_scale_codec::Encode,
        parity_scale_codec::Decode,
        scale_info::TypeInfo
    )
)]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, From, PartialEq, Eq)]
pub struct TimeoutHeightAttribute {
    pub timeout_height: TimeoutHeight,
}

impl From<TimeoutHeightAttribute> for abci::EventAttribute {
    fn from(attr: TimeoutHeightAttribute) -> Self {
        match attr.timeout_height {
            TimeoutHeight::Never => (PKT_TIMEOUT_HEIGHT_ATTRIBUTE_KEY, "0-0").into(),
            TimeoutHeight::At(height) => {
                (PKT_TIMEOUT_HEIGHT_ATTRIBUTE_KEY, height.to_string()).into()
            }
        }
    }
}

#[cfg_attr(
    feature = "parity-scale-codec",
    derive(
        parity_scale_codec::Encode,
        parity_scale_codec::Decode,
        scale_info::TypeInfo
    )
)]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, From, PartialEq, Eq)]
pub struct TimeoutTimestampAttribute {
    pub timeout_timestamp: Timestamp,
}

impl From<TimeoutTimestampAttribute> for abci::EventAttribute {
    fn from(attr: TimeoutTimestampAttribute) -> Self {
        (
            PKT_TIMEOUT_TIMESTAMP_ATTRIBUTE_KEY,
            attr.timeout_timestamp.nanoseconds().to_string(),
        )
            .into()
    }
}

#[cfg_attr(
    feature = "parity-scale-codec",
    derive(
        parity_scale_codec::Encode,
        parity_scale_codec::Decode,
        scale_info::TypeInfo
    )
)]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, From, PartialEq, Eq)]
pub struct SequenceAttribute {
    pub sequence: Sequence,
}

impl From<SequenceAttribute> for abci::EventAttribute {
    fn from(attr: SequenceAttribute) -> Self {
        (PKT_SEQ_ATTRIBUTE_KEY, attr.sequence.to_string()).into()
    }
}

#[cfg_attr(
    feature = "parity-scale-codec",
    derive(
        parity_scale_codec::Encode,
        parity_scale_codec::Decode,
        scale_info::TypeInfo
    )
)]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, From, PartialEq, Eq)]
pub struct SrcPortIdAttribute {
    pub src_port_id: PortId,
}

impl From<SrcPortIdAttribute> for abci::EventAttribute {
    fn from(attr: SrcPortIdAttribute) -> Self {
        (PKT_SRC_PORT_ATTRIBUTE_KEY, attr.src_port_id.as_str()).into()
    }
}

#[cfg_attr(
    feature = "parity-scale-codec",
    derive(
        parity_scale_codec::Encode,
        parity_scale_codec::Decode,
        scale_info::TypeInfo
    )
)]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, From, PartialEq, Eq)]
pub struct SrcChannelIdAttribute {
    pub src_channel_id: ChannelId,
}

impl From<SrcChannelIdAttribute> for abci::EventAttribute {
    fn from(attr: SrcChannelIdAttribute) -> Self {
        (PKT_SRC_CHANNEL_ATTRIBUTE_KEY, attr.src_channel_id.as_str()).into()
    }
}

#[cfg_attr(
    feature = "parity-scale-codec",
    derive(
        parity_scale_codec::Encode,
        parity_scale_codec::Decode,
        scale_info::TypeInfo
    )
)]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, From, PartialEq, Eq)]
pub struct DstPortIdAttribute {
    pub dst_port_id: PortId,
}

impl From<DstPortIdAttribute> for abci::EventAttribute {
    fn from(attr: DstPortIdAttribute) -> Self {
        (PKT_DST_PORT_ATTRIBUTE_KEY, attr.dst_port_id.as_str()).into()
    }
}

#[cfg_attr(
    feature = "parity-scale-codec",
    derive(
        parity_scale_codec::Encode,
        parity_scale_codec::Decode,
        scale_info::TypeInfo
    )
)]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, From, PartialEq, Eq)]
pub struct DstChannelIdAttribute {
    pub dst_channel_id: ChannelId,
}

impl From<DstChannelIdAttribute> for abci::EventAttribute {
    fn from(attr: DstChannelIdAttribute) -> Self {
        (PKT_DST_CHANNEL_ATTRIBUTE_KEY, attr.dst_channel_id.as_str()).into()
    }
}

#[cfg_attr(
    feature = "parity-scale-codec",
    derive(
        parity_scale_codec::Encode,
        parity_scale_codec::Decode,
        scale_info::TypeInfo
    )
)]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, From, PartialEq, Eq)]
pub struct ChannelOrderingAttribute {
    pub order: Order,
}

impl From<ChannelOrderingAttribute> for abci::EventAttribute {
    fn from(attr: ChannelOrderingAttribute) -> Self {
        (PKT_CHANNEL_ORDERING_ATTRIBUTE_KEY, attr.order.as_str()).into()
    }
}

#[cfg_attr(
    feature = "parity-scale-codec",
    derive(
        parity_scale_codec::Encode,
        parity_scale_codec::Decode,
        scale_info::TypeInfo
    )
)]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, From, PartialEq, Eq)]
pub struct PacketConnectionIdAttribute {
    pub connection_id: ConnectionId,
}

impl From<PacketConnectionIdAttribute> for abci::EventAttribute {
    fn from(attr: PacketConnectionIdAttribute) -> Self {
        (PKT_CONNECTION_ID_ATTRIBUTE_KEY, attr.connection_id.as_str()).into()
    }
}

#[cfg_attr(
    feature = "parity-scale-codec",
    derive(
        parity_scale_codec::Encode,
        parity_scale_codec::Decode,
        scale_info::TypeInfo
    )
)]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, From, PartialEq, Eq)]
pub struct AcknowledgementAttribute {
    pub acknowledgement: Acknowledgement,
}

impl TryFrom<AcknowledgementAttribute> for Vec<abci::EventAttribute> {
    type Error = ChannelError;

    fn try_from(attr: AcknowledgementAttribute) -> Result<Self, Self::Error> {
        let tags = vec![
            (
                PKT_ACK_ATTRIBUTE_KEY,
                // Note: this attribute forces us to assume that Packet data
                // is valid UTF-8, even though the standard doesn't require
                // it. It has been deprecated in ibc-go. It will be removed
                // in the future.
                str::from_utf8(attr.acknowledgement.as_bytes())
                    .map_err(|_| ChannelError::NonUtf8PacketData)?,
            )
                .into(),
            (
                PKT_ACK_HEX_ATTRIBUTE_KEY,
                String::from_utf8(hex::encode(attr.acknowledgement)).unwrap(),
            )
                .into(),
        ];

        Ok(tags)
    }
}
