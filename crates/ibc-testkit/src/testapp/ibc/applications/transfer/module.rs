use ibc::core::ics04_channel::acknowledgement::Acknowledgement;
use ibc::core::ics04_channel::channel::{Counterparty, Order};
use ibc::core::ics04_channel::error::{ChannelError, PacketError};
use ibc::core::ics04_channel::packet::Packet;
use ibc::core::ics04_channel::Version;
use ibc::core::ics24_host::identifier::{ChannelId, ConnectionId, PortId};
use ibc::core::router::{Module, ModuleExtras};
use ibc::prelude::*;
use ibc::Signer;

use super::types::DummyTransferModule;

impl Module for DummyTransferModule {
    fn on_chan_open_init_validate(
        &self,
        _order: Order,
        _connection_hops: &[ConnectionId],
        _port_id: &PortId,
        _channel_id: &ChannelId,
        _counterparty: &Counterparty,
        version: &Version,
    ) -> Result<Version, ChannelError> {
        Ok(version.clone())
    }

    fn on_chan_open_init_execute(
        &mut self,
        _order: Order,
        _connection_hops: &[ConnectionId],
        _port_id: &PortId,
        _channel_id: &ChannelId,
        _counterparty: &Counterparty,
        version: &Version,
    ) -> Result<(ModuleExtras, Version), ChannelError> {
        Ok((ModuleExtras::empty(), version.clone()))
    }

    fn on_chan_open_try_validate(
        &self,
        _order: Order,
        _connection_hops: &[ConnectionId],
        _port_id: &PortId,
        _channel_id: &ChannelId,
        _counterparty: &Counterparty,
        counterparty_version: &Version,
    ) -> Result<Version, ChannelError> {
        Ok(counterparty_version.clone())
    }

    fn on_chan_open_try_execute(
        &mut self,
        _order: Order,
        _connection_hops: &[ConnectionId],
        _port_id: &PortId,
        _channel_id: &ChannelId,
        _counterparty: &Counterparty,
        counterparty_version: &Version,
    ) -> Result<(ModuleExtras, Version), ChannelError> {
        Ok((ModuleExtras::empty(), counterparty_version.clone()))
    }

    fn on_recv_packet_execute(
        &mut self,
        _packet: &Packet,
        _relayer: &Signer,
    ) -> (ModuleExtras, Acknowledgement) {
        (
            ModuleExtras::empty(),
            Acknowledgement::try_from(vec![1u8]).expect("Never fails"),
        )
    }

    fn on_timeout_packet_validate(
        &self,
        _packet: &Packet,
        _relayer: &Signer,
    ) -> Result<(), PacketError> {
        Ok(())
    }

    fn on_timeout_packet_execute(
        &mut self,
        _packet: &Packet,
        _relayer: &Signer,
    ) -> (ModuleExtras, Result<(), PacketError>) {
        (ModuleExtras::empty(), Ok(()))
    }

    fn on_acknowledgement_packet_validate(
        &self,
        _packet: &Packet,
        _acknowledgement: &Acknowledgement,
        _relayer: &Signer,
    ) -> Result<(), PacketError> {
        Ok(())
    }

    fn on_acknowledgement_packet_execute(
        &mut self,
        _packet: &Packet,
        _acknowledgement: &Acknowledgement,
        _relayer: &Signer,
    ) -> (ModuleExtras, Result<(), PacketError>) {
        (ModuleExtras::empty(), Ok(()))
    }
}
