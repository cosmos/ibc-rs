use crate::core::ics05_port::error::PortError;
use crate::core::{ics04_channel::handler::ModuleExtras, ics24_host::path::PortPath};
use crate::prelude::*;

use alloc::borrow::{Borrow, Cow};
use core::{
    fmt::{Debug, Display, Error as FmtError, Formatter},
    str::FromStr,
};

use crate::core::ics04_channel::channel::{Counterparty, Order};
use crate::core::ics04_channel::error::{ChannelError, PacketError};
use crate::core::ics04_channel::msgs::acknowledgement::Acknowledgement;
use crate::core::ics04_channel::packet::Packet;
use crate::core::ics04_channel::Version;
use crate::core::ics24_host::identifier::{ChannelId, ConnectionId, PortId};
use crate::signer::Signer;

#[derive(Debug, PartialEq, Eq)]
pub struct InvalidModuleId;

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
    pub fn new(s: Cow<'_, str>) -> Result<Self, InvalidModuleId> {
        if !s.trim().is_empty() && s.chars().all(char::is_alphanumeric) {
            Ok(Self(s.into_owned()))
        } else {
            Err(InvalidModuleId)
        }
    }
}

impl Display for ModuleId {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        write!(f, "{}", self.0)
    }
}

impl FromStr for ModuleId {
    type Err = InvalidModuleId;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(Cow::Borrowed(s))
    }
}

impl Borrow<str> for ModuleId {
    fn borrow(&self) -> &str {
        self.0.as_str()
    }
}

pub trait ModuleContext {
    /// Returns the identifier of the module.
    fn module_id(&self) -> ModuleId;

    /// Returns the owned ports by the module.
    fn get_owned_ports(&self) -> Vec<PortId>;

    /// Stores an identifier of the module at a path associated with the port_id and bind the port.
    fn bind_port(&mut self, port_path: PortPath, port_owner: ModuleId) -> Result<(), PortError>;

    /// Deletes the identifier of the module at a path associated with the port_id and unbind the port.
    fn release_port(&mut self, port_path: PortPath) -> Result<(), PortError>;
}

pub trait Module: ValidationModule + ExecutionModule {
    fn as_validation_module(&self) -> &dyn ValidationModule;
    fn as_execution_module(&mut self) -> &mut dyn ExecutionModule;
}

impl<M> Module for M
where
    M: ValidationModule + ExecutionModule,
{
    fn as_validation_module(&self) -> &dyn ValidationModule {
        self
    }
    fn as_execution_module(&mut self) -> &mut dyn ExecutionModule {
        self
    }
}

pub trait ValidationModule: ModuleContext + Debug {
    fn on_chan_open_init_validate(
        &self,
        order: Order,
        connection_hops: &[ConnectionId],
        port_id: &PortId,
        channel_id: &ChannelId,
        counterparty: &Counterparty,
        version: &Version,
    ) -> Result<Version, ChannelError>;

    fn on_chan_open_try_validate(
        &self,
        order: Order,
        connection_hops: &[ConnectionId],
        port_id: &PortId,
        channel_id: &ChannelId,
        counterparty: &Counterparty,
        counterparty_version: &Version,
    ) -> Result<Version, ChannelError>;

    fn on_chan_open_ack_validate(
        &self,
        _port_id: &PortId,
        _channel_id: &ChannelId,
        _counterparty_version: &Version,
    ) -> Result<(), ChannelError> {
        Ok(())
    }

    fn on_chan_open_confirm_validate(
        &self,
        _port_id: &PortId,
        _channel_id: &ChannelId,
    ) -> Result<(), ChannelError> {
        Ok(())
    }

    fn on_chan_close_init_validate(
        &self,
        _port_id: &PortId,
        _channel_id: &ChannelId,
    ) -> Result<(), ChannelError> {
        Ok(())
    }

    fn on_chan_close_confirm_validate(
        &self,
        _port_id: &PortId,
        _channel_id: &ChannelId,
    ) -> Result<(), ChannelError> {
        Ok(())
    }

    // Note: no `on_recv_packet_validate()`
    // the `onRecvPacket` callback always succeeds
    // if any error occurs, than an "error acknowledgement"
    // must be returned

    fn on_acknowledgement_packet_validate(
        &self,
        _packet: &Packet,
        _acknowledgement: &Acknowledgement,
        _relayer: &Signer,
    ) -> Result<(), PacketError>;

    /// Note: `MsgTimeout` and `MsgTimeoutOnClose` use the same callback

    fn on_timeout_packet_validate(
        &self,
        packet: &Packet,
        relayer: &Signer,
    ) -> Result<(), PacketError>;
}

pub trait ExecutionModule: ModuleContext + Debug {
    fn on_chan_open_init_execute(
        &mut self,
        order: Order,
        connection_hops: &[ConnectionId],
        port_id: &PortId,
        channel_id: &ChannelId,
        counterparty: &Counterparty,
        version: &Version,
    ) -> Result<(ModuleExtras, Version), ChannelError>;

    fn on_chan_open_try_execute(
        &mut self,
        order: Order,
        connection_hops: &[ConnectionId],
        port_id: &PortId,
        channel_id: &ChannelId,
        counterparty: &Counterparty,
        counterparty_version: &Version,
    ) -> Result<(ModuleExtras, Version), ChannelError>;

    fn on_chan_open_ack_execute(
        &mut self,
        _port_id: &PortId,
        _channel_id: &ChannelId,
        _counterparty_version: &Version,
    ) -> Result<ModuleExtras, ChannelError> {
        Ok(ModuleExtras::empty())
    }

    fn on_chan_open_confirm_execute(
        &mut self,
        _port_id: &PortId,
        _channel_id: &ChannelId,
    ) -> Result<ModuleExtras, ChannelError> {
        Ok(ModuleExtras::empty())
    }

    fn on_chan_close_init_execute(
        &mut self,
        _port_id: &PortId,
        _channel_id: &ChannelId,
    ) -> Result<ModuleExtras, ChannelError> {
        Ok(ModuleExtras::empty())
    }

    fn on_chan_close_confirm_execute(
        &mut self,
        _port_id: &PortId,
        _channel_id: &ChannelId,
    ) -> Result<ModuleExtras, ChannelError> {
        Ok(ModuleExtras::empty())
    }

    fn on_recv_packet_execute(
        &mut self,
        packet: &Packet,
        relayer: &Signer,
    ) -> (ModuleExtras, Acknowledgement);

    fn on_acknowledgement_packet_execute(
        &mut self,
        _packet: &Packet,
        _acknowledgement: &Acknowledgement,
        _relayer: &Signer,
    ) -> (ModuleExtras, Result<(), PacketError>);

    /// Note: `MsgTimeout` and `MsgTimeoutOnClose` use the same callback

    fn on_timeout_packet_execute(
        &mut self,
        packet: &Packet,
        relayer: &Signer,
    ) -> (ModuleExtras, Result<(), PacketError>);
}
