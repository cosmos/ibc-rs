//! Defines the packet type
use ibc_eureka_core_client_types::Height;
use ibc_eureka_core_commitment_types::commitment::CommitmentPrefix;
use ibc_eureka_core_host_types::error::DecodingError;
use ibc_eureka_core_host_types::identifiers::{ChannelId, PortId, Sequence};
use ibc_primitives::prelude::*;
use ibc_primitives::Timestamp;
use ibc_proto::ibc::core::channel::v1::{Packet as RawPacket, PacketState as RawPacketState};

use super::timeout::TimeoutHeight;
use crate::timeout::TimeoutTimestamp;

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
///
/// If the receipt is present in the host's state, it's marked as `Ok`,
/// indicating the packet has already been processed. If the receipt is absent,
/// it's marked as `None`, meaning the packet has not been received.
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
    None,
}

impl Receipt {
    pub fn is_ok(&self) -> bool {
        matches!(self, Receipt::Ok)
    }

    pub fn is_none(&self) -> bool {
        matches!(self, Receipt::None)
    }
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
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct PacketHeader {
    pub seq_on_a: Sequence,
    pub target_client_on_source: ChannelId,
    pub source_client_on_target: ChannelId,
    pub timeout_height_on_b: TimeoutHeight,
    pub timeout_timestamp_on_b: TimeoutTimestamp,
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
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct PayloadHeader {
    pub source_port: (CommitmentPrefix, PortId),
    pub target_port: (CommitmentPrefix, PortId),
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
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Payload {
    pub header: PayloadHeader,
    #[cfg_attr(
        feature = "serde",
        serde(serialize_with = "ibc_eureka_core_commitment_types::serializer::ser_hex_upper")
    )]
    pub data: Vec<u8>,
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
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Packet {
    pub header: PacketHeader,
    pub payloads: Vec<Payload>,
}

impl Packet {
    /// Checks whether a packet from a
    /// [`SendPacket`](crate::events::SendPacket)
    /// event is timed-out relative to the current state of the
    /// destination chain.
    ///
    /// Checks for time-out relative to the destination chain's
    /// current timestamp `dst_chain_ts` as well as relative to
    /// the height `dst_chain_height`.
    ///
    /// Note: a timed-out packet should result in a
    /// [`MsgTimeout`](crate::msgs::MsgTimeout),
    /// instead of the common-case where it results in
    /// [`MsgRecvPacket`](crate::msgs::MsgRecvPacket).
    pub fn timed_out(&self, dst_chain_ts: &Timestamp, dst_chain_height: Height) -> bool {
        let height_timed_out = self
            .header
            .timeout_height_on_b
            .has_expired(dst_chain_height);

        let timestamp_timed_out = self.header.timeout_timestamp_on_b.has_expired(dst_chain_ts);

        height_timed_out || timestamp_timed_out
    }
}

/// Custom debug output to omit the packet data
impl core::fmt::Display for Packet {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        write!(
            f,
            "seq:{}, path:{}/{}",
            self.header.seq_on_a,
            self.header.target_client_on_source,
            self.header.source_client_on_target
        )?;
        for payload in &self.payloads {
            write!(
                f,
                "src_port:{}, dst_port:{}",
                payload.header.source_port.1, payload.header.target_port.1
            )?;
        }
        Ok(())
    }
}

impl TryFrom<RawPacket> for Packet {
    type Error = DecodingError;

    fn try_from(raw_pkt: RawPacket) -> Result<Self, Self::Error> {
        if Sequence::from(raw_pkt.sequence).is_zero() {
            return Err(DecodingError::invalid_raw_data(
                "packet sequence cannot be 0",
            ));
        }

        if raw_pkt.data.is_empty() {
            return Err(DecodingError::missing_raw_data("packet data is not set"))?;
        }

        // Note: ibc-go currently (July 2022) incorrectly treats the timeout
        // heights `{revision_number : >0, revision_height: 0}` as valid
        // timeouts. However, heights with `revision_height == 0` are invalid in
        // Tendermint. We explicitly reject these values because they go against
        // the Tendermint spec, and shouldn't be used. To timeout on the next
        // revision_number as soon as the chain starts,
        // `{revision_number: old_rev + 1, revision_height: 1}`
        // should be used.
        let packet_timeout_height: TimeoutHeight = raw_pkt.timeout_height.try_into()?;

        let timeout_timestamp_on_b: TimeoutTimestamp = raw_pkt.timeout_timestamp.into();

        // Packet timeout height and packet timeout timestamp cannot both be unset.
        if !packet_timeout_height.is_set() && !timeout_timestamp_on_b.is_set() {
            return Err(DecodingError::missing_raw_data(
                "missing one of packet timeout height or timeout timestamp",
            ));
        }

        Ok(Packet {
            header: PacketHeader {
                seq_on_a: Sequence::from(raw_pkt.sequence),
                target_client_on_source: raw_pkt.source_channel.parse()?,
                source_client_on_target: raw_pkt.destination_channel.parse()?,
                timeout_height_on_b: packet_timeout_height,
                timeout_timestamp_on_b,
            },
            // TODO(rano): support multi payload; currently only one payload is supported
            payloads: vec![Payload {
                header: PayloadHeader {
                    source_port: (CommitmentPrefix::empty(), raw_pkt.source_port.parse()?),
                    target_port: (CommitmentPrefix::empty(), raw_pkt.destination_port.parse()?),
                },
                data: raw_pkt.data,
            }],
        })
    }
}

impl From<Packet> for RawPacket {
    fn from(packet: Packet) -> Self {
        Self {
            sequence: packet.header.seq_on_a.value(),
            source_channel: packet.header.target_client_on_source.to_string(),
            destination_channel: packet.header.source_client_on_target.to_string(),
            timeout_height: packet.header.timeout_height_on_b.into(),
            timeout_timestamp: packet.header.timeout_timestamp_on_b.nanoseconds(),
            // TODO(rano): support multi payload; currently only one payload is supported
            source_port: packet.payloads[0].header.source_port.1.to_string(),
            destination_port: packet.payloads[0].header.target_port.1.to_string(),
            data: packet.payloads[0].data.clone(),
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
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct PacketState {
    pub port_id: PortId,
    pub chan_id: ChannelId,
    pub seq: Sequence,
    #[cfg_attr(
        feature = "serde",
        serde(serialize_with = "ibc_eureka_core_commitment_types::serializer::ser_hex_upper")
    )]
    pub data: Vec<u8>,
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
    type Error = DecodingError;

    fn try_from(raw_pkt: RawPacketState) -> Result<Self, Self::Error> {
        if Sequence::from(raw_pkt.sequence).is_zero() {
            return Err(DecodingError::invalid_raw_data(
                "packet sequence cannot be 0",
            ));
        }

        if raw_pkt.data.is_empty() {
            return Err(DecodingError::missing_raw_data("packet data not set"))?;
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
