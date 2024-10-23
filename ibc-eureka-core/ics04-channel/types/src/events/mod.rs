//! Types for the IBC events emitted from Tendermint Websocket by the channels module.

mod channel_attributes;
mod packet_attributes;

use ibc_eureka_core_host_types::error::DecodingError;
use ibc_eureka_core_host_types::identifiers::{ChannelId, PortId, Sequence};
use ibc_primitives::prelude::*;
use tendermint::abci;

use self::channel_attributes::{
    ChannelIdAttribute, CounterpartyChannelIdAttribute, CounterpartyPortIdAttribute,
    PortIdAttribute, VersionAttribute, COUNTERPARTY_CHANNEL_ID_ATTRIBUTE_KEY,
};
use self::packet_attributes::{
    AcknowledgementAttribute, ChannelOrderingAttribute, DstChannelIdAttribute, DstPortIdAttribute,
    PacketDataAttribute, SequenceAttribute, SrcChannelIdAttribute, SrcPortIdAttribute,
    TimeoutHeightAttribute, TimeoutTimestampAttribute,
};
use super::acknowledgement::Acknowledgement;
use super::channel::Order;
use super::timeout::TimeoutHeight;
use super::Version;
use crate::packet::Packet;
use crate::timeout::TimeoutTimestamp;

/// Channel event types corresponding to ibc-go's channel events:
/// https://github.com/cosmos/ibc-go/blob/c4413c5877f9ef883494da1721cb18caaba7f7f5/modules/core/04-channel/types/events.go#L52-L72
const CHANNEL_OPEN_INIT_EVENT: &str = "channel_open_init";
const CHANNEL_OPEN_TRY_EVENT: &str = "channel_open_try";
const CHANNEL_OPEN_ACK_EVENT: &str = "channel_open_ack";
const CHANNEL_OPEN_CONFIRM_EVENT: &str = "channel_open_confirm";
const CHANNEL_CLOSE_INIT_EVENT: &str = "channel_close_init";
const CHANNEL_CLOSE_CONFIRM_EVENT: &str = "channel_close_confirm";
const CHANNEL_CLOSED_EVENT: &str = "channel_close";

/// Packet event types
const SEND_PACKET_EVENT: &str = "send_packet";
const RECEIVE_PACKET_EVENT: &str = "recv_packet";
const WRITE_ACK_EVENT: &str = "write_acknowledgement";
const ACK_PACKET_EVENT: &str = "acknowledge_packet";
const TIMEOUT_EVENT: &str = "timeout_packet";

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
    port_id_attr_on_a: PortIdAttribute,
    chan_id_attr_on_a: ChannelIdAttribute,
    port_id_attr_on_b: CounterpartyPortIdAttribute,
    version_attr_on_a: VersionAttribute,
}

impl OpenInit {
    pub fn new(
        port_id_on_a: PortId,
        chan_id_on_a: ChannelId,
        port_id_on_b: PortId,
        version_on_a: Version,
    ) -> Self {
        Self {
            port_id_attr_on_a: port_id_on_a.into(),
            chan_id_attr_on_a: chan_id_on_a.into(),
            port_id_attr_on_b: port_id_on_b.into(),
            version_attr_on_a: version_on_a.into(),
        }
    }
    pub fn port_id_on_a(&self) -> &PortId {
        &self.port_id_attr_on_a.port_id
    }
    pub fn chan_id_on_a(&self) -> &ChannelId {
        &self.chan_id_attr_on_a.channel_id
    }
    pub fn port_id_on_b(&self) -> &PortId {
        &self.port_id_attr_on_b.counterparty_port_id
    }
    pub fn version_on_a(&self) -> &Version {
        &self.version_attr_on_a.version
    }

    pub fn event_type(&self) -> &str {
        CHANNEL_OPEN_INIT_EVENT
    }
}

impl From<OpenInit> for abci::Event {
    fn from(o: OpenInit) -> Self {
        abci::Event {
            kind: CHANNEL_OPEN_INIT_EVENT.to_string(),
            attributes: vec![
                o.port_id_attr_on_a.into(),
                o.chan_id_attr_on_a.into(),
                o.port_id_attr_on_b.into(),
                (COUNTERPARTY_CHANNEL_ID_ATTRIBUTE_KEY, "").into(),
                o.version_attr_on_a.into(),
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
    port_id_attr_on_b: PortIdAttribute,
    chan_id_attr_on_b: ChannelIdAttribute,
    port_id_attr_on_a: CounterpartyPortIdAttribute,
    chan_id_attr_on_a: CounterpartyChannelIdAttribute,
    version_attr_on_b: VersionAttribute,
}

impl OpenTry {
    pub fn new(
        port_id_on_b: PortId,
        chan_id_on_b: ChannelId,
        port_id_on_a: PortId,
        chan_id_on_a: ChannelId,
        version_on_b: Version,
    ) -> Self {
        Self {
            port_id_attr_on_b: port_id_on_b.into(),
            chan_id_attr_on_b: chan_id_on_b.into(),
            port_id_attr_on_a: port_id_on_a.into(),
            chan_id_attr_on_a: chan_id_on_a.into(),
            version_attr_on_b: version_on_b.into(),
        }
    }
    pub fn port_id_on_b(&self) -> &PortId {
        &self.port_id_attr_on_b.port_id
    }
    pub fn chan_id_on_b(&self) -> &ChannelId {
        &self.chan_id_attr_on_b.channel_id
    }
    pub fn port_id_on_a(&self) -> &PortId {
        &self.port_id_attr_on_a.counterparty_port_id
    }
    pub fn chan_id_on_a(&self) -> &ChannelId {
        &self.chan_id_attr_on_a.counterparty_channel_id
    }
    pub fn version_on_b(&self) -> &Version {
        &self.version_attr_on_b.version
    }

    pub fn event_type(&self) -> &str {
        CHANNEL_OPEN_TRY_EVENT
    }
}

impl From<OpenTry> for abci::Event {
    fn from(o: OpenTry) -> Self {
        abci::Event {
            kind: CHANNEL_OPEN_TRY_EVENT.to_string(),
            attributes: vec![
                o.port_id_attr_on_b.into(),
                o.chan_id_attr_on_b.into(),
                o.port_id_attr_on_a.into(),
                o.chan_id_attr_on_a.into(),
                o.version_attr_on_b.into(),
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
    port_id_attr_on_a: PortIdAttribute,
    chan_id_attr_on_a: ChannelIdAttribute,
    port_id_attr_on_b: CounterpartyPortIdAttribute,
    chan_id_attr_on_b: CounterpartyChannelIdAttribute,
}

impl OpenAck {
    pub fn new(
        port_id_on_a: PortId,
        chan_id_on_a: ChannelId,
        port_id_on_b: PortId,
        chan_id_on_b: ChannelId,
    ) -> Self {
        Self {
            port_id_attr_on_a: port_id_on_a.into(),
            chan_id_attr_on_a: chan_id_on_a.into(),
            port_id_attr_on_b: port_id_on_b.into(),
            chan_id_attr_on_b: chan_id_on_b.into(),
        }
    }
    pub fn port_id_on_a(&self) -> &PortId {
        &self.port_id_attr_on_a.port_id
    }
    pub fn chan_id_on_a(&self) -> &ChannelId {
        &self.chan_id_attr_on_a.channel_id
    }
    pub fn port_id_on_b(&self) -> &PortId {
        &self.port_id_attr_on_b.counterparty_port_id
    }
    pub fn chan_id_on_b(&self) -> &ChannelId {
        &self.chan_id_attr_on_b.counterparty_channel_id
    }

    pub fn event_type(&self) -> &str {
        CHANNEL_OPEN_ACK_EVENT
    }
}

impl From<OpenAck> for abci::Event {
    fn from(o: OpenAck) -> Self {
        abci::Event {
            kind: CHANNEL_OPEN_ACK_EVENT.to_string(),
            attributes: vec![
                o.port_id_attr_on_a.into(),
                o.chan_id_attr_on_a.into(),
                o.port_id_attr_on_b.into(),
                o.chan_id_attr_on_b.into(),
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
    port_id_attr_on_b: PortIdAttribute,
    chan_id_attr_on_b: ChannelIdAttribute,
    port_id_attr_on_a: CounterpartyPortIdAttribute,
    chan_id_attr_on_a: CounterpartyChannelIdAttribute,
}

impl OpenConfirm {
    pub fn new(
        port_id_on_b: PortId,
        chan_id_on_b: ChannelId,
        port_id_on_a: PortId,
        chan_id_on_a: ChannelId,
    ) -> Self {
        Self {
            port_id_attr_on_b: port_id_on_b.into(),
            chan_id_attr_on_b: chan_id_on_b.into(),
            port_id_attr_on_a: port_id_on_a.into(),
            chan_id_attr_on_a: chan_id_on_a.into(),
        }
    }
    pub fn port_id_on_b(&self) -> &PortId {
        &self.port_id_attr_on_b.port_id
    }
    pub fn chan_id_on_b(&self) -> &ChannelId {
        &self.chan_id_attr_on_b.channel_id
    }
    pub fn port_id_on_a(&self) -> &PortId {
        &self.port_id_attr_on_a.counterparty_port_id
    }
    pub fn chan_id_on_a(&self) -> &ChannelId {
        &self.chan_id_attr_on_a.counterparty_channel_id
    }

    pub fn event_type(&self) -> &str {
        CHANNEL_OPEN_CONFIRM_EVENT
    }
}

impl From<OpenConfirm> for abci::Event {
    fn from(o: OpenConfirm) -> Self {
        abci::Event {
            kind: CHANNEL_OPEN_CONFIRM_EVENT.to_string(),
            attributes: vec![
                o.port_id_attr_on_b.into(),
                o.chan_id_attr_on_b.into(),
                o.port_id_attr_on_a.into(),
                o.chan_id_attr_on_a.into(),
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
    port_id_attr_on_a: PortIdAttribute,
    chan_id_attr_on_a: ChannelIdAttribute,
    port_id_attr_on_b: CounterpartyPortIdAttribute,
    chan_id_attr_on_b: CounterpartyChannelIdAttribute,
}

impl CloseInit {
    pub fn new(
        port_id_on_a: PortId,
        chan_id_on_a: ChannelId,
        port_id_on_b: PortId,
        chan_id_on_b: ChannelId,
    ) -> Self {
        Self {
            port_id_attr_on_a: port_id_on_a.into(),
            chan_id_attr_on_a: chan_id_on_a.into(),
            port_id_attr_on_b: port_id_on_b.into(),
            chan_id_attr_on_b: chan_id_on_b.into(),
        }
    }
    pub fn port_id_on_a(&self) -> &PortId {
        &self.port_id_attr_on_a.port_id
    }
    pub fn chan_id_on_a(&self) -> &ChannelId {
        &self.chan_id_attr_on_a.channel_id
    }
    pub fn port_id_on_b(&self) -> &PortId {
        &self.port_id_attr_on_b.counterparty_port_id
    }
    pub fn chan_id_on_b(&self) -> &ChannelId {
        &self.chan_id_attr_on_b.counterparty_channel_id
    }
    pub fn event_type(&self) -> &str {
        CHANNEL_CLOSE_INIT_EVENT
    }
}

impl From<CloseInit> for abci::Event {
    fn from(o: CloseInit) -> Self {
        abci::Event {
            kind: CHANNEL_CLOSE_INIT_EVENT.to_string(),
            attributes: vec![
                o.port_id_attr_on_a.into(),
                o.chan_id_attr_on_a.into(),
                o.port_id_attr_on_b.into(),
                o.chan_id_attr_on_b.into(),
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
    port_id_attr_on_b: PortIdAttribute,
    chan_id_attr_on_b: ChannelIdAttribute,
    port_id_attr_on_a: CounterpartyPortIdAttribute,
    chan_id_attr_on_a: CounterpartyChannelIdAttribute,
}

impl CloseConfirm {
    pub fn new(
        port_id_on_b: PortId,
        chan_id_on_b: ChannelId,
        port_id_on_a: PortId,
        chan_id_on_a: ChannelId,
    ) -> Self {
        Self {
            port_id_attr_on_b: port_id_on_b.into(),
            chan_id_attr_on_b: chan_id_on_b.into(),
            port_id_attr_on_a: port_id_on_a.into(),
            chan_id_attr_on_a: chan_id_on_a.into(),
        }
    }
    pub fn port_id_on_b(&self) -> &PortId {
        &self.port_id_attr_on_b.port_id
    }
    pub fn chan_id_on_b(&self) -> &ChannelId {
        &self.chan_id_attr_on_b.channel_id
    }
    pub fn port_id_on_a(&self) -> &PortId {
        &self.port_id_attr_on_a.counterparty_port_id
    }
    pub fn chan_id_on_a(&self) -> &ChannelId {
        &self.chan_id_attr_on_a.counterparty_channel_id
    }
    pub fn event_type(&self) -> &str {
        CHANNEL_CLOSE_CONFIRM_EVENT
    }
}

impl From<CloseConfirm> for abci::Event {
    fn from(o: CloseConfirm) -> Self {
        abci::Event {
            kind: CHANNEL_CLOSE_CONFIRM_EVENT.to_string(),
            attributes: vec![
                o.port_id_attr_on_b.into(),
                o.chan_id_attr_on_b.into(),
                o.port_id_attr_on_a.into(),
                o.chan_id_attr_on_a.into(),
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
    port_id_attr_on_a: PortIdAttribute,
    chan_id_attr_on_a: ChannelIdAttribute,
    port_id_attr_on_b: CounterpartyPortIdAttribute,
    maybe_chan_id_attr_on_b: Option<CounterpartyChannelIdAttribute>,
    channel_ordering_attr: ChannelOrderingAttribute,
}

impl ChannelClosed {
    pub fn new(
        port_id_on_a: PortId,
        chan_id_on_a: ChannelId,
        port_id_on_b: PortId,
        maybe_chan_id_on_b: Option<ChannelId>,
        channel_ordering: Order,
    ) -> Self {
        Self {
            port_id_attr_on_a: port_id_on_a.into(),
            chan_id_attr_on_a: chan_id_on_a.into(),
            port_id_attr_on_b: port_id_on_b.into(),
            maybe_chan_id_attr_on_b: maybe_chan_id_on_b.map(Into::into),
            channel_ordering_attr: channel_ordering.into(),
        }
    }
    pub fn port_id_on_b(&self) -> &PortId {
        &self.port_id_attr_on_a.port_id
    }
    pub fn chan_id_on_b(&self) -> &ChannelId {
        &self.chan_id_attr_on_a.channel_id
    }
    pub fn port_id_on_a(&self) -> &PortId {
        &self.port_id_attr_on_b.counterparty_port_id
    }
    pub fn chan_id_on_a(&self) -> Option<&ChannelId> {
        self.maybe_chan_id_attr_on_b.as_ref().map(AsRef::as_ref)
    }
    pub fn channel_ordering(&self) -> &Order {
        &self.channel_ordering_attr.order
    }

    pub fn event_type(&self) -> &str {
        CHANNEL_CLOSED_EVENT
    }
}

impl From<ChannelClosed> for abci::Event {
    fn from(ev: ChannelClosed) -> Self {
        abci::Event {
            kind: CHANNEL_CLOSED_EVENT.to_string(),
            attributes: vec![
                ev.port_id_attr_on_a.into(),
                ev.chan_id_attr_on_a.into(),
                ev.port_id_attr_on_b.into(),
                ev.maybe_chan_id_attr_on_b.map_or_else(
                    || (COUNTERPARTY_CHANNEL_ID_ATTRIBUTE_KEY, "").into(),
                    Into::into,
                ),
                ev.channel_ordering_attr.into(),
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
    packet_data_attr: PacketDataAttribute,
    timeout_height_attr_on_b: TimeoutHeightAttribute,
    timeout_timestamp_attr_on_b: TimeoutTimestampAttribute,
    seq_attr_on_a: SequenceAttribute,
    port_id_attr_on_a: SrcPortIdAttribute,
    chan_id_attr_on_a: SrcChannelIdAttribute,
    port_id_attr_on_b: DstPortIdAttribute,
    chan_id_attr_on_b: DstChannelIdAttribute,
    channel_ordering_attr: ChannelOrderingAttribute,
}

impl SendPacket {
    pub fn new(packet: Packet, channel_ordering: Order) -> Self {
        let payload = packet.payloads[0].clone();
        Self {
            packet_data_attr: payload.data.into(),
            timeout_height_attr_on_b: packet.header.timeout_height_on_b.into(),
            timeout_timestamp_attr_on_b: packet.header.timeout_timestamp_on_b.into(),
            seq_attr_on_a: packet.header.seq_on_a.into(),
            port_id_attr_on_a: payload.header.source_port.1.into(),
            chan_id_attr_on_a: packet.header.source_client.into(),
            port_id_attr_on_b: payload.header.target_port.1.into(),
            chan_id_attr_on_b: packet.header.target_client.into(),
            channel_ordering_attr: channel_ordering.into(),
        }
    }

    pub fn packet_data(&self) -> &[u8] {
        &self.packet_data_attr.packet_data
    }

    pub fn timeout_height_on_b(&self) -> &TimeoutHeight {
        &self.timeout_height_attr_on_b.timeout_height
    }

    pub fn timeout_timestamp_on_b(&self) -> &TimeoutTimestamp {
        &self.timeout_timestamp_attr_on_b.timeout_timestamp
    }

    pub fn seq_on_a(&self) -> &Sequence {
        &self.seq_attr_on_a.sequence
    }

    pub fn port_id_on_a(&self) -> &PortId {
        &self.port_id_attr_on_a.src_port_id
    }

    pub fn chan_id_on_a(&self) -> &ChannelId {
        &self.chan_id_attr_on_a.src_channel_id
    }

    pub fn port_id_on_b(&self) -> &PortId {
        &self.port_id_attr_on_b.dst_port_id
    }

    pub fn chan_id_on_b(&self) -> &ChannelId {
        &self.chan_id_attr_on_b.dst_channel_id
    }

    pub fn channel_ordering(&self) -> &Order {
        &self.channel_ordering_attr.order
    }

    pub fn event_type(&self) -> &str {
        SEND_PACKET_EVENT
    }
}

impl TryFrom<SendPacket> for abci::Event {
    type Error = DecodingError;

    fn try_from(v: SendPacket) -> Result<Self, Self::Error> {
        let mut attributes = Vec::with_capacity(11);
        attributes.append(&mut v.packet_data_attr.try_into()?);
        attributes.push(v.timeout_height_attr_on_b.into());
        attributes.push(v.timeout_timestamp_attr_on_b.into());
        attributes.push(v.seq_attr_on_a.into());
        attributes.push(v.port_id_attr_on_a.into());
        attributes.push(v.chan_id_attr_on_a.into());
        attributes.push(v.port_id_attr_on_b.into());
        attributes.push(v.chan_id_attr_on_b.into());
        attributes.push(v.channel_ordering_attr.into());

        Ok(abci::Event {
            kind: SEND_PACKET_EVENT.to_string(),
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
    packet_data_attr: PacketDataAttribute,
    timeout_height_attr_on_b: TimeoutHeightAttribute,
    timeout_timestamp_attr_on_b: TimeoutTimestampAttribute,
    seq_attr_on_a: SequenceAttribute,
    port_id_attr_on_a: SrcPortIdAttribute,
    chan_id_attr_on_a: SrcChannelIdAttribute,
    port_id_attr_on_b: DstPortIdAttribute,
    chan_id_attr_on_b: DstChannelIdAttribute,
    channel_ordering_attr: ChannelOrderingAttribute,
}

impl ReceivePacket {
    pub fn new(packet: Packet, channel_ordering: Order) -> Self {
        let payload = packet.payloads[0].clone();
        Self {
            packet_data_attr: payload.data.into(),
            timeout_height_attr_on_b: packet.header.timeout_height_on_b.into(),
            timeout_timestamp_attr_on_b: packet.header.timeout_timestamp_on_b.into(),
            seq_attr_on_a: packet.header.seq_on_a.into(),
            port_id_attr_on_a: payload.header.source_port.1.into(),
            chan_id_attr_on_a: packet.header.source_client.into(),
            port_id_attr_on_b: payload.header.target_port.1.into(),
            chan_id_attr_on_b: packet.header.target_client.into(),
            channel_ordering_attr: channel_ordering.into(),
        }
    }

    pub fn packet_data(&self) -> &[u8] {
        &self.packet_data_attr.packet_data
    }

    pub fn timeout_height_on_b(&self) -> &TimeoutHeight {
        &self.timeout_height_attr_on_b.timeout_height
    }

    pub fn timeout_timestamp_on_b(&self) -> &TimeoutTimestamp {
        &self.timeout_timestamp_attr_on_b.timeout_timestamp
    }

    pub fn seq_on_b(&self) -> &Sequence {
        &self.seq_attr_on_a.sequence
    }

    pub fn port_id_on_b(&self) -> &PortId {
        &self.port_id_attr_on_a.src_port_id
    }

    pub fn chan_id_on_b(&self) -> &ChannelId {
        &self.chan_id_attr_on_a.src_channel_id
    }

    pub fn port_id_on_a(&self) -> &PortId {
        &self.port_id_attr_on_b.dst_port_id
    }

    pub fn chan_id_on_a(&self) -> &ChannelId {
        &self.chan_id_attr_on_b.dst_channel_id
    }

    pub fn channel_ordering(&self) -> &Order {
        &self.channel_ordering_attr.order
    }

    pub fn event_type(&self) -> &str {
        RECEIVE_PACKET_EVENT
    }
}

impl TryFrom<ReceivePacket> for abci::Event {
    type Error = DecodingError;

    fn try_from(v: ReceivePacket) -> Result<Self, Self::Error> {
        let mut attributes = Vec::with_capacity(11);
        attributes.append(&mut v.packet_data_attr.try_into()?);
        attributes.push(v.timeout_height_attr_on_b.into());
        attributes.push(v.timeout_timestamp_attr_on_b.into());
        attributes.push(v.seq_attr_on_a.into());
        attributes.push(v.port_id_attr_on_a.into());
        attributes.push(v.chan_id_attr_on_a.into());
        attributes.push(v.port_id_attr_on_b.into());
        attributes.push(v.chan_id_attr_on_b.into());
        attributes.push(v.channel_ordering_attr.into());

        Ok(abci::Event {
            kind: RECEIVE_PACKET_EVENT.to_string(),
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
    timeout_height_attr_on_b: TimeoutHeightAttribute,
    timeout_timestamp_attr_on_b: TimeoutTimestampAttribute,
    seq_attr_on_a: SequenceAttribute,
    port_id_attr_on_a: SrcPortIdAttribute,
    chan_id_attr_on_a: SrcChannelIdAttribute,
    port_id_attr_on_b: DstPortIdAttribute,
    chan_id_attr_on_b: DstChannelIdAttribute,
    acknowledgement: AcknowledgementAttribute,
}

impl WriteAcknowledgement {
    pub fn new(packet: Packet, acknowledgement: Acknowledgement) -> Self {
        let payload = packet.payloads[0].clone();
        Self {
            packet_data: payload.data.into(),
            timeout_height_attr_on_b: packet.header.timeout_height_on_b.into(),
            timeout_timestamp_attr_on_b: packet.header.timeout_timestamp_on_b.into(),
            seq_attr_on_a: packet.header.seq_on_a.into(),
            port_id_attr_on_a: payload.header.source_port.1.into(),
            chan_id_attr_on_a: packet.header.source_client.into(),
            port_id_attr_on_b: payload.header.target_port.1.into(),
            chan_id_attr_on_b: packet.header.target_client.into(),
            acknowledgement: acknowledgement.into(),
        }
    }

    pub fn packet_data(&self) -> &[u8] {
        &self.packet_data.packet_data
    }

    pub fn timeout_height_on_b(&self) -> &TimeoutHeight {
        &self.timeout_height_attr_on_b.timeout_height
    }

    pub fn timeout_timestamp_on_b(&self) -> &TimeoutTimestamp {
        &self.timeout_timestamp_attr_on_b.timeout_timestamp
    }

    pub fn seq_on_a(&self) -> &Sequence {
        &self.seq_attr_on_a.sequence
    }

    pub fn port_id_on_a(&self) -> &PortId {
        &self.port_id_attr_on_a.src_port_id
    }

    pub fn chan_id_on_a(&self) -> &ChannelId {
        &self.chan_id_attr_on_a.src_channel_id
    }

    pub fn port_id_on_b(&self) -> &PortId {
        &self.port_id_attr_on_b.dst_port_id
    }

    pub fn chan_id_on_b(&self) -> &ChannelId {
        &self.chan_id_attr_on_b.dst_channel_id
    }

    pub fn acknowledgement(&self) -> &Acknowledgement {
        &self.acknowledgement.acknowledgement
    }

    pub fn event_type(&self) -> &str {
        WRITE_ACK_EVENT
    }
}

impl TryFrom<WriteAcknowledgement> for abci::Event {
    type Error = DecodingError;

    fn try_from(v: WriteAcknowledgement) -> Result<Self, Self::Error> {
        let mut attributes = Vec::with_capacity(11);
        attributes.append(&mut v.packet_data.try_into()?);
        attributes.push(v.timeout_height_attr_on_b.into());
        attributes.push(v.timeout_timestamp_attr_on_b.into());
        attributes.push(v.seq_attr_on_a.into());
        attributes.push(v.port_id_attr_on_a.into());
        attributes.push(v.chan_id_attr_on_a.into());
        attributes.push(v.port_id_attr_on_b.into());
        attributes.push(v.chan_id_attr_on_b.into());
        attributes.append(&mut v.acknowledgement.try_into()?);

        Ok(abci::Event {
            kind: WRITE_ACK_EVENT.to_string(),
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
    timeout_height_attr_on_b: TimeoutHeightAttribute,
    timeout_timestamp_attr_on_b: TimeoutTimestampAttribute,
    seq_on_a: SequenceAttribute,
    port_id_attr_on_a: SrcPortIdAttribute,
    chan_id_attr_on_a: SrcChannelIdAttribute,
    port_id_attr_on_b: DstPortIdAttribute,
    chan_id_attr_on_b: DstChannelIdAttribute,
    channel_ordering_attr: ChannelOrderingAttribute,
}

impl AcknowledgePacket {
    pub fn new(packet: Packet, channel_ordering: Order) -> Self {
        let payload = packet.payloads[0].clone();
        Self {
            timeout_height_attr_on_b: packet.header.timeout_height_on_b.into(),
            timeout_timestamp_attr_on_b: packet.header.timeout_timestamp_on_b.into(),
            seq_on_a: packet.header.seq_on_a.into(),
            port_id_attr_on_a: payload.header.source_port.1.into(),
            chan_id_attr_on_a: packet.header.source_client.into(),
            port_id_attr_on_b: payload.header.target_port.1.into(),
            chan_id_attr_on_b: packet.header.target_client.into(),
            channel_ordering_attr: channel_ordering.into(),
        }
    }

    pub fn timeout_height_on_b(&self) -> &TimeoutHeight {
        &self.timeout_height_attr_on_b.timeout_height
    }

    pub fn timeout_timestamp_on_b(&self) -> &TimeoutTimestamp {
        &self.timeout_timestamp_attr_on_b.timeout_timestamp
    }

    pub fn seq_on_a(&self) -> &Sequence {
        &self.seq_on_a.sequence
    }

    pub fn port_id_on_a(&self) -> &PortId {
        &self.port_id_attr_on_a.src_port_id
    }

    pub fn chan_id_on_a(&self) -> &ChannelId {
        &self.chan_id_attr_on_a.src_channel_id
    }

    pub fn port_id_on_b(&self) -> &PortId {
        &self.port_id_attr_on_b.dst_port_id
    }

    pub fn chan_id_on_b(&self) -> &ChannelId {
        &self.chan_id_attr_on_b.dst_channel_id
    }

    pub fn channel_ordering(&self) -> &Order {
        &self.channel_ordering_attr.order
    }

    pub fn event_type(&self) -> &str {
        ACK_PACKET_EVENT
    }
}

impl TryFrom<AcknowledgePacket> for abci::Event {
    type Error = DecodingError;

    fn try_from(v: AcknowledgePacket) -> Result<Self, Self::Error> {
        Ok(abci::Event {
            kind: ACK_PACKET_EVENT.to_string(),
            attributes: vec![
                v.timeout_height_attr_on_b.into(),
                v.timeout_timestamp_attr_on_b.into(),
                v.seq_on_a.into(),
                v.port_id_attr_on_a.into(),
                v.chan_id_attr_on_a.into(),
                v.port_id_attr_on_b.into(),
                v.chan_id_attr_on_b.into(),
                v.channel_ordering_attr.into(),
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
    timeout_height_attr_on_b: TimeoutHeightAttribute,
    timeout_timestamp_attr_on_b: TimeoutTimestampAttribute,
    seq_attr_on_a: SequenceAttribute,
    port_id_attr_on_a: SrcPortIdAttribute,
    chan_id_attr_on_a: SrcChannelIdAttribute,
    port_id_attr_on_b: DstPortIdAttribute,
    chan_id_attr_on_b: DstChannelIdAttribute,
    channel_ordering_attr: ChannelOrderingAttribute,
}

impl TimeoutPacket {
    pub fn new(packet: Packet, channel_ordering: Order) -> Self {
        let payload = packet.payloads[0].clone();
        Self {
            timeout_height_attr_on_b: packet.header.timeout_height_on_b.into(),
            timeout_timestamp_attr_on_b: packet.header.timeout_timestamp_on_b.into(),
            seq_attr_on_a: packet.header.seq_on_a.into(),
            port_id_attr_on_a: payload.header.source_port.1.into(),
            chan_id_attr_on_a: packet.header.source_client.into(),
            port_id_attr_on_b: payload.header.target_port.1.into(),
            chan_id_attr_on_b: packet.header.target_client.into(),
            channel_ordering_attr: channel_ordering.into(),
        }
    }

    pub fn timeout_height_on_b(&self) -> &TimeoutHeight {
        &self.timeout_height_attr_on_b.timeout_height
    }

    pub fn timeout_timestamp_on_b(&self) -> &TimeoutTimestamp {
        &self.timeout_timestamp_attr_on_b.timeout_timestamp
    }

    pub fn seq_on_a(&self) -> &Sequence {
        &self.seq_attr_on_a.sequence
    }

    pub fn port_id_on_a(&self) -> &PortId {
        &self.port_id_attr_on_a.src_port_id
    }

    pub fn chan_id_on_a(&self) -> &ChannelId {
        &self.chan_id_attr_on_a.src_channel_id
    }

    pub fn port_id_on_b(&self) -> &PortId {
        &self.port_id_attr_on_b.dst_port_id
    }

    pub fn chan_id_on_b(&self) -> &ChannelId {
        &self.chan_id_attr_on_b.dst_channel_id
    }

    pub fn channel_ordering(&self) -> &Order {
        &self.channel_ordering_attr.order
    }

    pub fn event_type(&self) -> &str {
        TIMEOUT_EVENT
    }
}

impl TryFrom<TimeoutPacket> for abci::Event {
    type Error = DecodingError;

    fn try_from(v: TimeoutPacket) -> Result<Self, Self::Error> {
        Ok(abci::Event {
            kind: TIMEOUT_EVENT.to_string(),
            attributes: vec![
                v.timeout_height_attr_on_b.into(),
                v.timeout_timestamp_attr_on_b.into(),
                v.seq_attr_on_a.into(),
                v.port_id_attr_on_a.into(),
                v.chan_id_attr_on_a.into(),
                v.port_id_attr_on_b.into(),
                v.chan_id_attr_on_b.into(),
                v.channel_ordering_attr.into(),
            ],
        })
    }
}

// #[cfg(test)]
// mod tests {
//     use tendermint::abci::Event as AbciEvent;

//     use super::*;

//     #[test]
//     fn ibc_to_abci_channel_events() {
//         struct Test {
//             kind: &'static str,
//             event: AbciEvent,
//             expected_keys: Vec<&'static str>,
//             expected_values: Vec<&'static str>,
//         }

//         let port_id = PortId::transfer();
//         let channel_id = ChannelId::zero();
//         let connection_id = ConnectionId::zero();
//         let counterparty_port_id = PortId::transfer();
//         let counterparty_channel_id = ChannelId::new(1);
//         let version = Version::new("ics20-1".to_string());
//         let expected_keys = vec![
//             "port_id",
//             "channel_id",
//             "counterparty_port_id",
//             "counterparty_channel_id",
//             "connection_id",
//             "version",
//         ];
//         let expected_values = vec![
//             "transfer",
//             "channel-0",
//             "transfer",
//             "channel-1",
//             "connection-0",
//             "ics20-1",
//         ];

//         let tests: Vec<Test> = vec![
//             Test {
//                 kind: CHANNEL_OPEN_INIT_EVENT,
//                 event: OpenInit::new(
//                     port_id.clone(),
//                     channel_id.clone(),
//                     counterparty_port_id.clone(),
//                     connection_id.clone(),
//                     version.clone(),
//                 )
//                 .into(),
//                 expected_keys: expected_keys.clone(),
//                 expected_values: expected_values
//                     .iter()
//                     .enumerate()
//                     .map(|(i, v)| if i == 3 { "" } else { v })
//                     .collect(),
//             },
//             Test {
//                 kind: CHANNEL_OPEN_TRY_EVENT,
//                 event: OpenTry::new(
//                     port_id.clone(),
//                     channel_id.clone(),
//                     counterparty_port_id.clone(),
//                     counterparty_channel_id.clone(),
//                     connection_id.clone(),
//                     version,
//                 )
//                 .into(),
//                 expected_keys: expected_keys.clone(),
//                 expected_values: expected_values.clone(),
//             },
//             Test {
//                 kind: CHANNEL_OPEN_ACK_EVENT,
//                 event: OpenAck::new(
//                     port_id.clone(),
//                     channel_id.clone(),
//                     counterparty_port_id.clone(),
//                     counterparty_channel_id.clone(),
//                     connection_id.clone(),
//                 )
//                 .into(),
//                 expected_keys: expected_keys[0..5].to_vec(),
//                 expected_values: expected_values[0..5].to_vec(),
//             },
//             Test {
//                 kind: CHANNEL_OPEN_CONFIRM_EVENT,
//                 event: OpenConfirm::new(
//                     port_id.clone(),
//                     channel_id.clone(),
//                     counterparty_port_id.clone(),
//                     counterparty_channel_id.clone(),
//                     connection_id.clone(),
//                 )
//                 .into(),
//                 expected_keys: expected_keys[0..5].to_vec(),
//                 expected_values: expected_values[0..5].to_vec(),
//             },
//             Test {
//                 kind: CHANNEL_CLOSE_INIT_EVENT,
//                 event: CloseInit::new(
//                     port_id.clone(),
//                     channel_id.clone(),
//                     counterparty_port_id.clone(),
//                     counterparty_channel_id.clone(),
//                     connection_id.clone(),
//                 )
//                 .into(),
//                 expected_keys: expected_keys[0..5].to_vec(),
//                 expected_values: expected_values[0..5].to_vec(),
//             },
//             Test {
//                 kind: CHANNEL_CLOSE_CONFIRM_EVENT,
//                 event: CloseConfirm::new(
//                     port_id,
//                     channel_id,
//                     counterparty_port_id,
//                     counterparty_channel_id,
//                     connection_id,
//                 )
//                 .into(),
//                 expected_keys: expected_keys[0..5].to_vec(),
//                 expected_values: expected_values[0..5].to_vec(),
//             },
//         ];

//         for t in tests {
//             assert_eq!(t.kind, t.event.kind);
//             assert_eq!(t.expected_keys.len(), t.event.attributes.len());
//             for (i, e) in t.event.attributes.iter().enumerate() {
//                 assert_eq!(
//                     e.key_str().unwrap(),
//                     t.expected_keys[i],
//                     "key mismatch for {:?}",
//                     t.kind
//                 );
//             }
//             assert_eq!(t.expected_values.len(), t.event.attributes.len());
//             for (i, e) in t.event.attributes.iter().enumerate() {
//                 assert_eq!(
//                     e.value_str().unwrap(),
//                     t.expected_values[i],
//                     "value mismatch for {:?}",
//                     t.kind
//                 );
//             }
//         }
//     }
// }
