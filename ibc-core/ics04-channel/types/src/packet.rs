//! Defines the packet type
use ibc_core_client_types::Height;
use ibc_core_host_types::identifiers::{ChannelId, PortId, Sequence};
use ibc_primitives::prelude::*;
use ibc_primitives::Expiry::Expired;
use ibc_primitives::Timestamp;
use ibc_proto::ibc::core::channel::v1::{Packet as RawPacket, PacketState as RawPacketState};

use super::timeout::TimeoutHeight;
use crate::error::PacketError;

/// Enumeration of proof carrying ICS4 message, helper for relayer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PacketMsgType {
    Recv,
    Ack,
    TimeoutUnordered,
    TimeoutOrdered,
    TimeoutOnClose,
}

/// Packet receipt, used over unordered channels.
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
#[derive(Clone, Debug)]
pub enum Receipt {
    Ok,
}

impl core::fmt::Display for PacketMsgType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            PacketMsgType::Recv => write!(f, "(PacketMsgType::Recv)"),
            PacketMsgType::Ack => write!(f, "(PacketMsgType::Ack)"),
            PacketMsgType::TimeoutUnordered => write!(f, "(PacketMsgType::TimeoutUnordered)"),
            PacketMsgType::TimeoutOrdered => write!(f, "(PacketMsgType::TimeoutOrdered)"),
            PacketMsgType::TimeoutOnClose => write!(f, "(PacketMsgType::TimeoutOnClose)"),
        }
    }
}

/// The packet type; this is what applications send to one another.
///
/// Each application defines the structure of the `data` field.
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
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Hash, PartialEq, Eq)]
pub struct Packet {
    pub seq_on_a: Sequence,
    pub port_id_on_a: PortId,
    pub chan_id_on_a: ChannelId,
    pub port_id_on_b: PortId,
    pub chan_id_on_b: ChannelId,
    #[cfg_attr(
        feature = "serde",
        serde(serialize_with = "ibc_core_commitment_types::serializer::ser_hex_upper")
    )]
    pub data: Vec<u8>,
    pub timeout_height_on_b: TimeoutHeight,
    pub timeout_timestamp_on_b: Timestamp,
}

struct PacketData<'a>(&'a [u8]);

impl<'a> core::fmt::Debug for PacketData<'a> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        write!(formatter, "{:?}", self.0)
    }
}

impl core::fmt::Debug for Packet {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        // Remember: if you alter the definition of `Packet`,
        // 1. update the formatter debug struct builder calls (return object of
        //    this function)
        // 2. update this destructuring assignment accordingly
        let Packet {
            seq_on_a: _,
            port_id_on_a: _,
            chan_id_on_a: _,
            port_id_on_b: _,
            chan_id_on_b: _,
            data,
            timeout_height_on_b: _,
            timeout_timestamp_on_b: _,
        } = self;
        let data_wrapper = PacketData(data);

        formatter
            .debug_struct("Packet")
            .field("sequence", &self.seq_on_a)
            .field("source_port", &self.port_id_on_a)
            .field("source_channel", &self.chan_id_on_a)
            .field("destination_port", &self.port_id_on_b)
            .field("destination_channel", &self.chan_id_on_b)
            .field("data", &data_wrapper)
            .field("timeout_height", &self.timeout_height_on_b)
            .field("timeout_timestamp", &self.timeout_timestamp_on_b)
            .finish()
    }
}

impl Packet {
    /// Checks whether a packet from a
    /// [`SendPacket`](crate::events::SendPacket)
    /// event is timed-out relative to the current state of the
    /// destination chain.
    ///
    /// Checks both for time-out relative to the destination chain's
    /// current timestamp `dst_chain_ts` as well as relative to
    /// the height `dst_chain_height`.
    ///
    /// Note: a timed-out packet should result in a
    /// [`MsgTimeout`](crate::msgs::MsgTimeout),
    /// instead of the common-case where it results in
    /// [`MsgRecvPacket`](crate::msgs::MsgRecvPacket).
    pub fn timed_out(&self, dst_chain_ts: &Timestamp, dst_chain_height: Height) -> bool {
        let height_timed_out = self.timeout_height_on_b.has_expired(dst_chain_height);

        let timestamp_timed_out = self.timeout_timestamp_on_b.is_set()
            && dst_chain_ts.check_expiry(&self.timeout_timestamp_on_b) == Expired;

        height_timed_out || timestamp_timed_out
    }
}

/// Custom debug output to omit the packet data
impl core::fmt::Display for Packet {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        write!(
            f,
            "seq:{}, path:{}/{}->{}/{}, toh:{}, tos:{})",
            self.seq_on_a,
            self.chan_id_on_a,
            self.port_id_on_a,
            self.chan_id_on_b,
            self.port_id_on_b,
            self.timeout_height_on_b,
            self.timeout_timestamp_on_b
        )
    }
}

impl TryFrom<RawPacket> for Packet {
    type Error = PacketError;

    fn try_from(raw_pkt: RawPacket) -> Result<Self, Self::Error> {
        if Sequence::from(raw_pkt.sequence).is_zero() {
            return Err(PacketError::ZeroPacketSequence);
        }

        if raw_pkt.data.is_empty() {
            return Err(PacketError::ZeroPacketData);
        }

        // Note: ibc-go currently (July 2022) incorrectly treats the timeout
        // heights `{revision_number : >0, revision_height: 0}` as valid
        // timeouts. However, heights with `revision_height == 0` are invalid in
        // Tendermint. We explicitly reject these values because they go against
        // the Tendermint spec, and shouldn't be used. To timeout on the next
        // revision_number as soon as the chain starts,
        // `{revision_number: old_rev + 1, revision_height: 1}`
        // should be used.
        let packet_timeout_height: TimeoutHeight = raw_pkt
            .timeout_height
            .try_into()
            .map_err(|_| PacketError::InvalidTimeoutHeight)?;

        let timeout_timestamp_on_b = Timestamp::from_nanoseconds(raw_pkt.timeout_timestamp)
            .map_err(PacketError::InvalidPacketTimestamp)?;

        // Packet timeout height and packet timeout timestamp cannot both be unset.
        if !packet_timeout_height.is_set() && !timeout_timestamp_on_b.is_set() {
            return Err(PacketError::MissingTimeout);
        }

        Ok(Packet {
            seq_on_a: Sequence::from(raw_pkt.sequence),
            port_id_on_a: raw_pkt.source_port.parse()?,
            chan_id_on_a: raw_pkt.source_channel.parse()?,
            port_id_on_b: raw_pkt.destination_port.parse()?,
            chan_id_on_b: raw_pkt.destination_channel.parse()?,
            data: raw_pkt.data,
            timeout_height_on_b: packet_timeout_height,
            timeout_timestamp_on_b,
        })
    }
}

impl From<Packet> for RawPacket {
    fn from(packet: Packet) -> Self {
        RawPacket {
            sequence: packet.seq_on_a.value(),
            source_port: packet.port_id_on_a.to_string(),
            source_channel: packet.chan_id_on_a.to_string(),
            destination_port: packet.port_id_on_b.to_string(),
            destination_channel: packet.chan_id_on_b.to_string(),
            data: packet.data,
            timeout_height: packet.timeout_height_on_b.into(),
            timeout_timestamp: packet.timeout_timestamp_on_b.nanoseconds(),
        }
    }
}

/// The packet state type.
///
/// Each application defines the structure of the `data` field.
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
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Hash, PartialEq, Eq)]
pub struct PacketState {
    pub port_id: PortId,
    pub chan_id: ChannelId,
    pub seq: Sequence,
    #[cfg_attr(
        feature = "serde",
        serde(serialize_with = "ibc_core_commitment_types::serializer::ser_hex_upper")
    )]
    pub data: Vec<u8>,
}
impl core::fmt::Debug for PacketState {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        let data_wrapper = PacketData(&self.data);

        formatter
            .debug_struct("PacketState")
            .field("port", &self.port_id)
            .field("channel", &self.chan_id)
            .field("sequence", &self.seq)
            .field("data", &data_wrapper)
            .finish()
    }
}

/// Custom debug output to omit the packet data
impl core::fmt::Display for PacketState {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        write!(
            f,
            "seq:{}, path:{}/{}",
            self.seq, self.chan_id, self.port_id,
        )
    }
}

impl TryFrom<RawPacketState> for PacketState {
    type Error = PacketError;

    fn try_from(raw_pkt: RawPacketState) -> Result<Self, Self::Error> {
        if Sequence::from(raw_pkt.sequence).is_zero() {
            return Err(PacketError::ZeroPacketSequence);
        }

        if raw_pkt.data.is_empty() {
            return Err(PacketError::ZeroPacketData);
        }

        Ok(PacketState {
            seq: Sequence::from(raw_pkt.sequence),
            port_id: raw_pkt.port_id.parse()?,
            chan_id: raw_pkt.channel_id.parse()?,
            data: raw_pkt.data,
        })
    }
}

impl From<PacketState> for RawPacketState {
    fn from(packet: PacketState) -> Self {
        Self {
            sequence: packet.seq.value(),
            port_id: packet.port_id.to_string(),
            channel_id: packet.chan_id.to_string(),
            data: packet.data,
        }
    }
}
