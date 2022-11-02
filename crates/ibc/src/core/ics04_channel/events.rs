//! Types for the IBC events emitted from Tendermint Websocket by the channels module.

mod channel_attributes;
mod packet_attributes;

use tendermint::abci::tag::Tag;
use tendermint::abci::Event as AbciEvent;

use crate::core::ics04_channel::error::Error;
use crate::core::ics04_channel::packet::Packet;
use crate::core::ics24_host::identifier::{ChannelId, ConnectionId, PortId};
use crate::events::IbcEventType;
use crate::prelude::*;

use self::channel_attributes::{
    ChannelIdAttribute, ConnectionIdAttribute, CounterpartyChannelIdAttribute,
    CounterpartyPortIdAttribute, PortIdAttribute, VersionAttribute,
    COUNTERPARTY_CHANNEL_ID_ATTRIBUTE_KEY,
};
use self::packet_attributes::{
    AcknowledgementAttribute, ChannelOrderingAttribute, DstChannelIdAttribute, DstPortIdAttribute,
    PacketConnectionIdAttribute, PacketDataAttribute, SequenceAttribute, SrcChannelIdAttribute,
    SrcPortIdAttribute, TimeoutHeightAttribute, TimeoutTimestampAttribute,
};

use super::channel::Order;
use super::msgs::acknowledgement::Acknowledgement;
use super::Version;

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
    counterparty_port_id: CounterpartyPortIdAttribute,
    maybe_counterparty_channel_id: Option<CounterpartyChannelIdAttribute>,
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
        &self.counterparty_port_id.counterparty_port_id
    }
    pub fn counterparty_channel_id(&self) -> Option<&ChannelId> {
        self.maybe_counterparty_channel_id
            .as_ref()
            .map(|c| c.as_ref())
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
    acknowledgement: AcknowledgementAttribute,
}

impl WriteAcknowledgement {
    pub fn new(packet: Packet, acknowledgement: Acknowledgement) -> Self {
        Self {
            packet_data: packet.data.into(),
            timeout_height: packet.timeout_height.into(),
            timeout_timestamp: packet.timeout_timestamp.into(),
            sequence: packet.sequence.into(),
            src_port_id: packet.source_port.into(),
            src_channel_id: packet.source_channel.into(),
            dst_port_id: packet.destination_port.into(),
            dst_channel_id: packet.destination_channel.into(),
            acknowledgement: acknowledgement.into(),
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
        attributes.append(&mut v.acknowledgement.try_into()?);

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
    timeout_height: TimeoutHeightAttribute,
    timeout_timestamp: TimeoutTimestampAttribute,
    sequence: SequenceAttribute,
    src_port_id: SrcPortIdAttribute,
    src_channel_id: SrcChannelIdAttribute,
    dst_port_id: DstPortIdAttribute,
    dst_channel_id: DstChannelIdAttribute,
    channel_ordering: ChannelOrderingAttribute,
}

impl TimeoutPacket {
    pub fn new(packet: Packet, channel_ordering: Order) -> Self {
        Self {
            timeout_height: packet.timeout_height.into(),
            timeout_timestamp: packet.timeout_timestamp.into(),
            sequence: packet.sequence.into(),
            src_port_id: packet.source_port.into(),
            src_channel_id: packet.source_channel.into(),
            dst_port_id: packet.destination_port.into(),
            dst_channel_id: packet.destination_channel.into(),
            channel_ordering: channel_ordering.into(),
        }
    }
}

impl TryFrom<TimeoutPacket> for AbciEvent {
    type Error = Error;

    fn try_from(v: TimeoutPacket) -> Result<Self, Self::Error> {
        let mut attributes = Vec::with_capacity(11);
        attributes.push(v.timeout_height.into());
        attributes.push(v.timeout_timestamp.into());
        attributes.push(v.sequence.into());
        attributes.push(v.src_port_id.into());
        attributes.push(v.src_channel_id.into());
        attributes.push(v.dst_port_id.into());
        attributes.push(v.dst_channel_id.into());
        attributes.push(v.channel_ordering.into());

        Ok(AbciEvent {
            type_str: IbcEventType::Timeout.as_str().to_string(),
            attributes,
        })
    }
}
