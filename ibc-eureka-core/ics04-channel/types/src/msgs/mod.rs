//! Message definitions for all ICS4 domain types: channel open & close handshake datagrams, as well
//! as packets.

mod acknowledgement;
mod recv_packet;
mod timeout;
mod timeout_on_close;

// Opening handshake messages.
// Packet specific messages.
pub use acknowledgement::*;
use ibc_eureka_core_host_types::identifiers::*;
use ibc_primitives::prelude::*;
pub use recv_packet::*;
pub use timeout::*;
pub use timeout_on_close::*;

/// All packet messages
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, PartialEq, Eq, derive_more::From)]
pub enum PacketMsg {
    Recv(MsgRecvPacket),
    Ack(MsgAcknowledgement),
    Timeout(MsgTimeout),
}

pub fn packet_msg_to_port_id(msg: &PacketMsg) -> &PortId {
    match msg {
        PacketMsg::Recv(msg) => &msg.packet.payloads[0].header.target_port,
        PacketMsg::Ack(msg) => &msg.packet.payloads[0].header.source_port,
        PacketMsg::Timeout(msg) => &msg.packet.payloads[0].header.source_port,
    }
}
