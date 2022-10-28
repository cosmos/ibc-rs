//! Types for the IBC events emitted from Tendermint Websocket by the channels module.

use derive_more::From;
use subtle_encoding::hex;
use tendermint::abci::tag::Tag;
use tendermint::abci::Event as AbciEvent;

use crate::core::ics04_channel::error::Error;
use crate::core::ics04_channel::packet::Packet;
use crate::core::ics24_host::identifier::{ChannelId, ConnectionId, PortId};
use crate::events::IbcEventType;
use crate::prelude::*;
use crate::timestamp::Timestamp;

use super::channel::Order;
use super::msgs::acknowledgement::Acknowledgement;
use super::packet::Sequence;
use super::timeout::TimeoutHeight;
use super::Version;

/// Channel event attribute keys
pub const CONNECTION_ID_ATTRIBUTE_KEY: &str = "connection_id";
pub const CHANNEL_ID_ATTRIBUTE_KEY: &str = "channel_id";
pub const PORT_ID_ATTRIBUTE_KEY: &str = "port_id";
pub const COUNTERPARTY_CHANNEL_ID_ATTRIBUTE_KEY: &str = "counterparty_channel_id";
pub const COUNTERPARTY_PORT_ID_ATTRIBUTE_KEY: &str = "counterparty_port_id";
pub const VERSION_ATTRIBUTE_KEY: &str = "version";

/// Packet event attribute keys
pub const PKT_SEQ_ATTRIBUTE_KEY: &str = "packet_sequence";
pub const PKT_DATA_ATTRIBUTE_KEY: &str = "packet_data";
pub const PKT_DATA_HEX_ATTRIBUTE_KEY: &str = "packet_data_hex";
pub const PKT_SRC_PORT_ATTRIBUTE_KEY: &str = "packet_src_port";
pub const PKT_SRC_CHANNEL_ATTRIBUTE_KEY: &str = "packet_src_channel";
pub const PKT_DST_PORT_ATTRIBUTE_KEY: &str = "packet_dst_port";
pub const PKT_DST_CHANNEL_ATTRIBUTE_KEY: &str = "packet_dst_channel";
pub const PKT_CHANNEL_ORDERING_ATTRIBUTE_KEY: &str = "packet_channel_ordering";
pub const PKT_TIMEOUT_HEIGHT_ATTRIBUTE_KEY: &str = "packet_timeout_height";
pub const PKT_TIMEOUT_TIMESTAMP_ATTRIBUTE_KEY: &str = "packet_timeout_timestamp";
pub const PKT_ACK_ATTRIBUTE_KEY: &str = "packet_ack";
pub const PKT_ACK_HEX_ATTRIBUTE_KEY: &str = "packet_ack_hex";
pub const PKT_CONNECTION_ID_ATTRIBUTE_KEY: &str = "packet_connection";

#[derive(Debug, From)]
struct PortIdAttribute {
    port_id: PortId,
}

impl From<PortIdAttribute> for Tag {
    fn from(attr: PortIdAttribute) -> Self {
        Tag {
            key: PORT_ID_ATTRIBUTE_KEY.parse().unwrap(),
            value: attr.port_id.to_string().parse().unwrap(),
        }
    }
}

#[derive(Clone, Debug, From)]
struct ChannelIdAttribute {
    channel_id: ChannelId,
}

impl From<ChannelIdAttribute> for Tag {
    fn from(attr: ChannelIdAttribute) -> Self {
        Tag {
            key: CHANNEL_ID_ATTRIBUTE_KEY.parse().unwrap(),
            value: attr.channel_id.to_string().parse().unwrap(),
        }
    }
}

#[derive(Debug, From)]
struct CounterpartyPortIdAttribute {
    counterparty_port_id: PortId,
}

impl From<CounterpartyPortIdAttribute> for Tag {
    fn from(attr: CounterpartyPortIdAttribute) -> Self {
        Tag {
            key: COUNTERPARTY_PORT_ID_ATTRIBUTE_KEY.parse().unwrap(),
            value: attr.counterparty_port_id.to_string().parse().unwrap(),
        }
    }
}

#[derive(Debug, From)]
struct CounterpartyChannelIdAttribute {
    counterparty_channel_id: ChannelId,
}

impl From<CounterpartyChannelIdAttribute> for Tag {
    fn from(attr: CounterpartyChannelIdAttribute) -> Self {
        Tag {
            key: COUNTERPARTY_CHANNEL_ID_ATTRIBUTE_KEY.parse().unwrap(),
            value: attr.counterparty_channel_id.to_string().parse().unwrap(),
        }
    }
}

#[derive(Debug, From)]
struct ConnectionIdAttribute {
    connection_id: ConnectionId,
}

impl From<ConnectionIdAttribute> for Tag {
    fn from(attr: ConnectionIdAttribute) -> Self {
        Tag {
            key: CONNECTION_ID_ATTRIBUTE_KEY.parse().unwrap(),
            value: attr.connection_id.to_string().parse().unwrap(),
        }
    }
}

#[derive(Debug, From)]
struct VersionAttribute {
    version: Version,
}

impl From<VersionAttribute> for Tag {
    fn from(attr: VersionAttribute) -> Self {
        Tag {
            key: VERSION_ATTRIBUTE_KEY.parse().unwrap(),
            value: attr.version.to_string().parse().unwrap(),
        }
    }
}

#[derive(Debug)]
pub struct OpenInit {
    port_id: PortIdAttribute,
    channel_id: ChannelIdAttribute,
    counterparty_port_id: PortIdAttribute,
    connection_id: ConnectionIdAttribute,
    version: VersionAttribute,
}

impl OpenInit {
    pub fn new(
        port_id: PortId,
        channel_id: ChannelId,
        counterparty_port_id: PortId,
        connection_id: ConnectionId,
        version: Version,
    ) -> Self {
        Self {
            port_id: port_id.into(),
            channel_id: channel_id.into(),
            counterparty_port_id: counterparty_port_id.into(),
            connection_id: connection_id.into(),
            version: version.into(),
        }
    }
    pub fn port_id(&self) -> &PortId {
        &self.port_id.port_id
    }
    pub fn channel_id(&self) -> &ChannelId {
        &self.channel_id.channel_id
    }
    pub fn counterparty_port_id(&self) -> &PortId {
        &self.counterparty_port_id.port_id
    }
    pub fn connection_id(&self) -> &ConnectionId {
        &self.connection_id.connection_id
    }
    pub fn version(&self) -> &Version {
        &self.version.version
    }
}

impl From<OpenInit> for AbciEvent {
    fn from(o: OpenInit) -> Self {
        AbciEvent {
            type_str: IbcEventType::OpenInitChannel.as_str().to_string(),
            attributes: vec![
                o.port_id.into(),
                o.channel_id.into(),
                o.counterparty_port_id.into(),
                Tag {
                    key: COUNTERPARTY_CHANNEL_ID_ATTRIBUTE_KEY.parse().unwrap(),
                    value: String::from("").parse().unwrap(),
                },
                o.connection_id.into(),
                o.version.into(),
            ],
        }
    }
}

#[derive(Debug)]
pub struct OpenTry {
    port_id: PortIdAttribute,
    channel_id: ChannelIdAttribute,
    counterparty_port_id: PortIdAttribute,
    counterparty_channel_id: ChannelIdAttribute,
    connection_id: ConnectionIdAttribute,
    version: VersionAttribute,
}

impl OpenTry {
    pub fn new(
        port_id: PortId,
        channel_id: ChannelId,
        counterparty_port_id: PortId,
        counterparty_channel_id: ChannelId,
        connection_id: ConnectionId,
        version: Version,
    ) -> Self {
        Self {
            port_id: port_id.into(),
            channel_id: channel_id.into(),
            counterparty_port_id: counterparty_port_id.into(),
            counterparty_channel_id: counterparty_channel_id.into(),
            connection_id: connection_id.into(),
            version: version.into(),
        }
    }
    pub fn port_id(&self) -> &PortId {
        &self.port_id.port_id
    }
    pub fn channel_id(&self) -> &ChannelId {
        &self.channel_id.channel_id
    }
    pub fn counterparty_port_id(&self) -> &PortId {
        &self.counterparty_port_id.port_id
    }
    pub fn counterparty_channel_id(&self) -> &ChannelId {
        &self.counterparty_channel_id.channel_id
    }
    pub fn connection_id(&self) -> &ConnectionId {
        &self.connection_id.connection_id
    }
    pub fn version(&self) -> &Version {
        &self.version.version
    }
}

impl From<OpenTry> for AbciEvent {
    fn from(o: OpenTry) -> Self {
        AbciEvent {
            type_str: IbcEventType::OpenTryChannel.as_str().to_string(),
            attributes: vec![
                o.port_id.into(),
                o.channel_id.into(),
                o.counterparty_port_id.into(),
                o.counterparty_channel_id.into(),
                o.connection_id.into(),
                o.version.into(),
            ],
        }
    }
}

#[derive(Debug)]
pub struct OpenAck {
    port_id: PortIdAttribute,
    channel_id: ChannelIdAttribute,
    counterparty_port_id: PortIdAttribute,
    counterparty_channel_id: ChannelIdAttribute,
    connection_id: ConnectionIdAttribute,
}

impl OpenAck {
    pub fn new(
        port_id: PortId,
        channel_id: ChannelId,
        counterparty_port_id: PortId,
        counterparty_channel_id: ChannelId,
        connection_id: ConnectionId,
    ) -> Self {
        Self {
            port_id: port_id.into(),
            channel_id: channel_id.into(),
            counterparty_port_id: counterparty_port_id.into(),
            counterparty_channel_id: counterparty_channel_id.into(),
            connection_id: connection_id.into(),
        }
    }
    pub fn port_id(&self) -> &PortId {
        &self.port_id.port_id
    }
    pub fn channel_id(&self) -> &ChannelId {
        &self.channel_id.channel_id
    }
    pub fn counterparty_port_id(&self) -> &PortId {
        &self.counterparty_port_id.port_id
    }
    pub fn counterparty_channel_id(&self) -> &ChannelId {
        &self.counterparty_channel_id.channel_id
    }
    pub fn connection_id(&self) -> &ConnectionId {
        &self.connection_id.connection_id
    }
}

impl From<OpenAck> for AbciEvent {
    fn from(o: OpenAck) -> Self {
        AbciEvent {
            type_str: IbcEventType::OpenAckChannel.as_str().to_string(),
            attributes: vec![
                o.port_id.into(),
                o.channel_id.into(),
                o.counterparty_port_id.into(),
                o.counterparty_channel_id.into(),
                o.connection_id.into(),
            ],
        }
    }
}

#[derive(Debug)]
pub struct OpenConfirm {
    port_id: PortIdAttribute,
    channel_id: ChannelIdAttribute,
    counterparty_port_id: PortIdAttribute,
    counterparty_channel_id: ChannelIdAttribute,
    connection_id: ConnectionIdAttribute,
}

impl OpenConfirm {
    pub fn new(
        port_id: PortId,
        channel_id: ChannelId,
        counterparty_port_id: PortId,
        counterparty_channel_id: ChannelId,
        connection_id: ConnectionId,
    ) -> Self {
        Self {
            port_id: port_id.into(),
            channel_id: channel_id.into(),
            counterparty_port_id: counterparty_port_id.into(),
            counterparty_channel_id: counterparty_channel_id.into(),
            connection_id: connection_id.into(),
        }
    }
    pub fn port_id(&self) -> &PortId {
        &self.port_id.port_id
    }
    pub fn channel_id(&self) -> &ChannelId {
        &self.channel_id.channel_id
    }
    pub fn counterparty_port_id(&self) -> &PortId {
        &self.counterparty_port_id.port_id
    }
    pub fn counterparty_channel_id(&self) -> &ChannelId {
        &self.counterparty_channel_id.channel_id
    }
    pub fn connection_id(&self) -> &ConnectionId {
        &self.connection_id.connection_id
    }
}

impl From<OpenConfirm> for AbciEvent {
    fn from(o: OpenConfirm) -> Self {
        AbciEvent {
            type_str: IbcEventType::OpenConfirmChannel.as_str().to_string(),
            attributes: vec![
                o.port_id.into(),
                o.channel_id.into(),
                o.counterparty_port_id.into(),
                o.counterparty_channel_id.into(),
                o.connection_id.into(),
            ],
        }
    }
}

#[derive(Debug)]
pub struct CloseInit {
    port_id: PortIdAttribute,
    channel_id: ChannelIdAttribute,
    counterparty_port_id: PortIdAttribute,
    counterparty_channel_id: ChannelIdAttribute,
    connection_id: ConnectionIdAttribute,
}

impl CloseInit {
    pub fn new(
        port_id: PortId,
        channel_id: ChannelId,
        counterparty_port_id: PortId,
        counterparty_channel_id: ChannelId,
        connection_id: ConnectionId,
    ) -> Self {
        Self {
            port_id: port_id.into(),
            channel_id: channel_id.into(),
            counterparty_port_id: counterparty_port_id.into(),
            counterparty_channel_id: counterparty_channel_id.into(),
            connection_id: connection_id.into(),
        }
    }
    pub fn port_id(&self) -> &PortId {
        &self.port_id.port_id
    }
    pub fn channel_id(&self) -> &ChannelId {
        &self.channel_id.channel_id
    }
    pub fn counterparty_port_id(&self) -> &PortId {
        &self.counterparty_port_id.port_id
    }
    pub fn counterparty_channel_id(&self) -> &ChannelId {
        &self.counterparty_channel_id.channel_id
    }
    pub fn connection_id(&self) -> &ConnectionId {
        &self.connection_id.connection_id
    }
}

impl From<CloseInit> for AbciEvent {
    fn from(o: CloseInit) -> Self {
        AbciEvent {
            type_str: IbcEventType::CloseInitChannel.as_str().to_string(),
            attributes: vec![
                o.port_id.into(),
                o.channel_id.into(),
                o.counterparty_port_id.into(),
                o.counterparty_channel_id.into(),
                o.connection_id.into(),
            ],
        }
    }
}

#[derive(Debug)]
pub struct CloseConfirm {
    port_id: PortIdAttribute,
    channel_id: ChannelIdAttribute,
    counterparty_port_id: PortIdAttribute,
    counterparty_channel_id: ChannelIdAttribute,
    connection_id: ConnectionIdAttribute,
}

impl CloseConfirm {
    pub fn new(
        port_id: PortId,
        channel_id: ChannelId,
        counterparty_port_id: PortId,
        counterparty_channel_id: ChannelId,
        connection_id: ConnectionId,
    ) -> Self {
        Self {
            port_id: port_id.into(),
            channel_id: channel_id.into(),
            counterparty_port_id: counterparty_port_id.into(),
            counterparty_channel_id: counterparty_channel_id.into(),
            connection_id: connection_id.into(),
        }
    }
    pub fn port_id(&self) -> &PortId {
        &self.port_id.port_id
    }
    pub fn channel_id(&self) -> &ChannelId {
        &self.channel_id.channel_id
    }
    pub fn counterparty_port_id(&self) -> &PortId {
        &self.counterparty_port_id.port_id
    }
    pub fn counterparty_channel_id(&self) -> &ChannelId {
        &self.counterparty_channel_id.channel_id
    }
    pub fn connection_id(&self) -> &ConnectionId {
        &self.connection_id.connection_id
    }
}

impl From<CloseConfirm> for AbciEvent {
    fn from(o: CloseConfirm) -> Self {
        AbciEvent {
            type_str: IbcEventType::CloseConfirmChannel.as_str().to_string(),
            attributes: vec![
                o.port_id.into(),
                o.channel_id.into(),
                o.counterparty_port_id.into(),
                o.counterparty_channel_id.into(),
                o.connection_id.into(),
            ],
        }
    }
}

/// A `ChannelClosed` event is emitted when a channel is closed as a result of a packet timing out. Note that
/// since optimistic packet sends (i.e. send a packet before channel handshake is complete) are supported,
/// we might not have a counterparty channel id value yet. This would happen if a packet is sent right
/// after a `ChannelOpenInit` message.
#[derive(Debug)]
pub struct ChannelClosed {
    port_id: PortIdAttribute,
    channel_id: ChannelIdAttribute,
    counterparty_port_id: PortIdAttribute,
    maybe_counterparty_channel_id: Option<ChannelIdAttribute>,
    connection_id: ConnectionIdAttribute,
    channel_ordering: ChannelOrderingAttribute,
}

impl ChannelClosed {
    pub fn new(
        port_id: PortId,
        channel_id: ChannelId,
        counterparty_port_id: PortId,
        maybe_counterparty_channel_id: Option<ChannelId>,
        connection_id: ConnectionId,
        channel_ordering: Order,
    ) -> Self {
        Self {
            port_id: port_id.into(),
            channel_id: channel_id.into(),
            counterparty_port_id: counterparty_port_id.into(),
            maybe_counterparty_channel_id: maybe_counterparty_channel_id.map(|c| c.into()),
            connection_id: connection_id.into(),
            channel_ordering: channel_ordering.into(),
        }
    }
    pub fn port_id(&self) -> &PortId {
        &self.port_id.port_id
    }
    pub fn channel_id(&self) -> &ChannelId {
        &self.channel_id.channel_id
    }
    pub fn counterparty_port_id(&self) -> &PortId {
        &self.counterparty_port_id.port_id
    }
    pub fn counterparty_channel_id(&self) -> Option<ChannelId> {
        self.maybe_counterparty_channel_id
            .clone()
            .map(|c| c.channel_id)
    }
    pub fn connection_id(&self) -> &ConnectionId {
        &self.connection_id.connection_id
    }
    pub fn channel_ordering(&self) -> &Order {
        &self.channel_ordering.order
    }
}

impl From<ChannelClosed> for AbciEvent {
    fn from(ev: ChannelClosed) -> Self {
        AbciEvent {
            type_str: IbcEventType::ChannelClosed.as_str().to_string(),
            attributes: vec![
                ev.port_id.into(),
                ev.channel_id.into(),
                ev.counterparty_port_id.into(),
                ev.maybe_counterparty_channel_id.map_or(
                    Tag {
                        key: COUNTERPARTY_CHANNEL_ID_ATTRIBUTE_KEY.parse().unwrap(),
                        value: "".parse().unwrap(),
                    },
                    |c| c.into(),
                ),
                ev.connection_id.into(),
                ev.channel_ordering.into(),
            ],
        }
    }
}

#[derive(Debug, From)]
struct PacketDataAttribute {
    packet_data: Vec<u8>,
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
struct TimeoutHeightAttribute {
    timeout_height: TimeoutHeight,
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
struct TimeoutTimestampAttribute {
    timeout_timestamp: Timestamp,
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
struct SequenceAttribute {
    sequence: Sequence,
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
struct SrcPortIdAttribute {
    src_port_id: PortId,
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
struct SrcChannelIdAttribute {
    src_channel_id: ChannelId,
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
struct DstPortIdAttribute {
    dst_port_id: PortId,
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
struct DstChannelIdAttribute {
    dst_channel_id: ChannelId,
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
struct ChannelOrderingAttribute {
    order: Order,
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
struct PacketConnectionIdAttribute {
    connection_id: ConnectionId,
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
struct AcknowledgementAttribute {
    acknowledgement: Acknowledgement,
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

#[derive(Debug)]
pub struct SendPacket {
    packet_data: PacketDataAttribute,
    timeout_height: TimeoutHeightAttribute,
    timeout_timestamp: TimeoutTimestampAttribute,
    sequence: SequenceAttribute,
    src_port_id: SrcPortIdAttribute,
    src_channel_id: SrcChannelIdAttribute,
    dst_port_id: DstPortIdAttribute,
    dst_channel_id: DstChannelIdAttribute,
    channel_ordering: ChannelOrderingAttribute,
    src_connection_id: PacketConnectionIdAttribute,
}

impl SendPacket {
    pub fn new(packet: Packet, channel_ordering: Order, src_connection_id: ConnectionId) -> Self {
        Self {
            packet_data: packet.data.into(),
            timeout_height: packet.timeout_height.into(),
            timeout_timestamp: packet.timeout_timestamp.into(),
            sequence: packet.sequence.into(),
            src_port_id: packet.source_port.into(),
            src_channel_id: packet.source_channel.into(),
            dst_port_id: packet.destination_port.into(),
            dst_channel_id: packet.destination_channel.into(),
            channel_ordering: channel_ordering.into(),
            src_connection_id: src_connection_id.into(),
        }
    }
}

impl TryFrom<SendPacket> for AbciEvent {
    type Error = Error;

    fn try_from(v: SendPacket) -> Result<Self, Self::Error> {
        let mut attributes = Vec::with_capacity(11);
        attributes.append(&mut v.packet_data.try_into()?);
        attributes.push(v.timeout_height.into());
        attributes.push(v.timeout_timestamp.into());
        attributes.push(v.sequence.into());
        attributes.push(v.src_port_id.into());
        attributes.push(v.src_channel_id.into());
        attributes.push(v.dst_port_id.into());
        attributes.push(v.dst_channel_id.into());
        attributes.push(v.channel_ordering.into());
        attributes.push(v.src_connection_id.into());

        Ok(AbciEvent {
            type_str: IbcEventType::SendPacket.as_str().to_string(),
            attributes,
        })
    }
}

#[derive(Debug)]
pub struct ReceivePacket {
    packet_data: PacketDataAttribute,
    timeout_height: TimeoutHeightAttribute,
    timeout_timestamp: TimeoutTimestampAttribute,
    sequence: SequenceAttribute,
    src_port_id: SrcPortIdAttribute,
    src_channel_id: SrcChannelIdAttribute,
    dst_port_id: DstPortIdAttribute,
    dst_channel_id: DstChannelIdAttribute,
    channel_ordering: ChannelOrderingAttribute,
    dst_connection_id: PacketConnectionIdAttribute,
}

impl ReceivePacket {
    pub fn new(packet: Packet, channel_ordering: Order, dst_connection_id: ConnectionId) -> Self {
        Self {
            packet_data: packet.data.into(),
            timeout_height: packet.timeout_height.into(),
            timeout_timestamp: packet.timeout_timestamp.into(),
            sequence: packet.sequence.into(),
            src_port_id: packet.source_port.into(),
            src_channel_id: packet.source_channel.into(),
            dst_port_id: packet.destination_port.into(),
            dst_channel_id: packet.destination_channel.into(),
            channel_ordering: channel_ordering.into(),
            dst_connection_id: dst_connection_id.into(),
        }
    }
}

impl TryFrom<ReceivePacket> for AbciEvent {
    type Error = Error;

    fn try_from(v: ReceivePacket) -> Result<Self, Self::Error> {
        let mut attributes = Vec::with_capacity(11);
        attributes.append(&mut v.packet_data.try_into()?);
        attributes.push(v.timeout_height.into());
        attributes.push(v.timeout_timestamp.into());
        attributes.push(v.sequence.into());
        attributes.push(v.src_port_id.into());
        attributes.push(v.src_channel_id.into());
        attributes.push(v.dst_port_id.into());
        attributes.push(v.dst_channel_id.into());
        attributes.push(v.channel_ordering.into());
        attributes.push(v.dst_connection_id.into());

        Ok(AbciEvent {
            type_str: IbcEventType::ReceivePacket.as_str().to_string(),
            attributes,
        })
    }
}

#[derive(Debug)]
pub struct WriteAcknowledgement {
    packet_data: PacketDataAttribute,
    timeout_height: TimeoutHeightAttribute,
    timeout_timestamp: TimeoutTimestampAttribute,
    sequence: SequenceAttribute,
    src_port_id: SrcPortIdAttribute,
    src_channel_id: SrcChannelIdAttribute,
    dst_port_id: DstPortIdAttribute,
    dst_channel_id: DstChannelIdAttribute,
    channel_ordering: ChannelOrderingAttribute,
    acknowledgement: AcknowledgementAttribute,
    dst_connection_id: PacketConnectionIdAttribute,
}

impl WriteAcknowledgement {
    pub fn new(
        packet: Packet,
        channel_ordering: Order,
        dst_connection_id: ConnectionId,
        acknowledgement: Acknowledgement,
    ) -> Self {
        Self {
            packet_data: packet.data.into(),
            timeout_height: packet.timeout_height.into(),
            timeout_timestamp: packet.timeout_timestamp.into(),
            sequence: packet.sequence.into(),
            src_port_id: packet.source_port.into(),
            src_channel_id: packet.source_channel.into(),
            dst_port_id: packet.destination_port.into(),
            dst_channel_id: packet.destination_channel.into(),
            channel_ordering: channel_ordering.into(),
            acknowledgement: acknowledgement.into(),
            dst_connection_id: dst_connection_id.into(),
        }
    }
}

impl TryFrom<WriteAcknowledgement> for AbciEvent {
    type Error = Error;

    fn try_from(v: WriteAcknowledgement) -> Result<Self, Self::Error> {
        let mut attributes = Vec::with_capacity(11);
        attributes.append(&mut v.packet_data.try_into()?);
        attributes.push(v.timeout_height.into());
        attributes.push(v.timeout_timestamp.into());
        attributes.push(v.sequence.into());
        attributes.push(v.src_port_id.into());
        attributes.push(v.src_channel_id.into());
        attributes.push(v.dst_port_id.into());
        attributes.push(v.dst_channel_id.into());
        attributes.push(v.channel_ordering.into());
        attributes.append(&mut v.acknowledgement.try_into()?);
        attributes.push(v.dst_connection_id.into());

        Ok(AbciEvent {
            type_str: IbcEventType::WriteAck.as_str().to_string(),
            attributes,
        })
    }
}

#[derive(Debug)]
pub struct AcknowledgePacket {
    packet_data: PacketDataAttribute,
    timeout_height: TimeoutHeightAttribute,
    timeout_timestamp: TimeoutTimestampAttribute,
    sequence: SequenceAttribute,
    src_port_id: SrcPortIdAttribute,
    src_channel_id: SrcChannelIdAttribute,
    dst_port_id: DstPortIdAttribute,
    dst_channel_id: DstChannelIdAttribute,
    channel_ordering: ChannelOrderingAttribute,
    src_connection_id: PacketConnectionIdAttribute,
}

impl AcknowledgePacket {
    pub fn new(packet: Packet, channel_ordering: Order, src_connection_id: ConnectionId) -> Self {
        Self {
            packet_data: packet.data.into(),
            timeout_height: packet.timeout_height.into(),
            timeout_timestamp: packet.timeout_timestamp.into(),
            sequence: packet.sequence.into(),
            src_port_id: packet.source_port.into(),
            src_channel_id: packet.source_channel.into(),
            dst_port_id: packet.destination_port.into(),
            dst_channel_id: packet.destination_channel.into(),
            channel_ordering: channel_ordering.into(),
            src_connection_id: src_connection_id.into(),
        }
    }
}

impl TryFrom<AcknowledgePacket> for AbciEvent {
    type Error = Error;

    fn try_from(v: AcknowledgePacket) -> Result<Self, Self::Error> {
        let mut attributes = Vec::with_capacity(11);
        attributes.append(&mut v.packet_data.try_into()?);
        attributes.push(v.timeout_height.into());
        attributes.push(v.timeout_timestamp.into());
        attributes.push(v.sequence.into());
        attributes.push(v.src_port_id.into());
        attributes.push(v.src_channel_id.into());
        attributes.push(v.dst_port_id.into());
        attributes.push(v.dst_channel_id.into());
        attributes.push(v.channel_ordering.into());
        attributes.push(v.src_connection_id.into());

        Ok(AbciEvent {
            type_str: IbcEventType::AckPacket.as_str().to_string(),
            attributes,
        })
    }
}

#[derive(Debug)]
pub struct TimeoutPacket {
    packet_data: PacketDataAttribute,
    timeout_height: TimeoutHeightAttribute,
    timeout_timestamp: TimeoutTimestampAttribute,
    sequence: SequenceAttribute,
    src_port_id: SrcPortIdAttribute,
    src_channel_id: SrcChannelIdAttribute,
    dst_port_id: DstPortIdAttribute,
    dst_channel_id: DstChannelIdAttribute,
    channel_ordering: ChannelOrderingAttribute,
    src_connection_id: PacketConnectionIdAttribute,
}

impl TimeoutPacket {
    pub fn new(packet: Packet, channel_ordering: Order, src_connection_id: ConnectionId) -> Self {
        Self {
            packet_data: packet.data.into(),
            timeout_height: packet.timeout_height.into(),
            timeout_timestamp: packet.timeout_timestamp.into(),
            sequence: packet.sequence.into(),
            src_port_id: packet.source_port.into(),
            src_channel_id: packet.source_channel.into(),
            dst_port_id: packet.destination_port.into(),
            dst_channel_id: packet.destination_channel.into(),
            channel_ordering: channel_ordering.into(),
            src_connection_id: src_connection_id.into(),
        }
    }
}

impl TryFrom<TimeoutPacket> for AbciEvent {
    type Error = Error;

    fn try_from(v: TimeoutPacket) -> Result<Self, Self::Error> {
        let mut attributes = Vec::with_capacity(11);
        attributes.append(&mut v.packet_data.try_into()?);
        attributes.push(v.timeout_height.into());
        attributes.push(v.timeout_timestamp.into());
        attributes.push(v.sequence.into());
        attributes.push(v.src_port_id.into());
        attributes.push(v.src_channel_id.into());
        attributes.push(v.dst_port_id.into());
        attributes.push(v.dst_channel_id.into());
        attributes.push(v.channel_ordering.into());
        attributes.push(v.src_connection_id.into());

        Ok(AbciEvent {
            type_str: IbcEventType::Timeout.as_str().to_string(),
            attributes,
        })
    }
}
