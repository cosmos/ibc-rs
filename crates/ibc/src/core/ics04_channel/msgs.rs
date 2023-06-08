//! Message definitions for all ICS4 domain types: channel open & close handshake datagrams, as well
//! as packets.

pub(crate) mod acknowledgement;
pub(crate) mod chan_close_confirm;
pub(crate) mod chan_close_init;
pub(crate) mod chan_open_ack;
pub(crate) mod chan_open_confirm;
pub(crate) mod chan_open_init;
pub(crate) mod chan_open_try;
pub(crate) mod recv_packet;
pub(crate) mod timeout;
pub(crate) mod timeout_on_close;

// Opening handshake messages.
pub use chan_open_ack::MsgChannelOpenAck;
pub use chan_open_confirm::MsgChannelOpenConfirm;
pub use chan_open_init::MsgChannelOpenInit;
pub use chan_open_try::MsgChannelOpenTry;

// Closing handshake messages.
pub use chan_close_confirm::MsgChannelCloseConfirm;
pub use chan_close_init::MsgChannelCloseInit;

// Packet specific messages.
pub use acknowledgement::MsgAcknowledgement;
pub use recv_packet::MsgRecvPacket;
pub use timeout::MsgTimeout;
pub use timeout_on_close::MsgTimeoutOnClose;

use crate::core::ics24_host::identifier::PortId;

/// All channel messages
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ChannelMsg {
    OpenInit(MsgChannelOpenInit),
    OpenTry(MsgChannelOpenTry),
    OpenAck(MsgChannelOpenAck),
    OpenConfirm(MsgChannelOpenConfirm),
    CloseInit(MsgChannelCloseInit),
    CloseConfirm(MsgChannelCloseConfirm),
}

/// All packet messages
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PacketMsg {
    Recv(MsgRecvPacket),
    Ack(MsgAcknowledgement),
    Timeout(MsgTimeout),
    TimeoutOnClose(MsgTimeoutOnClose),
}

pub(crate) fn channel_msg_to_port_id(msg: &ChannelMsg) -> PortId {
    match msg {
        ChannelMsg::OpenInit(msg) => &msg.port_id_on_a,
        ChannelMsg::OpenTry(msg) => &msg.port_id_on_b,
        ChannelMsg::OpenAck(msg) => &msg.port_id_on_a,
        ChannelMsg::OpenConfirm(msg) => &msg.port_id_on_b,
        ChannelMsg::CloseInit(msg) => &msg.port_id_on_a,
        ChannelMsg::CloseConfirm(msg) => &msg.port_id_on_b,
    }
    .clone()
}

pub(crate) fn packet_msg_to_port_id(msg: &PacketMsg) -> PortId {
    match msg {
        PacketMsg::Recv(msg) => &msg.packet.port_id_on_b,
        PacketMsg::Ack(msg) => &msg.packet.port_id_on_a,
        PacketMsg::Timeout(msg) => &msg.packet.port_id_on_a,
        PacketMsg::TimeoutOnClose(msg) => &msg.packet.port_id_on_a,
    }
    .clone()
}
