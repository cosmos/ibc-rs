/// The trait that defines an IBC application
use core::fmt::Debug;

use ibc_eureka_core_channel_types::acknowledgement::Acknowledgement;
use ibc_eureka_core_channel_types::error::ChannelError;
use ibc_eureka_core_channel_types::packet::Packet;
use ibc_eureka_core_channel_types::Version;
use ibc_eureka_core_host_types::identifiers::{ChannelId, PortId};
use ibc_eureka_core_router_types::module::ModuleExtras;
use ibc_primitives::prelude::*;
use ibc_primitives::Signer;

pub trait Module: Debug {
    fn on_chan_open_init_validate(
        &self,
        port_id: &PortId,
        channel_id: &ChannelId,
        version: &Version,
    ) -> Result<Version, ChannelError>;

    fn on_chan_open_init_execute(
        &mut self,
        port_id: &PortId,
        channel_id: &ChannelId,
        version: &Version,
    ) -> Result<(ModuleExtras, Version), ChannelError>;

    fn on_chan_open_try_validate(
        &self,
        port_id: &PortId,
        channel_id: &ChannelId,
        counterparty_version: &Version,
    ) -> Result<Version, ChannelError>;

    fn on_chan_open_try_execute(
        &mut self,
        port_id: &PortId,
        channel_id: &ChannelId,
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
    ) -> Result<(), ChannelError>;

    fn on_acknowledgement_packet_execute(
        &mut self,
        _packet: &Packet,
        _acknowledgement: &Acknowledgement,
        _relayer: &Signer,
    ) -> (ModuleExtras, Result<(), ChannelError>);

    /// Note: `MsgTimeout` and `MsgTimeoutOnClose` use the same callback

    fn on_timeout_packet_validate(
        &self,
        packet: &Packet,
        relayer: &Signer,
    ) -> Result<(), ChannelError>;

    /// Note: `MsgTimeout` and `MsgTimeoutOnClose` use the same callback

    fn on_timeout_packet_execute(
        &mut self,
        packet: &Packet,
        relayer: &Signer,
    ) -> (ModuleExtras, Result<(), ChannelError>);
}