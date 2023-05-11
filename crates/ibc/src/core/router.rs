//! Defines the `Router`, which binds modules to ports

use crate::{core::events::ModuleEvent, prelude::*};

use alloc::borrow::Borrow;
use core::fmt::{Debug, Display, Error as FmtError, Formatter};

use crate::core::ics04_channel::channel::{Counterparty, Order};
use crate::core::ics04_channel::error::{ChannelError, PacketError, PortError::UnknownPort};
use crate::core::ics04_channel::msgs::ChannelMsg;
use crate::core::ics04_channel::packet::{Acknowledgement, Packet};
use crate::core::ics04_channel::Version;
use crate::core::ics24_host::identifier::{ChannelId, ConnectionId, PortId};
use crate::signer::Signer;

use super::ics04_channel::msgs::PacketMsg;

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

/// Module name, internal to the chain.
///
/// That is, the IBC protocol never exposes this name. Note that this is
/// different from IBC host [identifiers][crate::core::ics24_host::identifier],
/// which are exposed to other chains by the protocol.
#[cfg_attr(
    feature = "parity-scale-codec",
    derive(
        parity_scale_codec::Encode,
        parity_scale_codec::Decode,
        scale_info::TypeInfo
    )
)]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ModuleId(String);

impl ModuleId {
    pub fn new(s: String) -> Self {
        Self(s)
    }
}

impl Display for ModuleId {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        write!(f, "{}", self.0)
    }
}

impl Borrow<str> for ModuleId {
    fn borrow(&self) -> &str {
        self.0.as_str()
    }
}

/// Logs and events produced during module callbacks
#[cfg_attr(
    feature = "parity-scale-codec",
    derive(
        parity_scale_codec::Encode,
        parity_scale_codec::Decode,
        scale_info::TypeInfo
    )
)]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug)]
pub struct ModuleExtras {
    pub events: Vec<ModuleEvent>,
    pub log: Vec<String>,
}

impl ModuleExtras {
    pub fn empty() -> Self {
        ModuleExtras {
            events: Vec::new(),
            log: Vec::new(),
        }
    }
}

/// The trait that defines an IBC application
pub trait Module: Debug {
    fn on_chan_open_init_validate(
        &self,
        order: Order,
        connection_hops: &[ConnectionId],
        port_id: &PortId,
        channel_id: &ChannelId,
        counterparty: &Counterparty,
        version: &Version,
    ) -> Result<Version, ChannelError>;

    fn on_chan_open_init_execute(
        &mut self,
        order: Order,
        connection_hops: &[ConnectionId],
        port_id: &PortId,
        channel_id: &ChannelId,
        counterparty: &Counterparty,
        version: &Version,
    ) -> Result<(ModuleExtras, Version), ChannelError>;

    fn on_chan_open_try_validate(
        &self,
        order: Order,
        connection_hops: &[ConnectionId],
        port_id: &PortId,
        channel_id: &ChannelId,
        counterparty: &Counterparty,
        counterparty_version: &Version,
    ) -> Result<Version, ChannelError>;

    fn on_chan_open_try_execute(
        &mut self,
        order: Order,
        connection_hops: &[ConnectionId],
        port_id: &PortId,
        channel_id: &ChannelId,
        counterparty: &Counterparty,
        counterparty_version: &Version,
    ) -> Result<(ModuleExtras, Version), ChannelError>;

    fn on_chan_open_ack_validate(
        &self,
        _port_id: &PortId,
        _channel_id: &ChannelId,
        _counterparty_version: &Version,
    ) -> Result<(), ChannelError> {
        Ok(())
    }

    fn on_chan_open_ack_execute(
        &mut self,
        _port_id: &PortId,
        _channel_id: &ChannelId,
        _counterparty_version: &Version,
    ) -> Result<ModuleExtras, ChannelError> {
        Ok(ModuleExtras::empty())
    }

    fn on_chan_open_confirm_validate(
        &self,
        _port_id: &PortId,
        _channel_id: &ChannelId,
    ) -> Result<(), ChannelError> {
        Ok(())
    }

    fn on_chan_open_confirm_execute(
        &mut self,
        _port_id: &PortId,
        _channel_id: &ChannelId,
    ) -> Result<ModuleExtras, ChannelError> {
        Ok(ModuleExtras::empty())
    }

    fn on_chan_close_init_validate(
        &self,
        _port_id: &PortId,
        _channel_id: &ChannelId,
    ) -> Result<(), ChannelError> {
        Ok(())
    }

    fn on_chan_close_init_execute(
        &mut self,
        _port_id: &PortId,
        _channel_id: &ChannelId,
    ) -> Result<ModuleExtras, ChannelError> {
        Ok(ModuleExtras::empty())
    }

    fn on_chan_close_confirm_validate(
        &self,
        _port_id: &PortId,
        _channel_id: &ChannelId,
    ) -> Result<(), ChannelError> {
        Ok(())
    }

    fn on_chan_close_confirm_execute(
        &mut self,
        _port_id: &PortId,
        _channel_id: &ChannelId,
    ) -> Result<ModuleExtras, ChannelError> {
        Ok(ModuleExtras::empty())
    }

    // Note: no `on_recv_packet_validate()`
    // the `onRecvPacket` callback always succeeds
    // if any error occurs, than an "error acknowledgement"
    // must be returned

    fn on_recv_packet_execute(
        &mut self,
        packet: &Packet,
        relayer: &Signer,
    ) -> (ModuleExtras, Acknowledgement);

    fn on_acknowledgement_packet_validate(
        &self,
        _packet: &Packet,
        _acknowledgement: &Acknowledgement,
        _relayer: &Signer,
    ) -> Result<(), PacketError>;

    fn on_acknowledgement_packet_execute(
        &mut self,
        _packet: &Packet,
        _acknowledgement: &Acknowledgement,
        _relayer: &Signer,
    ) -> (ModuleExtras, Result<(), PacketError>);

    /// Note: `MsgTimeout` and `MsgTimeoutOnClose` use the same callback

    fn on_timeout_packet_validate(
        &self,
        packet: &Packet,
        relayer: &Signer,
    ) -> Result<(), PacketError>;

    /// Note: `MsgTimeout` and `MsgTimeoutOnClose` use the same callback

    fn on_timeout_packet_execute(
        &mut self,
        packet: &Packet,
        relayer: &Signer,
    ) -> (ModuleExtras, Result<(), PacketError>);
}
