use ibc::apps::transfer::types::msgs::transfer::MsgTransfer;
use ibc::apps::transfer::types::packet::PacketData;
use ibc::apps::transfer::types::{Memo, PrefixedCoin};
use ibc::core::channel::types::packet::Packet;
use ibc::core::channel::types::timeout::TimeoutHeight;
use ibc::core::host::types::identifiers::{ChannelId, PortId, Sequence};
use ibc::core::primitives::{Signer, Timestamp};
use typed_builder::TypedBuilder;

use crate::fixtures::core::signer::dummy_account_id;

/// Configuration of the `MsgTransfer` message for building dummy messages.
#[derive(TypedBuilder, Debug)]
#[builder(build_method(into = MsgTransfer))]
pub struct MsgTransferConfig {
    #[builder(default = PortId::transfer())]
    pub port_id_on_a: PortId,
    #[builder(default = ChannelId::zero())]
    pub chan_id_on_a: ChannelId,
    pub packet_data: PacketData,
    #[builder(default = TimeoutHeight::Never)]
    pub timeout_height_on_b: TimeoutHeight,
    #[builder(default = Timestamp::none())]
    pub timeout_timestamp_on_b: Timestamp,
}

impl From<MsgTransferConfig> for MsgTransfer {
    fn from(config: MsgTransferConfig) -> Self {
        MsgTransfer {
            port_id_on_a: config.port_id_on_a,
            chan_id_on_a: config.chan_id_on_a,
            packet_data: config.packet_data,
            timeout_height_on_b: config.timeout_height_on_b,
            timeout_timestamp_on_b: config.timeout_timestamp_on_b,
        }
    }
}

pub fn extract_transfer_packet(msg: &MsgTransfer, sequence: Sequence) -> Packet {
    let data = serde_json::to_vec(&msg.packet_data)
        .expect("PacketData's infallible Serialize impl failed");

    Packet {
        seq_on_a: sequence,
        port_id_on_a: msg.port_id_on_a.clone(),
        chan_id_on_a: msg.chan_id_on_a.clone(),
        port_id_on_b: PortId::transfer(),
        chan_id_on_b: ChannelId::zero(),
        data,
        timeout_height_on_b: msg.timeout_height_on_b,
        timeout_timestamp_on_b: msg.timeout_timestamp_on_b,
    }
}

/// Configuration of the `PacketData` type for building dummy packets.
#[derive(TypedBuilder, Debug)]
#[builder(build_method(into = PacketData))]
pub struct PacketDataConfig {
    pub token: PrefixedCoin,
    #[builder(default = dummy_account_id())]
    pub sender: Signer,
    #[builder(default = dummy_account_id())]
    pub receiver: Signer,
    #[builder(default = "".into())]
    pub memo: Memo,
}

impl From<PacketDataConfig> for PacketData {
    fn from(config: PacketDataConfig) -> Self {
        PacketData {
            token: config.token,
            sender: config.sender,
            receiver: config.receiver,
            memo: config.memo,
        }
    }
}
