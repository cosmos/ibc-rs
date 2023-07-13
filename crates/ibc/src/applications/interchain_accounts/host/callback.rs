//! Provides a set of default on-module callback functions for the host chain.

use alloc::string::ToString;

use crate::applications::interchain_accounts::ack_success;
use crate::applications::interchain_accounts::context::InterchainAccountExecutionContext;
use crate::applications::interchain_accounts::context::InterchainAccountValidationContext;
use crate::applications::interchain_accounts::error::InterchainAccountError;
use crate::applications::interchain_accounts::metadata::Metadata;
use crate::applications::interchain_accounts::port::default_host_port_id;
use crate::core::ics04_channel::acknowledgement::Acknowledgement;
use crate::core::ics04_channel::acknowledgement::AcknowledgementStatus;
use crate::core::ics04_channel::channel::Counterparty;
use crate::core::ics04_channel::channel::Order;
use crate::core::ics04_channel::packet::Packet;
use crate::core::ics04_channel::Version;
use crate::core::ics24_host::identifier::ChannelId;
use crate::core::ics24_host::identifier::ConnectionId;
use crate::core::ics24_host::identifier::PortId;
use crate::core::ics24_host::path::ChannelEndPath;
use crate::core::router::ModuleExtras;
use crate::Signer;

use super::handler::create_interchain_account::create_interchain_account;
use super::handler::on_recv_packet::on_recv_packet;

/// Default validation callback function on the chan_open_init request for the host chain
///
/// Note: if your chain serves as both the controller and the host chain, you
/// may utilize the default
/// [controller](crate::applications::interchain_accounts::controller::callback::on_chan_open_init_validate)
/// callback implementation instead.
pub fn on_chan_open_init_validate(
    _ctx_b: &impl InterchainAccountValidationContext,
    _order: Order,
    _conn_hops_on_b: &[ConnectionId],
    _port_id_on_b: &PortId,
    _chan_id_on_b: &ChannelId,
    _counterparty: &Counterparty,
    _version_on_b: &Version,
) -> Result<(), InterchainAccountError> {
    Err(InterchainAccountError::not_allowed(
        "channel handshake must be initiated by the controller chain",
    ))
}

/// Default execution callback function on the chan_open_init request for the host chain
///
/// Note: if your chain serves as both the controller and the host chain, you
/// may utilize the default
/// [controller](crate::applications::interchain_accounts::controller::callback::on_chan_open_init_execute)
/// callback implementation instead.
pub fn on_chan_open_init_execute(
    _ctx_b: &mut impl InterchainAccountExecutionContext,
    _order: Order,
    _conn_hops_on_b: &[ConnectionId],
    _port_id_on_b: PortId,
    _chan_id_on_b: ChannelId,
    _counterparty: Counterparty,
    _version_on_b: &Version,
) -> Result<(ModuleExtras, Version), InterchainAccountError> {
    Err(InterchainAccountError::not_allowed(
        "channel handshake must be initiated by the controller chain",
    ))
}
/// Default validation callback function on the chan_open_try request for the host chain
pub fn on_chan_open_try_validate(
    ctx_b: &impl InterchainAccountValidationContext,
    order: Order,
    conn_hops_on_b: &[ConnectionId],
    port_id_on_b: &PortId,
    chan_id_on_b: &ChannelId,
    counterparty: &Counterparty,
    version_on_a: &Version,
) -> Result<(), InterchainAccountError> {
    let params = ctx_b.get_params()?;

    if !params.host_enabled {
        return Err(InterchainAccountError::not_supported(
            "host chain is not enabled.",
        ));
    }

    if order != Order::Ordered {
        return Err(InterchainAccountError::not_supported(
            "only ordered channels are supported.",
        ));
    }
    let host_port_id = default_host_port_id()?;

    if port_id_on_b != &host_port_id {
        return Err(
            InterchainAccountError::mismatch("port id must be the default host port id.")
                .given(port_id_on_b)
                .expected(&host_port_id),
        );
    }

    let metadata = serde_json::from_str::<Metadata>(version_on_a.as_str())
        .map_err(InterchainAccountError::source)?;

    metadata.validate(ctx_b, conn_hops_on_b)?;

    if let Ok(active_channel_id) =
        ctx_b.get_active_channel_id(&conn_hops_on_b[0], counterparty.port_id())
    {
        if &active_channel_id != chan_id_on_b {
            return Err(
                InterchainAccountError::mismatch("active channel id mismatch")
                    .given(chan_id_on_b)
                    .expected(&active_channel_id),
            );
        }

        let chan_end_path = ChannelEndPath::new(port_id_on_b, &active_channel_id);

        let chan_end_on_b = ctx_b.channel_end(&chan_end_path)?;

        if chan_end_on_b.state().is_open() {
            return Err(InterchainAccountError::invalid("channel is already open."));
        }

        metadata.verify_prev_metadata_matches(chan_end_on_b.version())?;
    }

    Ok(())
}

/// Default execution callback function on the chan_open_try request for the host chain
pub fn on_chan_open_try_execute(
    ctx_b: &mut impl InterchainAccountExecutionContext,
    _order: Order,
    conn_hops_on_b: &[ConnectionId],
    port_id_on_b: &PortId,
    _chan_id_on_b: &ChannelId,
    counterparty: &Counterparty,
    version_on_a: &Version,
) -> Result<(ModuleExtras, Version), InterchainAccountError> {
    let ica_address = if let Ok(ica_account) =
        ctx_b.get_ica_address(&conn_hops_on_b[0], counterparty.port_id())
    {
        ctx_b.validate_message_signer(&ica_account)?;

        ctx_b.log_message("reopening existing interchain account".to_string());

        ctx_b.get_interchain_account(&ica_account)?;

        ica_account
    } else {
        create_interchain_account(ctx_b, conn_hops_on_b[0].clone(), port_id_on_b.clone())?
    };

    ctx_b.log_message("interchain account created".to_string());

    let mut metadata = serde_json::from_str::<Metadata>(version_on_a.as_str())
        .map_err(InterchainAccountError::source)?;

    metadata.address = ica_address;

    let metadata_str = serde_json::to_string(&metadata).map_err(InterchainAccountError::source)?;

    Ok((ModuleExtras::empty(), Version::new(metadata_str)))
}

/// Default validation callback function on the chan_open_ack request for the host chain
///
/// Note: if your chain serves as both the controller and the host chain, you
/// may utilize the default
/// [controller](crate::applications::interchain_accounts::controller::callback::on_chan_open_ack_validate)
/// callback implementation instead.
pub fn on_chan_open_ack_validate(
    _ctx_b: &impl InterchainAccountValidationContext,
    _port_id_on_b: &PortId,
    _chan_id_on_b: &ChannelId,
    _version_on_a: &Version,
) -> Result<(), InterchainAccountError> {
    Err(InterchainAccountError::not_allowed(
        "channel handshake must be initiated by the controller chain",
    ))
}

/// Default execution callback function on the chan_open_ack request for the host chain
///
/// Note: if your chain serves as both the controller and the host chain, you
/// may utilize the default
/// [controller](crate::applications::interchain_accounts::controller::callback::on_chan_open_ack_execute)
/// callback implementation instead.
pub fn on_chan_open_ack_execute(
    _ctx_b: &mut impl InterchainAccountExecutionContext,
    _port_id_on_b: PortId,
    _chan_id_on_b: ChannelId,
    _version_on_a: Version,
) -> Result<ModuleExtras, InterchainAccountError> {
    Err(InterchainAccountError::not_allowed(
        "channel handshake must be initiated by the controller chain",
    ))
}

/// Default validation callback function on the chan_open_confirm request for the host chain
pub fn on_chan_open_confirm_validate(
    ctx_b: &impl InterchainAccountValidationContext,
    port_id_on_b: &PortId,
    chan_id_on_b: &ChannelId,
) -> Result<(), InterchainAccountError> {
    let params = ctx_b.get_params()?;

    if !params.host_enabled {
        return Err(InterchainAccountError::not_supported(
            "host chain is not enabled.",
        ));
    }

    // It is assumed the controller chain will not allow multiple active channels to be created for the same ConnectionId/PortId
    // If the controller chain does allow multiple active channels to be created for the same ConnectionId/PortId,
    // disallowing overwriting the current active channel guarantees the channel can no longer be used as the controller
    // and host will disagree on what the currently active channel is
    ctx_b.channel_end(&ChannelEndPath::new(port_id_on_b, chan_id_on_b))?;
    Ok(())
}

/// Default execution callback function on the chan_open_confirm request for the host chain
pub fn on_chan_open_confirm_execute(
    ctx_b: &mut impl InterchainAccountExecutionContext,
    port_id_on_b: PortId,
    chan_id_on_b: ChannelId,
) -> Result<ModuleExtras, InterchainAccountError> {
    let chan_end = ctx_b.channel_end(&ChannelEndPath::new(&port_id_on_b, &chan_id_on_b))?;

    ctx_b.store_active_channel_id(
        chan_end.connection_hops[0].clone(),
        port_id_on_b,
        chan_id_on_b,
    )?;

    Ok(ModuleExtras::empty())
}

/// Default validation callback function on the chan_close_init request for the host chain
pub fn on_chan_close_init_validate(
    _ctx_b: &impl InterchainAccountValidationContext,
    _port_id_on_b: &PortId,
    _chan_id_on_b: &ChannelId,
) -> Result<(), InterchainAccountError> {
    Err(InterchainAccountError::invalid("channel cannot be closed"))
}

/// Default execution callback function on the chan_close_init request for the host chain
pub fn on_chan_close_init_execute(
    _ctx_b: &mut impl InterchainAccountExecutionContext,
    _port_id_on_b: PortId,
    _chan_id_on_b: ChannelId,
) -> Result<ModuleExtras, InterchainAccountError> {
    Err(InterchainAccountError::invalid("channel cannot be closed"))
}

/// Default validation callback function on the chan_close_confirm request for the host chain
pub fn on_chan_close_confirm_validate(
    _ctx_b: &impl InterchainAccountValidationContext,
    _port_id_on_b: &PortId,
    _chan_ib_on_b: &ChannelId,
) -> Result<(), InterchainAccountError> {
    Ok(())
}

/// Default execution callback function on the chan_close_confirm request for the host chain
pub fn on_chan_close_confirm_execute(
    _ctx_b: &mut impl InterchainAccountExecutionContext,
    _port_id_on_b: PortId,
    _chan_id_on_b: ChannelId,
) -> Result<ModuleExtras, InterchainAccountError> {
    Ok(ModuleExtras::empty())
}

/// Default execution callback function on the recv_packet request for the host chain
pub fn on_recv_packet_execute(
    ctx_b: &mut impl InterchainAccountExecutionContext,
    packet: &Packet,
) -> (ModuleExtras, Acknowledgement) {
    let params = match ctx_b.get_params() {
        Ok(params) => params,
        Err(e) => {
            return (
                ModuleExtras::empty(),
                AcknowledgementStatus::error(e.into()).into(),
            )
        }
    };

    if !params.host_enabled {
        return (
            ModuleExtras::empty(),
            AcknowledgementStatus::error(
                InterchainAccountError::not_supported("host chain is not enabled.").into(),
            )
            .into(),
        );
    }

    if let Err(e) = on_recv_packet(ctx_b, packet) {
        return (
            ModuleExtras::empty(),
            AcknowledgementStatus::error(e.into()).into(),
        );
    }

    (
        ModuleExtras::empty(),
        AcknowledgementStatus::success(ack_success()).into(),
    )
}

/// Default validation callback function on the acknowledgement_packet request for the host chain
///
/// Note: if your chain serves as both the controller and the host chain, you
/// may utilize the default
/// [controller](crate::applications::interchain_accounts::controller::callback::on_acknowledgement_packet_validate)
/// callback implementation instead.
pub fn on_acknowledgement_packet_validate<Ctx>(
    _ctx_b: &impl InterchainAccountValidationContext,
    _packet: &Packet,
    _ack_on_b: &Acknowledgement,
    _relayer: &Signer,
) -> Result<(), InterchainAccountError> {
    Err(InterchainAccountError::not_allowed(
        "cannot receive acknowledgement on a host channel end, a host chain does not send a packet over the channel",
    ))
}

/// Default execution callback function on the acknowledgement_packet request for the host chain
///
/// Note: if your chain serves as both the controller and the host chain, you
/// may utilize the default
/// [controller](crate::applications::interchain_accounts::controller::callback::on_acknowledgement_packet_execute)
/// callback implementation instead.
pub fn on_acknowledgement_packet_execute(
    _ctx_b: &mut impl InterchainAccountExecutionContext,
    _packet: Packet,
    _ack_on_b: Acknowledgement,
    _relayer: Signer,
) -> (ModuleExtras, Result<(), InterchainAccountError>) {
    (ModuleExtras::empty(), Err(
        InterchainAccountError::not_allowed(
            "cannot receive acknowledgement on a host channel end, a host chain does not send a packet over the channel",
        )
    ))
}

/// Default validation callback function on the timeout_packet request for the host chain
///
/// Note: if your chain serves as both the controller and the host chain, you
/// may utilize the default
/// [controller](crate::applications::interchain_accounts::controller::callback::on_timeout_packet_validate)
/// callback implementation instead.
pub fn on_timeout_packet_validate<Ctx>(
    _ctx_b: &impl InterchainAccountValidationContext,
    _packet: &Packet,
    _relayer: &Signer,
) -> Result<(), InterchainAccountError> {
    Err(InterchainAccountError::not_allowed(
        "cannot cause a packet timeout on a host channel end, a host chain does not send a packet over the channel",
    ))
}

/// Default execution callback function on the timeout_packet request for the host chain
///
/// Note: if your chain serves as both the controller and the host chain, you
/// may utilize the default
/// [controller](crate::applications::interchain_accounts::controller::callback::on_timeout_packet_execute)
/// callback implementation instead.
pub fn on_timeout_packet_execute(
    _ctx_b: &mut impl InterchainAccountExecutionContext,
    _packet: Packet,
    _relayer: Signer,
) -> (ModuleExtras, Result<(), InterchainAccountError>) {
    (ModuleExtras::empty(), Err(
        InterchainAccountError::not_allowed(
            "cannot cause a packet timeout on a host channel end, a host chain does not send a packet over the channel",
        )
    ))
}
