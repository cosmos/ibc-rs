use bon::builder;
use ibc::apps::transfer::types::msgs::transfer::MsgTransfer;
use ibc::apps::transfer::types::packet::PacketData;
use ibc::apps::transfer::types::{Memo, PrefixedCoin};
use ibc::core::channel::types::packet::Packet;
use ibc::core::channel::types::timeout::{TimeoutHeight, TimeoutTimestamp};
use ibc::core::host::types::identifiers::{ChannelId, PortId, Sequence};
use ibc::core::primitives::Signer;

use crate::fixtures::core::signer::dummy_account_id;

/// Returns a dummy [`MsgTransfer`], for testing purposes only!
#[builder]
pub fn dummy_msg_transfer(
    #[builder(start_fn)] packet_data: PacketData,
    #[builder(default = PortId::transfer())] port_id_on_a: PortId,
    #[builder(default = ChannelId::zero())] chan_id_on_a: ChannelId,
    #[builder(default = TimeoutHeight::Never)] timeout_height_on_b: TimeoutHeight,
    #[builder(default = TimeoutTimestamp::Never)] timeout_timestamp_on_b: TimeoutTimestamp,
) -> MsgTransfer {
    MsgTransfer {
        port_id_on_a,
        chan_id_on_a,
        packet_data,
        timeout_height_on_b,
        timeout_timestamp_on_b,
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

/// Returns a dummy [`PacketData`], for testing purposes only!
#[builder]
pub fn dummy_packet_data(
    #[builder(start_fn)] token: PrefixedCoin,
    #[builder(default = dummy_account_id())] sender: Signer,
    #[builder(default = dummy_account_id())] receiver: Signer,
    #[builder(default = "".into())] memo: Memo,
) -> PacketData {
    PacketData {
        token,
        sender,
        receiver,
        memo,
    }
}
