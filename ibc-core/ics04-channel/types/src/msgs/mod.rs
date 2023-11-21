//! Message definitions for all ICS4 domain types: channel open & close handshake datagrams, as well
//! as packets.

mod acknowledgement;
mod chan_close_confirm;
mod chan_close_init;
mod chan_open_ack;
mod chan_open_confirm;
mod chan_open_init;
mod chan_open_try;
mod recv_packet;
mod timeout;
mod timeout_on_close;

// Opening handshake messages.
// Packet specific messages.
pub use acknowledgement::*;
// Closing handshake messages.
pub use chan_close_confirm::*;
pub use chan_close_init::*;
pub use chan_open_ack::*;
pub use chan_open_confirm::*;
pub use chan_open_init::*;
pub use chan_open_try::*;
use ibc_core_host_types::identifiers::*;
use ibc_primitives::prelude::*;
pub use recv_packet::*;
pub use timeout::*;
pub use timeout_on_close::*;

/// All channel messages
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, PartialEq, Eq, derive_more::From)]
pub enum ChannelMsg {
    OpenInit(MsgChannelOpenInit),
    OpenTry(MsgChannelOpenTry),
    OpenAck(MsgChannelOpenAck),
    OpenConfirm(MsgChannelOpenConfirm),
    CloseInit(MsgChannelCloseInit),
    CloseConfirm(MsgChannelCloseConfirm),
}

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
    TimeoutOnClose(MsgTimeoutOnClose),
}

pub fn channel_msg_to_port_id(msg: &ChannelMsg) -> &PortId {
    match msg {
        ChannelMsg::OpenInit(msg) => &msg.port_id_on_a,
        ChannelMsg::OpenTry(msg) => &msg.port_id_on_b,
        ChannelMsg::OpenAck(msg) => &msg.port_id_on_a,
        ChannelMsg::OpenConfirm(msg) => &msg.port_id_on_b,
        ChannelMsg::CloseInit(msg) => &msg.port_id_on_a,
        ChannelMsg::CloseConfirm(msg) => &msg.port_id_on_b,
    }
}

pub fn packet_msg_to_port_id(msg: &PacketMsg) -> &PortId {
    match msg {
        PacketMsg::Recv(msg) => &msg.packet.port_id_on_b,
        PacketMsg::Ack(msg) => &msg.packet.port_id_on_a,
        PacketMsg::Timeout(msg) => &msg.packet.port_id_on_a,
        PacketMsg::TimeoutOnClose(msg) => &msg.packet.port_id_on_a,
    }
}
