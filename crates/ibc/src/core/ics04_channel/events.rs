//! Types for the IBC events emitted from Tendermint Websocket by the channels module.

use derive_more::From;
use serde_derive::Serialize;
use subtle_encoding::hex;
use tendermint::abci::tag::Tag;
use tendermint::abci::Event as AbciEvent;

use crate::core::ics04_channel::error::Error;
use crate::core::ics04_channel::packet::Packet;
use crate::core::ics24_host::identifier::{ChannelId, ConnectionId, PortId};
use crate::events::{IbcEvent, IbcEventType};
use crate::prelude::*;
use crate::timestamp::Timestamp;

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
pub const PKT_TIMEOUT_HEIGHT_ATTRIBUTE_KEY: &str = "packet_timeout_height";
pub const PKT_TIMEOUT_TIMESTAMP_ATTRIBUTE_KEY: &str = "packet_timeout_timestamp";
pub const PKT_ACK_ATTRIBUTE_KEY: &str = "packet_ack";

/// Convert attributes to Tendermint ABCI tags
///
/// # Note
/// The parsing of `Key`s and `Value`s never fails, because the
/// `FromStr` instance of `tendermint::abci::tag::{Key, Value}`
/// is infallible, even if it is not represented in the error type.
/// Once tendermint-rs improves the API of the `Key` and `Value` types,
/// we will be able to remove the `.parse().unwrap()` calls.
impl TryFrom<Packet> for Vec<Tag> {
    type Error = Error;
    fn try_from(p: Packet) -> Result<Self, Self::Error> {
        let mut attributes = vec![];
        let src_port = Tag {
            key: PKT_SRC_PORT_ATTRIBUTE_KEY.parse().unwrap(),
            value: p.source_port.to_string().parse().unwrap(),
        };
        attributes.push(src_port);
        let src_channel = Tag {
            key: PKT_SRC_CHANNEL_ATTRIBUTE_KEY.parse().unwrap(),
            value: p.source_channel.to_string().parse().unwrap(),
        };
        attributes.push(src_channel);
        let dst_port = Tag {
            key: PKT_DST_PORT_ATTRIBUTE_KEY.parse().unwrap(),
            value: p.destination_port.to_string().parse().unwrap(),
        };
        attributes.push(dst_port);
        let dst_channel = Tag {
            key: PKT_DST_CHANNEL_ATTRIBUTE_KEY.parse().unwrap(),
            value: p.destination_channel.to_string().parse().unwrap(),
        };
        attributes.push(dst_channel);
        let sequence = Tag {
            key: PKT_SEQ_ATTRIBUTE_KEY.parse().unwrap(),
            value: p.sequence.to_string().parse().unwrap(),
        };
        attributes.push(sequence);
        let timeout_height = Tag {
            key: PKT_TIMEOUT_HEIGHT_ATTRIBUTE_KEY.parse().unwrap(),
            value: p.timeout_height.into(),
        };
        attributes.push(timeout_height);
        let timeout_timestamp = Tag {
            key: PKT_TIMEOUT_TIMESTAMP_ATTRIBUTE_KEY.parse().unwrap(),
            value: p
                .timeout_timestamp
                .nanoseconds()
                .to_string()
                .parse()
                .unwrap(),
        };
        attributes.push(timeout_timestamp);
        let val =
            String::from_utf8(p.data).expect("hex-encoded string should always be valid UTF-8");
        let packet_data = Tag {
            key: PKT_DATA_ATTRIBUTE_KEY.parse().unwrap(),
            value: val.parse().unwrap(),
        };
        attributes.push(packet_data);
        let ack = Tag {
            key: PKT_ACK_ATTRIBUTE_KEY.parse().unwrap(),
            value: "".parse().unwrap(),
        };
        attributes.push(ack);
        Ok(attributes)
    }
}

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

#[derive(Debug, From)]
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
            port_id: PortIdAttribute::from(port_id),
            channel_id: ChannelIdAttribute::from(channel_id),
            counterparty_port_id: PortIdAttribute::from(counterparty_port_id),
            connection_id: ConnectionIdAttribute::from(connection_id),
            version: VersionAttribute::from(version),
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
            port_id: PortIdAttribute::from(port_id),
            channel_id: ChannelIdAttribute::from(channel_id),
            counterparty_port_id: PortIdAttribute::from(counterparty_port_id),
            counterparty_channel_id: ChannelIdAttribute::from(counterparty_channel_id),
            connection_id: ConnectionIdAttribute::from(connection_id),
            version: VersionAttribute::from(version),
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
            port_id: PortIdAttribute::from(port_id),
            channel_id: ChannelIdAttribute::from(channel_id),
            counterparty_port_id: PortIdAttribute::from(counterparty_port_id),
            counterparty_channel_id: ChannelIdAttribute::from(counterparty_channel_id),
            connection_id: ConnectionIdAttribute::from(connection_id),
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
            port_id: PortIdAttribute::from(port_id),
            channel_id: ChannelIdAttribute::from(channel_id),
            counterparty_port_id: PortIdAttribute::from(counterparty_port_id),
            counterparty_channel_id: ChannelIdAttribute::from(counterparty_channel_id),
            connection_id: ConnectionIdAttribute::from(connection_id),
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
            port_id: PortIdAttribute::from(port_id),
            channel_id: ChannelIdAttribute::from(channel_id),
            counterparty_port_id: PortIdAttribute::from(counterparty_port_id),
            counterparty_channel_id: ChannelIdAttribute::from(counterparty_channel_id),
            connection_id: ConnectionIdAttribute::from(connection_id),
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
            port_id: PortIdAttribute::from(port_id),
            channel_id: ChannelIdAttribute::from(channel_id),
            counterparty_port_id: PortIdAttribute::from(counterparty_port_id),
            counterparty_channel_id: ChannelIdAttribute::from(counterparty_channel_id),
            connection_id: ConnectionIdAttribute::from(connection_id),
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

#[deprecated]
#[derive(Debug, From)]
struct DataAttribute {
    data: Vec<u8>,
}

impl TryFrom<DataAttribute> for Tag {
    type Error = Error;

    fn try_from(attr: DataAttribute) -> Result<Self, Self::Error> {
        Ok(Tag {
            key: PKT_DATA_ATTRIBUTE_KEY.parse().unwrap(),
            value: String::from_utf8(attr.data)
                // TODO: use error defined in v0.21.0
                .map_err(|_| Error::invalid_packet())?
                .parse()
                .unwrap(),
        })
    }
}

#[derive(Debug, From)]
struct DataHexAttribute {
    data: Vec<u8>,
}

impl From<DataHexAttribute> for Tag {
    fn from(attr: DataHexAttribute) -> Self {
        Tag {
            key: PKT_DATA_HEX_ATTRIBUTE_KEY.parse().unwrap(),
            value: String::from_utf8(hex::encode(attr.data))
                .unwrap()
                .parse()
                .unwrap(),
        }
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SendPacket {
    pub packet: Packet,
}

impl SendPacket {
    pub fn src_port_id(&self) -> &PortId {
        &self.packet.source_port
    }
    pub fn src_channel_id(&self) -> &ChannelId {
        &self.packet.source_channel
    }
    pub fn dst_port_id(&self) -> &PortId {
        &self.packet.destination_port
    }
    pub fn dst_channel_id(&self) -> &ChannelId {
        &self.packet.destination_channel
    }
}

impl From<SendPacket> for IbcEvent {
    fn from(v: SendPacket) -> Self {
        IbcEvent::SendPacket(v)
    }
}

impl TryFrom<SendPacket> for AbciEvent {
    type Error = Error;

    fn try_from(v: SendPacket) -> Result<Self, Self::Error> {
        let attributes = Vec::<Tag>::try_from(v.packet)?;
        Ok(AbciEvent {
            type_str: IbcEventType::SendPacket.as_str().to_string(),
            attributes,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ReceivePacket {
    pub packet: Packet,
}

impl ReceivePacket {
    pub fn src_port_id(&self) -> &PortId {
        &self.packet.source_port
    }
    pub fn src_channel_id(&self) -> &ChannelId {
        &self.packet.source_channel
    }
    pub fn dst_port_id(&self) -> &PortId {
        &self.packet.destination_port
    }
    pub fn dst_channel_id(&self) -> &ChannelId {
        &self.packet.destination_channel
    }
}

impl From<ReceivePacket> for IbcEvent {
    fn from(v: ReceivePacket) -> Self {
        IbcEvent::ReceivePacket(v)
    }
}

impl TryFrom<ReceivePacket> for AbciEvent {
    type Error = Error;

    fn try_from(v: ReceivePacket) -> Result<Self, Self::Error> {
        let attributes = Vec::<Tag>::try_from(v.packet)?;
        Ok(AbciEvent {
            type_str: IbcEventType::ReceivePacket.as_str().to_string(),
            attributes,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct WriteAcknowledgement {
    pub packet: Packet,
    #[serde(serialize_with = "crate::serializers::ser_hex_upper")]
    pub ack: Vec<u8>,
}

impl WriteAcknowledgement {
    pub fn src_port_id(&self) -> &PortId {
        &self.packet.source_port
    }
    pub fn src_channel_id(&self) -> &ChannelId {
        &self.packet.source_channel
    }
    pub fn dst_port_id(&self) -> &PortId {
        &self.packet.destination_port
    }
    pub fn dst_channel_id(&self) -> &ChannelId {
        &self.packet.destination_channel
    }
}

impl From<WriteAcknowledgement> for IbcEvent {
    fn from(v: WriteAcknowledgement) -> Self {
        IbcEvent::WriteAcknowledgement(v)
    }
}

impl TryFrom<WriteAcknowledgement> for AbciEvent {
    type Error = Error;

    fn try_from(v: WriteAcknowledgement) -> Result<Self, Self::Error> {
        let mut attributes = Vec::<Tag>::try_from(v.packet)?;
        let val =
            String::from_utf8(v.ack).expect("hex-encoded string should always be valid UTF-8");
        // No actual conversion from string to `Tag::Key` or `Tag::Value`
        let ack = Tag {
            key: PKT_ACK_ATTRIBUTE_KEY.parse().unwrap(),
            value: val.parse().unwrap(),
        };
        attributes.push(ack);
        Ok(AbciEvent {
            type_str: IbcEventType::WriteAck.as_str().to_string(),
            attributes,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct AcknowledgePacket {
    pub packet: Packet,
}

impl AcknowledgePacket {
    pub fn src_port_id(&self) -> &PortId {
        &self.packet.source_port
    }
    pub fn src_channel_id(&self) -> &ChannelId {
        &self.packet.source_channel
    }
}

impl From<AcknowledgePacket> for IbcEvent {
    fn from(v: AcknowledgePacket) -> Self {
        IbcEvent::AcknowledgePacket(v)
    }
}

impl TryFrom<AcknowledgePacket> for AbciEvent {
    type Error = Error;

    fn try_from(v: AcknowledgePacket) -> Result<Self, Self::Error> {
        let attributes = Vec::<Tag>::try_from(v.packet)?;
        Ok(AbciEvent {
            type_str: IbcEventType::AckPacket.as_str().to_string(),
            attributes,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct TimeoutPacket {
    pub packet: Packet,
}

impl TimeoutPacket {
    pub fn src_port_id(&self) -> &PortId {
        &self.packet.source_port
    }
    pub fn src_channel_id(&self) -> &ChannelId {
        &self.packet.source_channel
    }
    pub fn dst_port_id(&self) -> &PortId {
        &self.packet.destination_port
    }
    pub fn dst_channel_id(&self) -> &ChannelId {
        &self.packet.destination_channel
    }
}

impl From<TimeoutPacket> for IbcEvent {
    fn from(v: TimeoutPacket) -> Self {
        IbcEvent::TimeoutPacket(v)
    }
}

impl TryFrom<TimeoutPacket> for AbciEvent {
    type Error = Error;

    fn try_from(v: TimeoutPacket) -> Result<Self, Self::Error> {
        let attributes = Vec::<Tag>::try_from(v.packet)?;
        Ok(AbciEvent {
            type_str: IbcEventType::Timeout.as_str().to_string(),
            attributes,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct TimeoutOnClosePacket {
    pub packet: Packet,
}

impl TimeoutOnClosePacket {
    pub fn src_port_id(&self) -> &PortId {
        &self.packet.source_port
    }
    pub fn src_channel_id(&self) -> &ChannelId {
        &self.packet.source_channel
    }
    pub fn dst_port_id(&self) -> &PortId {
        &self.packet.destination_port
    }
    pub fn dst_channel_id(&self) -> &ChannelId {
        &self.packet.destination_channel
    }
}

impl From<TimeoutOnClosePacket> for IbcEvent {
    fn from(v: TimeoutOnClosePacket) -> Self {
        IbcEvent::TimeoutOnClosePacket(v)
    }
}

impl TryFrom<TimeoutOnClosePacket> for AbciEvent {
    type Error = Error;

    fn try_from(v: TimeoutOnClosePacket) -> Result<Self, Self::Error> {
        let attributes = Vec::<Tag>::try_from(v.packet)?;
        Ok(AbciEvent {
            type_str: IbcEventType::TimeoutOnClose.as_str().to_string(),
            attributes,
        })
    }
}
