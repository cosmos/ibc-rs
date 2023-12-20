//! Provides IBC module callbacks implementation for the ICS-721 transfer.

use ibc_app_nft_transfer_types::error::NftTransferError;
use ibc_core::channel::types::acknowledgement::Acknowledgement;
use ibc_core::channel::types::channel::{Counterparty, Order};
use ibc_core::channel::types::packet::Packet;
use ibc_core::channel::types::Version;
use ibc_core::host::types::identifiers::{ChannelId, ConnectionId, PortId};
use ibc_core::primitives::Signer;
use ibc_core::router::types::module::ModuleExtras;

use crate::context::{NftTransferExecutionContext, NftTransferValidationContext};

pub fn on_chan_open_init_validate(
    _ctx: &impl NftTransferValidationContext,
    _order: Order,
    _connection_hops: &[ConnectionId],
    _port_id: &PortId,
    _channel_id: &ChannelId,
    _counterparty: &Counterparty,
    _version: &Version,
) -> Result<(), NftTransferError> {
    unimplemented!()
}

pub fn on_chan_open_init_execute(
    _ctx: &mut impl NftTransferExecutionContext,
    _order: Order,
    _connection_hops: &[ConnectionId],
    _port_id: &PortId,
    _channel_id: &ChannelId,
    _counterparty: &Counterparty,
    _version: &Version,
) -> Result<(ModuleExtras, Version), NftTransferError> {
    unimplemented!()
}

pub fn on_chan_open_try_validate(
    _ctx: &impl NftTransferValidationContext,
    _order: Order,
    _connection_hops: &[ConnectionId],
    _port_id: &PortId,
    _channel_id: &ChannelId,
    _counterparty: &Counterparty,
    _counterparty_version: &Version,
) -> Result<(), NftTransferError> {
    unimplemented!()
}

pub fn on_chan_open_try_execute(
    _ctx: &mut impl NftTransferExecutionContext,
    _order: Order,
    _connection_hops: &[ConnectionId],
    _port_id: &PortId,
    _channel_id: &ChannelId,
    _counterparty: &Counterparty,
    _counterparty_version: &Version,
) -> Result<(ModuleExtras, Version), NftTransferError> {
    unimplemented!()
}

pub fn on_chan_open_ack_validate(
    _ctx: &impl NftTransferExecutionContext,
    _port_id: &PortId,
    _channel_id: &ChannelId,
    _counterparty_version: &Version,
) -> Result<(), NftTransferError> {
    unimplemented!()
}

pub fn on_chan_open_ack_execute(
    _ctx: &mut impl NftTransferExecutionContext,
    _port_id: &PortId,
    _channel_id: &ChannelId,
    _counterparty_version: &Version,
) -> Result<ModuleExtras, NftTransferError> {
    unimplemented!()
}

pub fn on_chan_open_confirm_validate(
    _ctx: &impl NftTransferValidationContext,
    _port_id: &PortId,
    _channel_id: &ChannelId,
) -> Result<(), NftTransferError> {
    unimplemented!()
}

pub fn on_chan_open_confirm_execute(
    _ctx: &mut impl NftTransferExecutionContext,
    _port_id: &PortId,
    _channel_id: &ChannelId,
) -> Result<ModuleExtras, NftTransferError> {
    unimplemented!()
}

pub fn on_chan_close_init_validate(
    _ctx: &impl NftTransferValidationContext,
    _port_id: &PortId,
    _channel_id: &ChannelId,
) -> Result<(), NftTransferError> {
    unimplemented!()
}

pub fn on_chan_close_init_execute(
    _ctx: &mut impl NftTransferExecutionContext,
    _port_id: &PortId,
    _channel_id: &ChannelId,
) -> Result<ModuleExtras, NftTransferError> {
    unimplemented!()
}

pub fn on_chan_close_confirm_validate(
    _ctx: &impl NftTransferValidationContext,
    _port_id: &PortId,
    _channel_id: &ChannelId,
) -> Result<(), NftTransferError> {
    unimplemented!()
}

pub fn on_chan_close_confirm_execute(
    _ctx: &mut impl NftTransferExecutionContext,
    _port_id: &PortId,
    _channel_id: &ChannelId,
) -> Result<ModuleExtras, NftTransferError> {
    unimplemented!()
}

pub fn on_recv_packet_execute(
    _ctx_b: &mut impl NftTransferExecutionContext,
    _packet: &Packet,
) -> (ModuleExtras, Acknowledgement) {
    unimplemented!()
}

pub fn on_acknowledgement_packet_validate<Ctx>(
    _ctx: &impl NftTransferValidationContext,
    _packet: &Packet,
    _acknowledgement: &Acknowledgement,
    _relayer: &Signer,
) -> Result<(), NftTransferError> {
    unimplemented!()
}

pub fn on_acknowledgement_packet_execute(
    _ctx: &mut impl NftTransferExecutionContext,
    _packet: &Packet,
    _acknowledgement: &Acknowledgement,
    _relayer: &Signer,
) -> (ModuleExtras, Result<(), NftTransferError>) {
    unimplemented!()
}

pub fn on_timeout_packet_validate(
    _ctx: &impl NftTransferValidationContext,
    _packet: &Packet,
    _relayer: &Signer,
) -> Result<(), NftTransferError> {
    unimplemented!()
}

pub fn on_timeout_packet_execute(
    _ctx: &mut impl NftTransferExecutionContext,
    _packet: &Packet,
    _relayer: &Signer,
) -> (ModuleExtras, Result<(), NftTransferError>) {
    unimplemented!()
}
