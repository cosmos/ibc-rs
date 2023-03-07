//! Types for the IBC events emitted from Tendermint Websocket by the channels module.

mod channel_attributes;
mod packet_attributes;

use tendermint::abci;

use crate::core::ics04_channel::error::ChannelError;
use crate::core::ics04_channel::packet::Packet;
use crate::core::ics24_host::identifier::{ChannelId, ConnectionId, PortId};
use crate::events::IbcEventType;
use crate::prelude::*;
use crate::timestamp::Timestamp;

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
use super::packet::Sequence;
use super::timeout::TimeoutHeight;
use super::Version;

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
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OpenInit {
    port_id: PortIdAttribute,
    channel_id: ChannelIdAttribute,
    counterparty_port_id: CounterpartyPortIdAttribute,
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
        &self.counterparty_port_id.counterparty_port_id
    }
    pub fn connection_id(&self) -> &ConnectionId {
        &self.connection_id.connection_id
    }
    pub fn version(&self) -> &Version {
        &self.version.version
    }
}

impl From<OpenInit> for abci::Event {
    fn from(o: OpenInit) -> Self {
        abci::Event {
            kind: IbcEventType::OpenInitChannel.as_str().to_owned(),
            attributes: vec![
                o.port_id.into(),
                o.channel_id.into(),
                o.counterparty_port_id.into(),
                (COUNTERPARTY_CHANNEL_ID_ATTRIBUTE_KEY, "").into(),
                o.connection_id.into(),
                o.version.into(),
            ],
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
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OpenTry {
    port_id: PortIdAttribute,
    channel_id: ChannelIdAttribute,
    counterparty_port_id: CounterpartyPortIdAttribute,
    counterparty_channel_id: CounterpartyChannelIdAttribute,
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
        &self.counterparty_port_id.counterparty_port_id
    }
    pub fn counterparty_channel_id(&self) -> &ChannelId {
        &self.counterparty_channel_id.counterparty_channel_id
    }
    pub fn connection_id(&self) -> &ConnectionId {
        &self.connection_id.connection_id
    }
    pub fn version(&self) -> &Version {
        &self.version.version
    }
}

impl From<OpenTry> for abci::Event {
    fn from(o: OpenTry) -> Self {
        abci::Event {
            kind: IbcEventType::OpenTryChannel.as_str().to_owned(),
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
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OpenAck {
    port_id: PortIdAttribute,
    channel_id: ChannelIdAttribute,
    counterparty_port_id: CounterpartyPortIdAttribute,
    counterparty_channel_id: CounterpartyChannelIdAttribute,
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
        &self.counterparty_port_id.counterparty_port_id
    }
    pub fn counterparty_channel_id(&self) -> &ChannelId {
        &self.counterparty_channel_id.counterparty_channel_id
    }
    pub fn connection_id(&self) -> &ConnectionId {
        &self.connection_id.connection_id
    }
}

impl From<OpenAck> for abci::Event {
    fn from(o: OpenAck) -> Self {
        abci::Event {
            kind: IbcEventType::OpenAckChannel.as_str().to_owned(),
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
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OpenConfirm {
    port_id: PortIdAttribute,
    channel_id: ChannelIdAttribute,
    counterparty_port_id: CounterpartyPortIdAttribute,
    counterparty_channel_id: CounterpartyChannelIdAttribute,
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
        &self.counterparty_port_id.counterparty_port_id
    }
    pub fn counterparty_channel_id(&self) -> &ChannelId {
        &self.counterparty_channel_id.counterparty_channel_id
    }
    pub fn connection_id(&self) -> &ConnectionId {
        &self.connection_id.connection_id
    }
}

impl From<OpenConfirm> for abci::Event {
    fn from(o: OpenConfirm) -> Self {
        abci::Event {
            kind: IbcEventType::OpenConfirmChannel.as_str().to_owned(),
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
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CloseInit {
    port_id: PortIdAttribute,
    channel_id: ChannelIdAttribute,
    counterparty_port_id: CounterpartyPortIdAttribute,
    counterparty_channel_id: CounterpartyChannelIdAttribute,
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
        &self.counterparty_port_id.counterparty_port_id
    }
    pub fn counterparty_channel_id(&self) -> &ChannelId {
        &self.counterparty_channel_id.counterparty_channel_id
    }
    pub fn connection_id(&self) -> &ConnectionId {
        &self.connection_id.connection_id
    }
}

impl From<CloseInit> for abci::Event {
    fn from(o: CloseInit) -> Self {
        abci::Event {
            kind: IbcEventType::CloseInitChannel.as_str().to_owned(),
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
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CloseConfirm {
    port_id: PortIdAttribute,
    channel_id: ChannelIdAttribute,
    counterparty_port_id: CounterpartyPortIdAttribute,
    counterparty_channel_id: CounterpartyChannelIdAttribute,
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
        &self.counterparty_port_id.counterparty_port_id
    }
    pub fn counterparty_channel_id(&self) -> &ChannelId {
        &self.counterparty_channel_id.counterparty_channel_id
    }
    pub fn connection_id(&self) -> &ConnectionId {
        &self.connection_id.connection_id
    }
}

impl From<CloseConfirm> for abci::Event {
    fn from(o: CloseConfirm) -> Self {
        abci::Event {
            kind: IbcEventType::CloseConfirmChannel.as_str().to_owned(),
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
#[derive(Clone, Debug, PartialEq, Eq)]
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

impl From<ChannelClosed> for abci::Event {
    fn from(ev: ChannelClosed) -> Self {
        abci::Event {
            kind: IbcEventType::ChannelClosed.as_str().to_owned(),
            attributes: vec![
                ev.port_id.into(),
                ev.channel_id.into(),
                ev.counterparty_port_id.into(),
                ev.maybe_counterparty_channel_id.map_or_else(
                    || (COUNTERPARTY_CHANNEL_ID_ATTRIBUTE_KEY, "").into(),
                    |c| c.into(),
                ),
                ev.connection_id.into(),
                ev.channel_ordering.into(),
            ],
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
#[derive(Clone, Debug, PartialEq, Eq)]
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
            timeout_height: packet.timeout_height_on_b.into(),
            timeout_timestamp: packet.timeout_timestamp_on_b.into(),
            sequence: packet.seq_on_a.into(),
            src_port_id: packet.port_id_on_a.into(),
            src_channel_id: packet.chan_id_on_a.into(),
            dst_port_id: packet.port_id_on_b.into(),
            dst_channel_id: packet.chan_id_on_b.into(),
            channel_ordering: channel_ordering.into(),
            src_connection_id: src_connection_id.into(),
        }
    }

    pub fn packet_data(&self) -> &[u8] {
        &self.packet_data.packet_data
    }

    pub fn timeout_height(&self) -> &TimeoutHeight {
        &self.timeout_height.timeout_height
    }

    pub fn timeout_timestamp(&self) -> &Timestamp {
        &self.timeout_timestamp.timeout_timestamp
    }

    pub fn sequence(&self) -> &Sequence {
        &self.sequence.sequence
    }

    pub fn src_port_id(&self) -> &PortId {
        &self.src_port_id.src_port_id
    }

    pub fn src_channel_id(&self) -> &ChannelId {
        &self.src_channel_id.src_channel_id
    }

    pub fn dst_port_id(&self) -> &PortId {
        &self.dst_port_id.dst_port_id
    }

    pub fn dst_channel_id(&self) -> &ChannelId {
        &self.dst_channel_id.dst_channel_id
    }

    pub fn channel_ordering(&self) -> &Order {
        &self.channel_ordering.order
    }

    pub fn src_connection_id(&self) -> &ConnectionId {
        &self.src_connection_id.connection_id
    }
}

impl TryFrom<SendPacket> for abci::Event {
    type Error = ChannelError;

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

        Ok(abci::Event {
            kind: IbcEventType::SendPacket.as_str().to_owned(),
            attributes,
        })
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
#[derive(Clone, Debug, PartialEq, Eq)]
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
            timeout_height: packet.timeout_height_on_b.into(),
            timeout_timestamp: packet.timeout_timestamp_on_b.into(),
            sequence: packet.seq_on_a.into(),
            src_port_id: packet.port_id_on_a.into(),
            src_channel_id: packet.chan_id_on_a.into(),
            dst_port_id: packet.port_id_on_b.into(),
            dst_channel_id: packet.chan_id_on_b.into(),
            channel_ordering: channel_ordering.into(),
            dst_connection_id: dst_connection_id.into(),
        }
    }

    pub fn packet_data(&self) -> &[u8] {
        &self.packet_data.packet_data
    }

    pub fn timeout_height(&self) -> &TimeoutHeight {
        &self.timeout_height.timeout_height
    }

    pub fn timeout_timestamp(&self) -> &Timestamp {
        &self.timeout_timestamp.timeout_timestamp
    }

    pub fn sequence(&self) -> &Sequence {
        &self.sequence.sequence
    }

    pub fn src_port_id(&self) -> &PortId {
        &self.src_port_id.src_port_id
    }

    pub fn src_channel_id(&self) -> &ChannelId {
        &self.src_channel_id.src_channel_id
    }

    pub fn dst_port_id(&self) -> &PortId {
        &self.dst_port_id.dst_port_id
    }

    pub fn dst_channel_id(&self) -> &ChannelId {
        &self.dst_channel_id.dst_channel_id
    }

    pub fn channel_ordering(&self) -> &Order {
        &self.channel_ordering.order
    }

    pub fn dst_connection_id(&self) -> &ConnectionId {
        &self.dst_connection_id.connection_id
    }
}

impl TryFrom<ReceivePacket> for abci::Event {
    type Error = ChannelError;

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

        Ok(abci::Event {
            kind: IbcEventType::ReceivePacket.as_str().to_owned(),
            attributes,
        })
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
#[derive(Clone, Debug, PartialEq, Eq)]
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
    dst_connection_id: PacketConnectionIdAttribute,
}

impl WriteAcknowledgement {
    pub fn new(
        packet: Packet,
        acknowledgement: Acknowledgement,
        dst_connection_id: ConnectionId,
    ) -> Self {
        Self {
            packet_data: packet.data.into(),
            timeout_height: packet.timeout_height_on_b.into(),
            timeout_timestamp: packet.timeout_timestamp_on_b.into(),
            sequence: packet.seq_on_a.into(),
            src_port_id: packet.port_id_on_a.into(),
            src_channel_id: packet.chan_id_on_a.into(),
            dst_port_id: packet.port_id_on_b.into(),
            dst_channel_id: packet.chan_id_on_b.into(),
            acknowledgement: acknowledgement.into(),
            dst_connection_id: dst_connection_id.into(),
        }
    }

    pub fn packet_data(&self) -> &[u8] {
        &self.packet_data.packet_data
    }

    pub fn timeout_height(&self) -> &TimeoutHeight {
        &self.timeout_height.timeout_height
    }

    pub fn timeout_timestamp(&self) -> &Timestamp {
        &self.timeout_timestamp.timeout_timestamp
    }

    pub fn sequence(&self) -> &Sequence {
        &self.sequence.sequence
    }

    pub fn src_port_id(&self) -> &PortId {
        &self.src_port_id.src_port_id
    }

    pub fn src_channel_id(&self) -> &ChannelId {
        &self.src_channel_id.src_channel_id
    }

    pub fn dst_port_id(&self) -> &PortId {
        &self.dst_port_id.dst_port_id
    }

    pub fn dst_channel_id(&self) -> &ChannelId {
        &self.dst_channel_id.dst_channel_id
    }

    pub fn acknowledgement(&self) -> &Acknowledgement {
        &self.acknowledgement.acknowledgement
    }

    pub fn dst_connection_id(&self) -> &ConnectionId {
        &self.dst_connection_id.connection_id
    }
}

impl TryFrom<WriteAcknowledgement> for abci::Event {
    type Error = ChannelError;

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
        attributes.push(v.dst_connection_id.into());

        Ok(abci::Event {
            kind: IbcEventType::WriteAck.as_str().to_owned(),
            attributes,
        })
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
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AcknowledgePacket {
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
            timeout_height: packet.timeout_height_on_b.into(),
            timeout_timestamp: packet.timeout_timestamp_on_b.into(),
            sequence: packet.seq_on_a.into(),
            src_port_id: packet.port_id_on_a.into(),
            src_channel_id: packet.chan_id_on_a.into(),
            dst_port_id: packet.port_id_on_b.into(),
            dst_channel_id: packet.chan_id_on_b.into(),
            channel_ordering: channel_ordering.into(),
            src_connection_id: src_connection_id.into(),
        }
    }

    pub fn timeout_height(&self) -> &TimeoutHeight {
        &self.timeout_height.timeout_height
    }

    pub fn timeout_timestamp(&self) -> &Timestamp {
        &self.timeout_timestamp.timeout_timestamp
    }

    pub fn sequence(&self) -> &Sequence {
        &self.sequence.sequence
    }

    pub fn src_port_id(&self) -> &PortId {
        &self.src_port_id.src_port_id
    }

    pub fn src_channel_id(&self) -> &ChannelId {
        &self.src_channel_id.src_channel_id
    }

    pub fn dst_port_id(&self) -> &PortId {
        &self.dst_port_id.dst_port_id
    }

    pub fn dst_channel_id(&self) -> &ChannelId {
        &self.dst_channel_id.dst_channel_id
    }

    pub fn channel_ordering(&self) -> &Order {
        &self.channel_ordering.order
    }

    pub fn src_connection_id(&self) -> &ConnectionId {
        &self.src_connection_id.connection_id
    }
}

impl TryFrom<AcknowledgePacket> for abci::Event {
    type Error = ChannelError;

    fn try_from(v: AcknowledgePacket) -> Result<Self, Self::Error> {
        Ok(abci::Event {
            kind: IbcEventType::AckPacket.as_str().to_owned(),
            attributes: vec![
                v.timeout_height.into(),
                v.timeout_timestamp.into(),
                v.sequence.into(),
                v.src_port_id.into(),
                v.src_channel_id.into(),
                v.dst_port_id.into(),
                v.dst_channel_id.into(),
                v.channel_ordering.into(),
                v.src_connection_id.into(),
            ],
        })
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
#[derive(Clone, Debug, PartialEq, Eq)]
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
            timeout_height: packet.timeout_height_on_b.into(),
            timeout_timestamp: packet.timeout_timestamp_on_b.into(),
            sequence: packet.seq_on_a.into(),
            src_port_id: packet.port_id_on_a.into(),
            src_channel_id: packet.chan_id_on_a.into(),
            dst_port_id: packet.port_id_on_b.into(),
            dst_channel_id: packet.chan_id_on_b.into(),
            channel_ordering: channel_ordering.into(),
        }
    }

    pub fn timeout_height(&self) -> &TimeoutHeight {
        &self.timeout_height.timeout_height
    }

    pub fn timeout_timestamp(&self) -> &Timestamp {
        &self.timeout_timestamp.timeout_timestamp
    }

    pub fn sequence(&self) -> &Sequence {
        &self.sequence.sequence
    }

    pub fn src_port_id(&self) -> &PortId {
        &self.src_port_id.src_port_id
    }

    pub fn src_channel_id(&self) -> &ChannelId {
        &self.src_channel_id.src_channel_id
    }

    pub fn dst_port_id(&self) -> &PortId {
        &self.dst_port_id.dst_port_id
    }

    pub fn dst_channel_id(&self) -> &ChannelId {
        &self.dst_channel_id.dst_channel_id
    }

    pub fn channel_ordering(&self) -> &Order {
        &self.channel_ordering.order
    }
}

impl TryFrom<TimeoutPacket> for abci::Event {
    type Error = ChannelError;

    fn try_from(v: TimeoutPacket) -> Result<Self, Self::Error> {
        Ok(abci::Event {
            kind: IbcEventType::Timeout.as_str().to_owned(),
            attributes: vec![
                v.timeout_height.into(),
                v.timeout_timestamp.into(),
                v.sequence.into(),
                v.src_port_id.into(),
                v.src_channel_id.into(),
                v.dst_port_id.into(),
                v.dst_channel_id.into(),
                v.channel_ordering.into(),
            ],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tendermint::abci::Event as AbciEvent;

    #[test]
    fn ibc_to_abci_channel_events() {
        struct Test {
            kind: IbcEventType,
            event: AbciEvent,
            expected_keys: Vec<&'static str>,
            expected_values: Vec<&'static str>,
        }

        let port_id = PortId::transfer();
        let channel_id = ChannelId::new(0);
        let connection_id = ConnectionId::new(0);
        let counterparty_port_id = PortId::transfer();
        let counterparty_channel_id = ChannelId::new(1);
        let version = Version::new("ics20-1".to_string());
        let expected_keys = vec![
            "port_id",
            "channel_id",
            "counterparty_port_id",
            "counterparty_channel_id",
            "connection_id",
            "version",
        ];
        let expected_values = vec![
            "transfer",
            "channel-0",
            "transfer",
            "channel-1",
            "connection-0",
            "ics20-1",
        ];

        let tests: Vec<Test> = vec![
            Test {
                kind: IbcEventType::OpenInitChannel,
                event: OpenInit::new(
                    port_id.clone(),
                    channel_id.clone(),
                    counterparty_port_id.clone(),
                    connection_id.clone(),
                    version.clone(),
                )
                .into(),
                expected_keys: expected_keys.clone(),
                expected_values: expected_values
                    .iter()
                    .enumerate()
                    .map(|(i, v)| if i == 3 { "" } else { v })
                    .collect(),
            },
            Test {
                kind: IbcEventType::OpenTryChannel,
                event: OpenTry::new(
                    port_id.clone(),
                    channel_id.clone(),
                    counterparty_port_id.clone(),
                    counterparty_channel_id.clone(),
                    connection_id.clone(),
                    version,
                )
                .into(),
                expected_keys: expected_keys.clone(),
                expected_values: expected_values.clone(),
            },
            Test {
                kind: IbcEventType::OpenAckChannel,
                event: OpenAck::new(
                    port_id.clone(),
                    channel_id.clone(),
                    counterparty_port_id.clone(),
                    counterparty_channel_id.clone(),
                    connection_id.clone(),
                )
                .into(),
                expected_keys: expected_keys[0..5].to_vec(),
                expected_values: expected_values[0..5].to_vec(),
            },
            Test {
                kind: IbcEventType::OpenConfirmChannel,
                event: OpenConfirm::new(
                    port_id.clone(),
                    channel_id.clone(),
                    counterparty_port_id.clone(),
                    counterparty_channel_id.clone(),
                    connection_id.clone(),
                )
                .into(),
                expected_keys: expected_keys[0..5].to_vec(),
                expected_values: expected_values[0..5].to_vec(),
            },
            Test {
                kind: IbcEventType::CloseInitChannel,
                event: CloseInit::new(
                    port_id.clone(),
                    channel_id.clone(),
                    counterparty_port_id.clone(),
                    counterparty_channel_id.clone(),
                    connection_id.clone(),
                )
                .into(),
                expected_keys: expected_keys[0..5].to_vec(),
                expected_values: expected_values[0..5].to_vec(),
            },
            Test {
                kind: IbcEventType::CloseConfirmChannel,
                event: CloseConfirm::new(
                    port_id,
                    channel_id,
                    counterparty_port_id,
                    counterparty_channel_id,
                    connection_id,
                )
                .into(),
                expected_keys: expected_keys[0..5].to_vec(),
                expected_values: expected_values[0..5].to_vec(),
            },
        ];

        for t in tests {
            assert_eq!(t.kind.as_str(), t.event.kind);
            assert_eq!(t.expected_keys.len(), t.event.attributes.len());
            for (i, e) in t.event.attributes.iter().enumerate() {
                assert_eq!(
                    e.key,
                    t.expected_keys[i],
                    "key mismatch for {:?}",
                    t.kind.as_str()
                );
            }
            assert_eq!(t.expected_values.len(), t.event.attributes.len());
            for (i, e) in t.event.attributes.iter().enumerate() {
                assert_eq!(
                    e.value,
                    t.expected_values[i],
                    "value mismatch for {:?}",
                    t.kind.as_str()
                );
            }
        }
    }
}
