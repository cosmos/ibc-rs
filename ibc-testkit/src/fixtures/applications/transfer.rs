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
#[builder(finish_fn = build)]
pub fn dummy_msg_transfer(
    port_id_on_a: Option<PortId>,
    chan_id_on_a: Option<ChannelId>,
    packet_data: PacketData,
    timeout_height_on_b: Option<TimeoutHeight>,
    timeout_timestamp_on_b: Option<TimeoutTimestamp>,
) -> MsgTransfer {
    MsgTransfer {
        port_id_on_a: port_id_on_a.unwrap_or_else(PortId::transfer),
        chan_id_on_a: chan_id_on_a.unwrap_or_else(ChannelId::zero),
        packet_data,
        timeout_height_on_b: timeout_height_on_b.unwrap_or(TimeoutHeight::Never),
        timeout_timestamp_on_b: timeout_timestamp_on_b.unwrap_or(TimeoutTimestamp::Never),
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
#[builder(finish_fn = build)]
pub fn dummy_packet_data(
    token: PrefixedCoin,
    sender: Option<Signer>,
    receiver: Option<Signer>,
    memo: Option<Memo>,
) -> PacketData {
    PacketData {
        token,
        sender: sender.unwrap_or_else(dummy_account_id),
        receiver: receiver.unwrap_or_else(dummy_account_id),
        memo: memo.unwrap_or_else(|| "".into()),
    }
}
