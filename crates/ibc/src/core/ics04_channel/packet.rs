use crate::prelude::*;

use core::str::FromStr;

use ibc_proto::ibc::core::channel::v1::Packet as RawPacket;

use super::timeout::TimeoutHeight;
use crate::core::ics04_channel::error::{ChannelError, PacketError};
use crate::core::ics24_host::identifier::{ChannelId, PortId};
use crate::timestamp::{Expiry::Expired, Timestamp};
use crate::Height;

/// Enumeration of proof carrying ICS4 message, helper for relayer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PacketMsgType {
    Recv,
    Ack,
    TimeoutUnordered,
    TimeoutOrdered,
    TimeoutOnClose,
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
/// The sequence number of a packet enforces ordering among packets from the same source.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Sequence(u64);

impl FromStr for Sequence {
    type Err = ChannelError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from(s.parse::<u64>().map_err(|e| {
            ChannelError::InvalidStringAsSequence {
                value: s.to_string(),
                error: e,
            }
        })?))
    }
}

impl Sequence {
    pub fn is_zero(&self) -> bool {
        self.0 == 0
    }

    pub fn increment(&self) -> Sequence {
        Sequence(self.0 + 1)
    }
}

impl From<u64> for Sequence {
    fn from(seq: u64) -> Self {
        Sequence(seq)
    }
}

impl From<Sequence> for u64 {
    fn from(s: Sequence) -> u64 {
        s.0
    }
}

impl core::fmt::Display for Sequence {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        write!(f, "{}", self.0)
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
#[derive(Clone, Default, Hash, PartialEq, Eq)]
pub struct Packet {
    pub sequence: Sequence,
    pub port_on_a: PortId,
    pub chan_on_a: ChannelId,
    pub port_on_b: PortId,
    pub chan_on_b: ChannelId,
    #[cfg_attr(
        feature = "serde",
        serde(serialize_with = "crate::serializers::ser_hex_upper")
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
            sequence: _,
            port_on_a: _,
            chan_on_a: _,
            port_on_b: _,
            chan_on_b: _,
            data,
            timeout_height_on_b: _,
            timeout_timestamp_on_b: _,
        } = self;
        let data_wrapper = PacketData(data);

        formatter
            .debug_struct("Packet")
            .field("sequence", &self.sequence)
            .field("source_port", &self.port_on_a)
            .field("source_channel", &self.chan_on_a)
            .field("destination_port", &self.port_on_b)
            .field("destination_channel", &self.chan_on_b)
            .field("data", &data_wrapper)
            .field("timeout_height", &self.timeout_height_on_b)
            .field("timeout_timestamp", &self.timeout_timestamp_on_b)
            .finish()
    }
}

impl Packet {
    /// Checks whether a packet from a
    /// [`SendPacket`](crate::core::ics04_channel::events::SendPacket)
    /// event is timed-out relative to the current state of the
    /// destination chain.
    ///
    /// Checks both for time-out relative to the destination chain's
    /// current timestamp `dst_chain_ts` as well as relative to
    /// the height `dst_chain_height`.
    ///
    /// Note: a timed-out packet should result in a
    /// [`MsgTimeout`](crate::core::ics04_channel::msgs::timeout::MsgTimeout),
    /// instead of the common-case where it results in
    /// [`MsgRecvPacket`](crate::core::ics04_channel::msgs::recv_packet::MsgRecvPacket).
    pub fn timed_out(&self, dst_chain_ts: &Timestamp, dst_chain_height: Height) -> bool {
        let height_timed_out = self.timeout_height_on_b.has_expired(dst_chain_height);

        let timestamp_timed_out = self.timeout_timestamp_on_b != Timestamp::none()
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
            self.sequence,
            self.chan_on_a,
            self.port_on_a,
            self.chan_on_b,
            self.port_on_b,
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

        if raw_pkt.data.is_empty() {
            return Err(PacketError::ZeroPacketData);
        }

        let timeout_timestamp_on_b = Timestamp::from_nanoseconds(raw_pkt.timeout_timestamp)
            .map_err(PacketError::InvalidPacketTimestamp)?;

        Ok(Packet {
            sequence: Sequence::from(raw_pkt.sequence),
            port_on_a: raw_pkt
                .source_port
                .parse()
                .map_err(PacketError::Identifier)?,
            chan_on_a: raw_pkt
                .source_channel
                .parse()
                .map_err(PacketError::Identifier)?,
            port_on_b: raw_pkt
                .destination_port
                .parse()
                .map_err(PacketError::Identifier)?,
            chan_on_b: raw_pkt
                .destination_channel
                .parse()
                .map_err(PacketError::Identifier)?,
            data: raw_pkt.data,
            timeout_height_on_b: packet_timeout_height,
            timeout_timestamp_on_b,
        })
    }
}

impl From<Packet> for RawPacket {
    fn from(packet: Packet) -> Self {
        RawPacket {
            sequence: packet.sequence.0,
            source_port: packet.port_on_a.to_string(),
            source_channel: packet.chan_on_a.to_string(),
            destination_port: packet.port_on_b.to_string(),
            destination_channel: packet.chan_on_b.to_string(),
            data: packet.data,
            timeout_height: packet.timeout_height_on_b.into(),
            timeout_timestamp: packet.timeout_timestamp_on_b.nanoseconds(),
        }
    }
}

#[cfg(test)]
pub mod test_utils {
    use crate::prelude::*;
    use ibc_proto::ibc::core::channel::v1::Packet as RawPacket;
    use ibc_proto::ibc::core::client::v1::Height as RawHeight;

    use crate::core::ics24_host::identifier::{ChannelId, PortId};

    /// Returns a dummy `RawPacket`, for testing only!
    pub fn get_dummy_raw_packet(timeout_height: u64, timeout_timestamp: u64) -> RawPacket {
        RawPacket {
            sequence: 1,
            source_port: PortId::default().to_string(),
            source_channel: ChannelId::default().to_string(),
            destination_port: PortId::default().to_string(),
            destination_channel: ChannelId::default().to_string(),
            data: vec![0],
            timeout_height: Some(RawHeight {
                revision_number: 0,
                revision_height: timeout_height,
            }),
            timeout_timestamp,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    use test_log::test;

    use ibc_proto::ibc::core::channel::v1::Packet as RawPacket;
    use ibc_proto::ibc::core::client::v1::Height as RawHeight;

    use crate::core::ics04_channel::packet::test_utils::get_dummy_raw_packet;
    use crate::core::ics04_channel::packet::Packet;

    #[test]
    fn packet_try_from_raw() {
        struct Test {
            name: String,
            raw: RawPacket,
            want_pass: bool,
        }

        let proof_height = 10;
        let default_raw_packet = get_dummy_raw_packet(proof_height, 0);
        let raw_packet_no_timeout_or_timestamp = get_dummy_raw_packet(0, 0);

        let mut raw_packet_invalid_timeout_height = get_dummy_raw_packet(0, 0);
        raw_packet_invalid_timeout_height.timeout_height = Some(RawHeight {
            revision_number: 1,
            revision_height: 0,
        });

        let tests: Vec<Test> = vec![
            Test {
                name: "Good parameters".to_string(),
                raw: default_raw_packet.clone(),
                want_pass: true,
            },
            Test {
                // Note: ibc-go currently (July 2022) incorrectly rejects this
                // case, even though it is allowed in ICS-4.
                name: "Packet with no timeout of timestamp".to_string(),
                raw: raw_packet_no_timeout_or_timestamp,
                want_pass: true,
            },
            Test {
                name: "Packet with invalid timeout height".to_string(),
                raw: raw_packet_invalid_timeout_height,
                want_pass: false,
            },
            Test {
                name: "Src port validation: correct".to_string(),
                raw: RawPacket {
                    source_port: "srcportp34".to_string(),
                    ..default_raw_packet.clone()
                },
                want_pass: true,
            },
            Test {
                name: "Bad src port, name too short".to_string(),
                raw: RawPacket {
                    source_port: "p".to_string(),
                    ..default_raw_packet.clone()
                },
                want_pass: false,
            },
            Test {
                name: "Bad src port, name too long".to_string(),
                raw: RawPacket {
                    source_port: "abcdefghijasdfasdfasdfasdfasdfasdfasdfasdfasdfasdfadgasgasdfasdfasdfasdfaklmnopqrstuabcdefghijasdfasdfasdfasdfasdfasdfasdfasdfasdfasdfadgasgasdfasdfasdfasdfaklmnopqrstu".to_string(),
                    ..default_raw_packet.clone()
                },
                want_pass: false,
            },
            Test {
                name: "Dst port validation: correct".to_string(),
                raw: RawPacket {
                    destination_port: "destportsrcp34".to_string(),
                    ..default_raw_packet.clone()
                },
                want_pass: true,
            },
            Test {
                name: "Bad dst port, name too short".to_string(),
                raw: RawPacket {
                    destination_port: "p".to_string(),
                    ..default_raw_packet.clone()
                },
                want_pass: false,
            },
            Test {
                name: "Bad dst port, name too long".to_string(),
                raw: RawPacket {
                    destination_port: "abcdefghijasdfasdfasdfasdfasdfasdfasdfasdfasdfasdfadgasgasdfasdfasdfasdfaklmnopqrstuabcdefghijasdfasdfasdfasdfasdfasdfasdfasdfasdfasdfadgasgasdfas".to_string(),
                    ..default_raw_packet.clone()
                },
                want_pass: false,
            },
            Test {
                name: "Src channel validation: correct".to_string(),
                raw: RawPacket {
                    source_channel: "channel-1".to_string(),
                    ..default_raw_packet.clone()
                },
                want_pass: true,
            },
            Test {
                name: "Bad src channel, name too short".to_string(),
                raw: RawPacket {
                    source_channel: "p".to_string(),
                    ..default_raw_packet.clone()
                },
                want_pass: false,
            },
            Test {
                name: "Bad src channel, name too long".to_string(),
                raw: RawPacket {
                    source_channel: "channel-128391283791827398127398791283912837918273981273987912839".to_string(),
                    ..default_raw_packet.clone()
                },
                want_pass: false,
            },
            Test {
                name: "Dst channel validation: correct".to_string(),
                raw: RawPacket {
                    destination_channel: "channel-34".to_string(),
                    ..default_raw_packet.clone()
                },
                want_pass: true,
            },
            Test {
                name: "Bad dst channel, name too short".to_string(),
                raw: RawPacket {
                    destination_channel: "p".to_string(),
                    ..default_raw_packet.clone()
                },
                want_pass: false,
            },
            Test {
                name: "Bad dst channel, name too long".to_string(),
                raw: RawPacket {
                    destination_channel: "channel-128391283791827398127398791283912837918273981273987912839".to_string(),
                    ..default_raw_packet.clone()
                },
                want_pass: false,
            },
            // Note: `timeout_height == None` is a quirk of protobuf. In
            // `proto3` syntax, all structs are "nullable" by default and are
            // represented as `Option<T>`. `ibc-go` defines the protobuf file
            // with the extension option `gogoproto.nullable = false`, which
            // means that they will always generate a field. It is left
            // unspecified what a `None` value means. In this case, I believe it
            // is best to assume the obvious semantic of "no timeout".
            Test {
                name: "Missing timeout height".to_string(),
                raw: RawPacket {
                    timeout_height: None,
                    ..default_raw_packet
                },
                want_pass: true,
            },
        ];

        for test in tests {
            let res_msg = Packet::try_from(test.raw.clone());

            assert_eq!(
                test.want_pass,
                res_msg.is_ok(),
                "Packet::try_from failed for test {}, \nraw packet {:?} with error {:?}",
                test.name,
                test.raw,
                res_msg.err(),
            );
        }
    }

    #[test]
    fn to_and_from() {
        let raw = get_dummy_raw_packet(15, 0);
        let msg = Packet::try_from(raw.clone()).unwrap();
        let raw_back = RawPacket::from(msg.clone());
        let msg_back = Packet::try_from(raw_back.clone()).unwrap();
        assert_eq!(raw, raw_back);
        assert_eq!(msg, msg_back);
    }
}
