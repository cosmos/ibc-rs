use alloc::string::ToString;

use ibc::applications::transfer::msgs::transfer::MsgTransfer;
use ibc::applications::transfer::packet::PacketData;
use ibc::applications::transfer::{Memo, PrefixedCoin};
#[cfg(feature = "serde")]
use ibc::core::ics04_channel::packet::{Packet, Sequence};
use ibc::core::ics04_channel::timeout::TimeoutHeight;
use ibc::core::ics24_host::identifier::{ChannelId, PortId};
use ibc::core::timestamp::Timestamp;
use ibc::utils::dummy::get_dummy_account_id;
use ibc::Signer;
use typed_builder::TypedBuilder;

/// Configuration for a `MsgTransfer` message.
#[derive(TypedBuilder, Debug)]
#[builder(build_method(into = MsgTransfer))]
pub struct MsgTransferConfig {
    #[builder(default = PortId::transfer())]
    pub port_id_on_a: PortId,
    #[builder(default)]
    pub chan_id_on_a: ChannelId,
    pub packet_data: PacketData,
    #[builder(default)]
    pub timeout_height_on_b: TimeoutHeight,
    #[builder(default)]
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

#[cfg(feature = "serde")]
pub fn extract_transfer_packet(msg: &MsgTransfer, sequence: Sequence) -> Packet {
    let data = serde_json::to_vec(&msg.packet_data)
        .expect("PacketData's infallible Serialize impl failed");

    Packet {
        seq_on_a: sequence,
        port_id_on_a: msg.port_id_on_a.clone(),
        chan_id_on_a: msg.chan_id_on_a.clone(),
        port_id_on_b: PortId::transfer(),
        chan_id_on_b: ChannelId::default(),
        data,
        timeout_height_on_b: msg.timeout_height_on_b,
        timeout_timestamp_on_b: msg.timeout_timestamp_on_b,
    }
}

/// Configuration for a `PacketData` type.
#[derive(TypedBuilder, Debug)]
#[builder(build_method(into = PacketData))]
pub struct PacketDataConfig {
    pub token: PrefixedCoin,
    #[builder(default = get_dummy_account_id())]
    pub sender: Signer,
    #[builder(default = get_dummy_account_id())]
    pub receiver: Signer,
    #[builder(default = Memo::from("".to_string()))]
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
