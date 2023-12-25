//! Provides IBC module callbacks implementation for the ICS-721 transfer.

use ibc_app_nft_transfer_types::error::NftTransferError;
use ibc_app_nft_transfer_types::VERSION;
use ibc_core::channel::types::acknowledgement::Acknowledgement;
use ibc_core::channel::types::channel::{Counterparty, Order};
use ibc_core::channel::types::packet::Packet;
use ibc_core::channel::types::Version;
use ibc_core::handler::types::error::ContextError;
use ibc_core::host::types::identifiers::{ChannelId, ConnectionId, PortId};
use ibc_core::primitives::prelude::*;
use ibc_core::primitives::Signer;
use ibc_core::router::types::module::ModuleExtras;

use crate::context::{NftTransferExecutionContext, NftTransferValidationContext};

pub fn on_chan_open_init_validate<N, C>(
    ctx: &impl NftTransferValidationContext<N, C>,
    order: Order,
    _connection_hops: &[ConnectionId],
    port_id: &PortId,
    _channel_id: &ChannelId,
    _counterparty: &Counterparty,
    version: &Version,
) -> Result<(), NftTransferError> {
    if order != Order::Unordered {
        return Err(NftTransferError::ChannelNotUnordered {
            expect_order: Order::Unordered,
            got_order: order,
        });
    }
    let bound_port = ctx.get_port()?;
    if port_id != &bound_port {
        return Err(NftTransferError::InvalidPort {
            port_id: port_id.clone(),
            exp_port_id: bound_port,
        });
    }

    if !version.is_empty() {
        version
            .verify_is_expected(Version::new(VERSION.to_string()))
            .map_err(ContextError::from)?;
    }

    Ok(())
}

pub fn on_chan_open_init_execute<N, C>(
    _ctx: &mut impl NftTransferExecutionContext<N, C>,
    _order: Order,
    _connection_hops: &[ConnectionId],
    _port_id: &PortId,
    _channel_id: &ChannelId,
    _counterparty: &Counterparty,
    _version: &Version,
) -> Result<(ModuleExtras, Version), NftTransferError> {
    Ok((ModuleExtras::empty(), Version::new(VERSION.to_string())))
}

pub fn on_chan_open_try_validate<N, C>(
    _ctx: &impl NftTransferValidationContext<N, C>,
    order: Order,
    _connection_hops: &[ConnectionId],
    _port_id: &PortId,
    _channel_id: &ChannelId,
    _counterparty: &Counterparty,
    counterparty_version: &Version,
) -> Result<(), NftTransferError> {
    if order != Order::Unordered {
        return Err(NftTransferError::ChannelNotUnordered {
            expect_order: Order::Unordered,
            got_order: order,
        });
    }

    counterparty_version
        .verify_is_expected(Version::new(VERSION.to_string()))
        .map_err(ContextError::from)?;

    Ok(())
}

pub fn on_chan_open_try_execute<N, C>(
    _ctx: &mut impl NftTransferExecutionContext<N, C>,
    _order: Order,
    _connection_hops: &[ConnectionId],
    _port_id: &PortId,
    _channel_id: &ChannelId,
    _counterparty: &Counterparty,
    _counterparty_version: &Version,
) -> Result<(ModuleExtras, Version), NftTransferError> {
    Ok((ModuleExtras::empty(), Version::new(VERSION.to_string())))
}

pub fn on_chan_open_ack_validate<N, C>(
    _ctx: &impl NftTransferExecutionContext<N, C>,
    _port_id: &PortId,
    _channel_id: &ChannelId,
    counterparty_version: &Version,
) -> Result<(), NftTransferError> {
    counterparty_version
        .verify_is_expected(Version::new(VERSION.to_string()))
        .map_err(ContextError::from)?;

    Ok(())
}

pub fn on_chan_open_ack_execute<N, C>(
    _ctx: &mut impl NftTransferExecutionContext<N, C>,
    _port_id: &PortId,
    _channel_id: &ChannelId,
    _counterparty_version: &Version,
) -> Result<ModuleExtras, NftTransferError> {
    Ok(ModuleExtras::empty())
}

pub fn on_chan_open_confirm_validate<N, C>(
    _ctx: &impl NftTransferValidationContext<N, C>,
    _port_id: &PortId,
    _channel_id: &ChannelId,
) -> Result<(), NftTransferError> {
    Ok(())
}

pub fn on_chan_open_confirm_execute<N, C>(
    _ctx: &mut impl NftTransferExecutionContext<N, C>,
    _port_id: &PortId,
    _channel_id: &ChannelId,
) -> Result<ModuleExtras, NftTransferError> {
    Ok(ModuleExtras::empty())
}

pub fn on_chan_close_init_validate<N, C>(
    _ctx: &impl NftTransferValidationContext<N, C>,
    _port_id: &PortId,
    _channel_id: &ChannelId,
) -> Result<(), NftTransferError> {
    Err(NftTransferError::CantCloseChannel)
}

pub fn on_chan_close_init_execute<N, C>(
    _ctx: &mut impl NftTransferExecutionContext<N, C>,
    _port_id: &PortId,
    _channel_id: &ChannelId,
) -> Result<ModuleExtras, NftTransferError> {
    unimplemented!()
}

pub fn on_chan_close_confirm_validate<N, C>(
    _ctx: &impl NftTransferValidationContext<N, C>,
    _port_id: &PortId,
    _channel_id: &ChannelId,
) -> Result<(), NftTransferError> {
    Ok(())
}

pub fn on_chan_close_confirm_execute<N, C>(
    _ctx: &mut impl NftTransferExecutionContext<N, C>,
    _port_id: &PortId,
    _channel_id: &ChannelId,
) -> Result<ModuleExtras, NftTransferError> {
    Ok(ModuleExtras::empty())
}

pub fn on_recv_packet_execute<N, C>(
    _ctx_b: &mut impl NftTransferExecutionContext<N, C>,
    _packet: &Packet,
) -> (ModuleExtras, Acknowledgement) {
    unimplemented!()
}

pub fn on_acknowledgement_packet_validate<N, C>(
    _ctx: &impl NftTransferValidationContext<N, C>,
    _packet: &Packet,
    _acknowledgement: &Acknowledgement,
    _relayer: &Signer,
) -> Result<(), NftTransferError> {
    unimplemented!()
}

pub fn on_acknowledgement_packet_execute<N, C>(
    _ctx: &mut impl NftTransferExecutionContext<N, C>,
    _packet: &Packet,
    _acknowledgement: &Acknowledgement,
    _relayer: &Signer,
) -> (ModuleExtras, Result<(), NftTransferError>) {
    unimplemented!()
}

pub fn on_timeout_packet_validate<N, C>(
    _ctx: &impl NftTransferValidationContext<N, C>,
    _packet: &Packet,
    _relayer: &Signer,
) -> Result<(), NftTransferError> {
    unimplemented!()
}

pub fn on_timeout_packet_execute<N, C>(
    _ctx: &mut impl NftTransferExecutionContext<N, C>,
    _packet: &Packet,
    _relayer: &Signer,
) -> (ModuleExtras, Result<(), NftTransferError>) {
    unimplemented!()
}
