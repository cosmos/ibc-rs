use ibc::core::channel::types::packet::Packet;
use ibc::core::channel::types::proto::v1::Packet as RawPacket;
use ibc::core::channel::types::timeout::TimeoutHeight;
use ibc::core::client::types::proto::v1::Height as RawHeight;
use ibc::core::host::types::identifiers::{ChannelId, PortId, Sequence};
use ibc::core::primitives::prelude::*;
use ibc::core::primitives::Timestamp;
use typed_builder::TypedBuilder;

/// Configuration of the `PacketData` type for building dummy packets.
#[derive(TypedBuilder, Debug)]
#[builder(build_method(into = Packet))]
pub struct PacketConfig {
    #[builder(default = Sequence::from(0))]
    pub seq_on_a: Sequence,
    #[builder(default = PortId::transfer())]
    pub port_id_on_a: PortId,
    #[builder(default = ChannelId::zero())]
    pub chan_id_on_a: ChannelId,
    #[builder(default = PortId::transfer())]
    pub port_id_on_b: PortId,
    #[builder(default = ChannelId::zero())]
    pub chan_id_on_b: ChannelId,
    #[builder(default)]
    pub data: Vec<u8>,
    #[builder(default = TimeoutHeight::Never)]
    pub timeout_height_on_b: TimeoutHeight,
    #[builder(default = Timestamp::none())]
    pub timeout_timestamp_on_b: Timestamp,
}

impl From<PacketConfig> for Packet {
    fn from(config: PacketConfig) -> Self {
        Packet {
            seq_on_a: config.seq_on_a,
            port_id_on_a: config.port_id_on_a,
            chan_id_on_a: config.chan_id_on_a,
            port_id_on_b: config.port_id_on_b,
            chan_id_on_b: config.chan_id_on_b,
            data: config.data,
            timeout_height_on_b: config.timeout_height_on_b,
            timeout_timestamp_on_b: config.timeout_timestamp_on_b,
        }
    }
}

/// Returns a dummy `RawPacket`, for testing purposes only!
pub fn dummy_raw_packet(timeout_height: u64, timeout_timestamp: u64) -> RawPacket {
    RawPacket {
        sequence: 1,
        source_port: PortId::transfer().to_string(),
        source_channel: ChannelId::zero().to_string(),
        destination_port: PortId::transfer().to_string(),
        destination_channel: ChannelId::zero().to_string(),
        data: vec![0],
        timeout_height: Some(RawHeight {
            revision_number: 0,
            revision_height: timeout_height,
        }),
        timeout_timestamp,
    }
}

pub fn dummy_proof() -> Vec<u8> {
    b"Y29uc2Vuc3VzU3RhdGUvaWJjb25lY2xpZW50LzIy".to_vec()
}

#[cfg(test)]
mod tests {
    use ibc::core::channel::types::channel::Order;
    use ibc::core::channel::types::events::SendPacket;
    use ibc::core::handler::types::events::IbcEvent;
    use ibc::core::host::types::identifiers::ConnectionId;

    use super::*;

    #[test]
    fn packet_try_from_raw() {
        struct Test {
            name: String,
            raw: RawPacket,
            want_pass: bool,
        }

        let proof_height = 10;
        let default_raw_packet = dummy_raw_packet(proof_height, 1000);
        let raw_packet_no_timeout_or_timestamp = dummy_raw_packet(10, 0);

        let mut raw_packet_invalid_timeout_height = dummy_raw_packet(0, 10);
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
                raw: raw_packet_no_timeout_or_timestamp.clone(),
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
            Test {
                name: "Missing both timeout height and timestamp".to_string(),
                raw: RawPacket {
                    timeout_height: None,
                    ..raw_packet_no_timeout_or_timestamp
                },
                want_pass: false,
            }
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
        let raw = dummy_raw_packet(15, 0);
        let msg = Packet::try_from(raw.clone()).unwrap();
        let raw_back = RawPacket::from(msg.clone());
        let msg_back = Packet::try_from(raw_back.clone()).unwrap();
        assert_eq!(raw, raw_back);
        assert_eq!(msg, msg_back);
    }

    #[test]
    /// Ensures that we don't panic when packet data is not valid UTF-8.
    /// See issue [#199](https://github.com/cosmos/ibc-rs/issues/199)
    pub fn test_packet_data_non_utf8() {
        let mut packet = Packet::try_from(dummy_raw_packet(1, 1)).unwrap();
        packet.data = vec![128];

        let ibc_event = IbcEvent::SendPacket(SendPacket::new(
            packet,
            Order::Unordered,
            ConnectionId::zero(),
        ));
        let _ = tendermint::abci::Event::try_from(ibc_event);
    }
}
