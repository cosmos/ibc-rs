//! Provides a set of default on-module callback functions for the controller chain.
use ibc_core::channel::types::acknowledgement::{Acknowledgement, AcknowledgementStatus};
use ibc_core::channel::types::channel::{Counterparty, Order, State};
use ibc_core::channel::types::packet::Packet;
use ibc_core::channel::types::Version;
use ibc_core::host::types::identifiers::{ChannelId, ConnectionId, PortId};
use ibc_core::host::types::path::ChannelEndPath;
use ibc_core::primitives::Signer;
use ibc_core::router::types::module::ModuleExtras;

use crate::context::{InterchainAccountExecutionContext, InterchainAccountValidationContext};
use crate::error::InterchainAccountError;
use crate::metadata::Metadata;
use crate::port::{default_host_port_id, verify_controller_port_id_prefix};

/// Default validation callback function on the chan_open_init request for the
/// controller chain.
///
/// It performs basic validation for channel initialization when receiving a
/// request to register an interchain account.
pub fn on_chan_open_init_validate(
    ctx_a: &impl InterchainAccountValidationContext,
    order: Order,
    conn_hops_on_a: &[ConnectionId],
    port_id_on_a: &PortId,
    _chan_id_on_a: &ChannelId,
    counterparty: &Counterparty,
    version_on_a: &Version,
) -> Result<(), InterchainAccountError> {
    if !ctx_a.is_controller_enabled() {
        return Err(InterchainAccountError::not_supported(
            "controller chain is not enabled",
        ));
    }

    if order != Order::Ordered {
        return Err(InterchainAccountError::not_supported(
            "only ordered channels are supported",
        ));
    }

    verify_controller_port_id_prefix(port_id_on_a)?;

    let port_id_on_b = default_host_port_id()?;

    if counterparty.port_id != port_id_on_b {
        return Err(InterchainAccountError::mismatch("counterparty port id")
            .expected(&port_id_on_b)
            .given(&counterparty.port_id));
    }

    // Validates the provided version ending up with a correct metadata
    let metadata = if version_on_a.is_empty() {
        let conn_end_on_a = ctx_a.connection_end(&conn_hops_on_a[0])?;
        let conn_id_on_b = if let Some(id) = conn_end_on_a.counterparty().connection_id.clone() {
            id
        } else {
            return Err(InterchainAccountError::not_found(
                "counterparty connection id",
            ));
        };

        Metadata::new_default(conn_hops_on_a[0].clone(), conn_id_on_b)
    } else {
        serde_json::from_str::<Metadata>(version_on_a.as_str())
            .map_err(InterchainAccountError::source)?
    };

    metadata.validate(ctx_a, conn_hops_on_a)?;

    if let Ok(active_chan_id_on_a) = ctx_a.get_active_channel_id(&conn_hops_on_a[0], port_id_on_a) {
        let chan_end_on_a =
            ctx_a.channel_end(&ChannelEndPath::new(port_id_on_a, &active_chan_id_on_a))?;

        chan_end_on_a.verify_state_matches(&State::Open)?;

        metadata.verify_prev_metadata_matches(chan_end_on_a.version())?;
    }

    Ok(())
}

/// Default execution callback function on the chan_open_init request for the controller chain.
pub fn on_chan_open_init_execute(
    _ctx_a: &mut impl InterchainAccountExecutionContext,
    _order: Order,
    _conn_hops_on_a: &[ConnectionId],
    _port_id_on_a: PortId,
    _chan_id_on_a: ChannelId,
    _counterparty: Counterparty,
    version_on_a: Version,
) -> Result<(ModuleExtras, Version), InterchainAccountError> {
    Ok((ModuleExtras::empty(), version_on_a))
}

/// Default validation callback function on the chan_open_try request for the controller chain
///
/// Note: if your chain serves as both the controller and the host chain, you
/// may utilize the default
/// [host](crate::host::callback::on_chan_open_try_validate)
/// callback implementation instead.
pub fn on_chan_open_try_validate(
    _ctx_a: &impl InterchainAccountValidationContext,
    _order: Order,
    _conn_hops_on_a: &[ConnectionId],
    _port_id_on_a: &PortId,
    _chan_id_on_a: &ChannelId,
    _counterparty: &Counterparty,
    _version_on_b: &Version,
) -> Result<(), InterchainAccountError> {
    Err(InterchainAccountError::not_allowed(
        "channel handshake must be initiated by the controller chain",
    ))
}

/// Default execution callback function on the chan_open_try request for the controller chain
///
/// Note: if your chain serves as both the controller and the host chain, you
/// may utilize the default
/// [host](crate::host::callback::on_chan_open_try_execute)
/// callback implementation instead.
pub fn on_chan_open_try_execute(
    _ctx_a: &mut impl InterchainAccountExecutionContext,
    _order: Order,
    _conn_hops_on_a: &[ConnectionId],
    _port_id_on_a: &PortId,
    _chan_id_on_a: &ChannelId,
    _counterparty: &Counterparty,
    _version_on_b: &Version,
) -> Result<(ModuleExtras, Version), InterchainAccountError> {
    Err(InterchainAccountError::not_allowed(
        "channel handshake must be initiated by the controller chain",
    ))
}

/// Default validation callback function on the chan_open_ack request for the controller chain
pub fn on_chan_open_ack_validate(
    ctx_a: &impl InterchainAccountValidationContext,
    port_id_on_a: &PortId,
    chan_id_on_a: &ChannelId,
    version_on_b: &Version,
) -> Result<(), InterchainAccountError> {
    if !ctx_a.is_controller_enabled() {
        return Err(InterchainAccountError::not_supported(
            "controller chain is not enabled.",
        ));
    }

    let port_id_on_b = default_host_port_id()?;

    if port_id_on_a == &port_id_on_b {
        return Err(InterchainAccountError::invalid(
            "port id cannot be the same as host chain port id.",
        ));
    }

    verify_controller_port_id_prefix(port_id_on_a)?;

    let metadata = serde_json::from_str::<Metadata>(version_on_b.as_str())
        .map_err(InterchainAccountError::source)?;

    // Checks that no active channel exists for the given controller port identifier
    if ctx_a
        .get_active_channel_id(&metadata.conn_id_on_a, port_id_on_a)
        .is_ok()
    {
        return Err(InterchainAccountError::already_exists("active channel").given(&port_id_on_a));
    }

    let chan_end_on_a = ctx_a.channel_end(&ChannelEndPath::new(port_id_on_a, chan_id_on_a))?;

    chan_end_on_a.verify_state_matches(&State::Init)?;

    metadata.validate(ctx_a, chan_end_on_a.connection_hops())?;

    Ok(())
}

/// Default execution callback function on the chan_open_ack request for the controller chain
///
/// It sets the active channel for the interchain account/owner pair and stores
/// the associated interchain account address by it's corresponding port identifier.
pub fn on_chan_open_ack_execute(
    ctx_a: &mut impl InterchainAccountExecutionContext,
    port_id_on_a: PortId,
    chan_id_on_a: ChannelId,
    version_on_b: Version,
) -> Result<ModuleExtras, InterchainAccountError> {
    let metadata = serde_json::from_str::<Metadata>(version_on_b.as_str())
        .map_err(InterchainAccountError::source)?;

    ctx_a.store_active_channel_id(
        metadata.conn_id_on_a.clone(),
        port_id_on_a.clone(),
        chan_id_on_a,
    )?;
    ctx_a.store_ica_address(metadata.conn_id_on_a, port_id_on_a, metadata.address)?;

    Ok(ModuleExtras::empty())
}

/// Default validation callback function on the chan_open_confirm request for the controller chain
///
/// Note: if your chain serves as both the controller and the host chain, you
/// may utilize the default
/// [host](crate::host::callback::on_chan_open_confirm_validate)
/// callback implementation instead.
pub fn on_chan_open_confirm_validate(
    _ctx_a: &impl InterchainAccountValidationContext,
    _port_id_on_a: &PortId,
    _chan_id_on_a: &ChannelId,
) -> Result<(), InterchainAccountError> {
    Err(InterchainAccountError::not_allowed(
        "channel handshake must be initiated by the controller chain",
    ))
}

/// Default execution callback function on the chan_open_confirm request for the controller chain
///
/// Note: if your chain serves as both the controller and the host chain, you
/// may utilize the default
/// [host](crate::host::callback::on_chan_open_confirm_execute)
/// callback implementation instead.
pub fn on_chan_open_confirm_execute(
    _ctx_a: &mut impl InterchainAccountExecutionContext,
    _port_id_on_a: PortId,
    _chan_id_on_a: ChannelId,
) -> Result<ModuleExtras, InterchainAccountError> {
    Err(InterchainAccountError::not_allowed(
        "channel handshake must be initiated by the controller chain",
    ))
}

/// Default validation callback function on the chan_close_init request for the controller chain
pub fn on_chan_close_init_validate(
    _ctx_a: &impl InterchainAccountValidationContext,
    _port_id_on_a: &PortId,
    _chan_id_on_a: &ChannelId,
) -> Result<(), InterchainAccountError> {
    Err(InterchainAccountError::invalid("channel cannot be closed."))
}

/// Default execution callback function on the chan_close_init request for the controller chain
pub fn on_chan_close_init_execute(
    _ctx_a: &mut impl InterchainAccountExecutionContext,
    _port_id_on_a: PortId,
    _chan_id_on_a: ChannelId,
) -> Result<ModuleExtras, InterchainAccountError> {
    Err(InterchainAccountError::invalid("channel cannot be closed."))
}

/// Default validation callback function on the recv_packet request for the controller chain
///
/// Note: if your chain serves as both the controller and the host chain, you
/// may utilize the default
/// [host](crate::host::callback::on_recv_packet_execute)
/// callback implementation instead.
pub fn on_recv_packet_execute(
    _ctx_a: &mut impl InterchainAccountExecutionContext,
    _packet: Packet,
) -> (ModuleExtras, Acknowledgement) {
    (
        ModuleExtras::empty(),
        AcknowledgementStatus::error(
            InterchainAccountError::not_allowed("packet cannot be received").into(),
        )
        .into(),
    )
}

/// Default validation callback function on the acknowledgment_packet request for the controller chain
pub fn on_acknowledgement_packet_validate<Ctx>(
    _ctx_a: &impl InterchainAccountValidationContext,
    _packet: &Packet,
    _ack_on_a: &Acknowledgement,
    _relayer: &Signer,
) -> Result<(), InterchainAccountError> {
    Ok(())
}

/// Default execution callback function on the acknowledgment_packet request for the controller chain
pub fn on_acknowledgement_packet_execute(
    _ctx_a: &mut impl InterchainAccountExecutionContext,
    _packet: Packet,
    _ack_on_a: Acknowledgement,
    _relayer: Signer,
) -> (ModuleExtras, Result<(), InterchainAccountError>) {
    (ModuleExtras::empty(), Ok(()))
}

/// Default validation callback function on the timeout_packet request for the controller chain
pub fn on_timeout_packet_validate<Ctx>(
    _ctx_a: &impl InterchainAccountValidationContext,
    _packet: &Packet,
    _relayer: &Signer,
) -> Result<(), InterchainAccountError> {
    Ok(())
}

/// Default execution callback function on the timeout_packet request for the controller chain
pub fn on_timeout_packet_execute(
    _ctx_a: &mut impl InterchainAccountExecutionContext,
    _packet: Packet,
    _relayer: Signer,
) -> (ModuleExtras, Result<(), InterchainAccountError>) {
    (ModuleExtras::empty(), Ok(()))
}
