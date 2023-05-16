//! Defines the `Router`, which binds modules to ports

use crate::prelude::*;

use crate::core::ics04_channel::error::{ChannelError, PortError::UnknownPort};
use crate::core::ics04_channel::msgs::ChannelMsg;
use crate::core::ics04_channel::msgs::PacketMsg;
use crate::core::ics24_host::identifier::PortId;

use super::module::{Module, ModuleId};

/// Router as defined in ICS-26, which binds modules to ports.
pub trait Router {
    /// Returns a reference to a `Module` registered against the specified `ModuleId`
    fn get_route(&self, module_id: &ModuleId) -> Option<&dyn Module>;

    /// Returns a mutable reference to a `Module` registered against the specified `ModuleId`
    fn get_route_mut(&mut self, module_id: &ModuleId) -> Option<&mut dyn Module>;

    /// Returns true if the `Router` has a `Module` registered against the specified `ModuleId`
    fn has_route(&self, module_id: &ModuleId) -> bool;

    /// Return the module_id associated with a given port_id
    fn lookup_module_by_port(&self, port_id: &PortId) -> Option<ModuleId>;

    fn lookup_module_channel(&self, msg: &ChannelMsg) -> Result<ModuleId, ChannelError> {
        let port_id = match msg {
            ChannelMsg::OpenInit(msg) => &msg.port_id_on_a,
            ChannelMsg::OpenTry(msg) => &msg.port_id_on_b,
            ChannelMsg::OpenAck(msg) => &msg.port_id_on_a,
            ChannelMsg::OpenConfirm(msg) => &msg.port_id_on_b,
            ChannelMsg::CloseInit(msg) => &msg.port_id_on_a,
            ChannelMsg::CloseConfirm(msg) => &msg.port_id_on_b,
        };
        let module_id = self
            .lookup_module_by_port(port_id)
            .ok_or(ChannelError::Port(UnknownPort {
                port_id: port_id.clone(),
            }))?;
        Ok(module_id)
    }

    fn lookup_module_packet(&self, msg: &PacketMsg) -> Result<ModuleId, ChannelError> {
        let port_id = match msg {
            PacketMsg::Recv(msg) => &msg.packet.port_id_on_b,
            PacketMsg::Ack(msg) => &msg.packet.port_id_on_a,
            PacketMsg::Timeout(msg) => &msg.packet.port_id_on_a,
            PacketMsg::TimeoutOnClose(msg) => &msg.packet.port_id_on_a,
        };
        let module_id = self
            .lookup_module_by_port(port_id)
            .ok_or(ChannelError::Port(UnknownPort {
                port_id: port_id.clone(),
            }))?;
        Ok(module_id)
    }
}
