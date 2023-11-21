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
    #[builder(default)]
    pub seq_on_a: Sequence,
    #[builder(default = PortId::transfer())]
    pub port_id_on_a: PortId,
    #[builder(default)]
    pub chan_id_on_a: ChannelId,
    #[builder(default = PortId::transfer())]
    pub port_id_on_b: PortId,
    #[builder(default)]
    pub chan_id_on_b: ChannelId,
    #[builder(default)]
    pub data: Vec<u8>,
    #[builder(default)]
    pub timeout_height_on_b: TimeoutHeight,
    #[builder(default)]
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
        source_channel: ChannelId::default().to_string(),
        destination_port: PortId::transfer().to_string(),
        destination_channel: ChannelId::default().to_string(),
        data: vec![0],
        timeout_height: Some(RawHeight {
            revision_number: 0,
            revision_height: timeout_height,
        }),
        timeout_timestamp,
    }
}

pub fn dummy_proof() -> Vec<u8> {
    "Y29uc2Vuc3VzU3RhdGUvaWJjb25lY2xpZW50LzIy"
        .as_bytes()
        .to_vec()
}
