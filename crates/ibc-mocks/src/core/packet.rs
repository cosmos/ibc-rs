use ibc::core::ics04_channel::packet::{Packet, Sequence};
use ibc::core::ics04_channel::timeout::TimeoutHeight;
use ibc::core::ics24_host::identifier::{ChannelId, PortId};
use ibc::core::timestamp::Timestamp;
use ibc::prelude::*;
use typed_builder::TypedBuilder;

/// Configuration for a `PacketData` type.
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
