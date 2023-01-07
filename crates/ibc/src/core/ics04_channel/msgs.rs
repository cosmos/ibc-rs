//! Message definitions for all ICS4 domain types: channel open & close handshake datagrams, as well
//! as packets.

use crate::core::ics04_channel::error::ChannelError;
use crate::core::ics04_channel::msgs::acknowledgement::MsgAcknowledgement;
use crate::core::ics04_channel::msgs::chan_close_confirm::MsgChannelCloseConfirm;
use crate::core::ics04_channel::msgs::chan_close_init::MsgChannelCloseInit;
use crate::core::ics04_channel::msgs::chan_open_ack::MsgChannelOpenAck;
use crate::core::ics04_channel::msgs::chan_open_confirm::MsgChannelOpenConfirm;
use crate::core::ics04_channel::msgs::chan_open_init::MsgChannelOpenInit;
use crate::core::ics04_channel::msgs::chan_open_try::MsgChannelOpenTry;
use crate::core::ics04_channel::msgs::recv_packet::MsgRecvPacket;
use crate::core::ics04_channel::msgs::timeout::MsgTimeout;
use crate::core::ics04_channel::msgs::timeout_on_close::MsgTimeoutOnClose;
use crate::core::ics26_routing::context::{ModuleId, RouterContext};

// Opening handshake messages.
pub mod chan_open_ack;
pub mod chan_open_confirm;
pub mod chan_open_init;
pub mod chan_open_try;

// Closing handshake messages.
pub mod chan_close_confirm;
pub mod chan_close_init;

// Packet specific messages.
pub mod acknowledgement;
pub mod recv_packet;
pub mod timeout;
pub mod timeout_on_close;

/// Enumeration of all possible messages that the ICS4 protocol processes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ChannelMsg {
    ChannelOpenInit(MsgChannelOpenInit),
    ChannelOpenTry(MsgChannelOpenTry),
    ChannelOpenAck(MsgChannelOpenAck),
    ChannelOpenConfirm(MsgChannelOpenConfirm),
    ChannelCloseInit(MsgChannelCloseInit),
    ChannelCloseConfirm(MsgChannelCloseConfirm),
}

impl ChannelMsg {
    pub(super) fn lookup_module(&self, ctx: &impl RouterContext) -> Result<ModuleId, ChannelError> {
        let port_id = match self {
            ChannelMsg::ChannelOpenInit(msg) => &msg.port_id_on_a,
            ChannelMsg::ChannelOpenTry(msg) => &msg.port_id_on_b,
            ChannelMsg::ChannelOpenAck(msg) => &msg.port_id_on_a,
            ChannelMsg::ChannelOpenConfirm(msg) => &msg.port_id_on_b,
            ChannelMsg::ChannelCloseInit(msg) => &msg.port_id_on_a,
            ChannelMsg::ChannelCloseConfirm(msg) => &msg.port_id_on_b,
        };
        let module_id = ctx
            .lookup_module_by_port(port_id)
            .map_err(ChannelError::Port)?;
        Ok(module_id)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PacketMsg {
    RecvPacket(MsgRecvPacket),
    AckPacket(MsgAcknowledgement),
    TimeoutPacket(MsgTimeout),
    TimeoutOnClosePacket(MsgTimeoutOnClose),
}

impl PacketMsg {
    pub(super) fn lookup_module(&self, ctx: &impl RouterContext) -> Result<ModuleId, ChannelError> {
        let port_id = match self {
            PacketMsg::RecvPacket(msg) => &msg.packet.port_on_b,
            PacketMsg::AckPacket(msg) => &msg.packet.port_on_a,
            PacketMsg::TimeoutPacket(msg) => &msg.packet.port_on_a,
            PacketMsg::TimeoutOnClosePacket(msg) => &msg.packet.port_on_a,
        };
        let module_id = ctx
            .lookup_module_by_port(port_id)
            .map_err(ChannelError::Port)?;
        Ok(module_id)
    }
}
