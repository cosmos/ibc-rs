use crate::core::ics04_channel::events::OpenTry;
use crate::core::ics04_channel::handler::chan_open_try;
use crate::core::ics24_host::identifier::ChannelId;
use crate::{core::ics04_channel::msgs::chan_open_try::MsgChannelOpenTry, prelude::*};

use crate::core::ics04_channel::channel::{ChannelEnd, Counterparty, State};
use crate::core::ics04_channel::error::ChannelError;
use crate::core::ics26_routing::context::ModuleId;

use crate::events::IbcEvent;

use super::{ContextError, ExecutionContext, ValidationContext};

pub(super) fn chan_open_try_validate<ValCtx>(
    ctx_b: &ValCtx,
    module_id: ModuleId,
    msg: MsgChannelOpenTry,
) -> Result<(), ContextError>
where
    ValCtx: ValidationContext,
{
    chan_open_try::validate(ctx_b, &msg)?;
    let chan_id_on_b = ChannelId::new(ctx_b.channel_counter()?);

    let module = ctx_b
        .get_route(&module_id)
        .ok_or(ChannelError::RouteNotFound)?;
    module.on_chan_open_try_validate(
        msg.ordering,
        &msg.connection_hops_on_b,
        &msg.port_id_on_b,
        &chan_id_on_b,
        &Counterparty::new(msg.port_id_on_a.clone(), Some(msg.chan_id_on_a.clone())),
        &msg.version_supported_on_a,
    )?;

    Ok(())
}

pub(super) fn chan_open_try_execute<ExecCtx>(
    ctx_b: &mut ExecCtx,
    module_id: ModuleId,
    msg: MsgChannelOpenTry,
) -> Result<(), ContextError>
where
    ExecCtx: ExecutionContext,
{
    let chan_id_on_b = ChannelId::new(ctx_b.channel_counter()?);
    let module = ctx_b
        .get_route_mut(&module_id)
        .ok_or(ChannelError::RouteNotFound)?;

    let (extras, version) = module.on_chan_open_try_execute(
        msg.ordering,
        &msg.connection_hops_on_b,
        &msg.port_id_on_b,
        &chan_id_on_b,
        &Counterparty::new(msg.port_id_on_a.clone(), Some(msg.chan_id_on_a.clone())),
        &msg.version_supported_on_a,
    )?;

    let conn_id_on_b = msg.connection_hops_on_b[0].clone();

    // state changes
    {
        let port_channel_id_on_b = (msg.port_id_on_b.clone(), chan_id_on_b.clone());
        let chan_end_on_b = ChannelEnd::new(
            State::TryOpen,
            msg.ordering,
            Counterparty::new(msg.port_id_on_a.clone(), Some(msg.chan_id_on_a.clone())),
            msg.connection_hops_on_b.clone(),
            version.clone(),
        );

        ctx_b.store_channel(port_channel_id_on_b.clone(), chan_end_on_b)?;

        ctx_b.increase_channel_counter();

        // Associate also the channel end to its connection.
        ctx_b.store_connection_channels(conn_id_on_b.clone(), port_channel_id_on_b.clone())?;

        // Initialize send, recv, and ack sequence numbers.
        ctx_b.store_next_sequence_send(port_channel_id_on_b.clone(), 1.into())?;
        ctx_b.store_next_sequence_recv(port_channel_id_on_b.clone(), 1.into())?;
        ctx_b.store_next_sequence_ack(port_channel_id_on_b, 1.into())?;
    }

    // emit events and logs
    {
        ctx_b.log_message(format!(
            "success: channel open try with channel identifier: {chan_id_on_b}"
        ));

        let core_event = IbcEvent::OpenTryChannel(OpenTry::new(
            msg.port_id_on_b.clone(),
            chan_id_on_b.clone(),
            msg.port_id_on_a.clone(),
            msg.chan_id_on_a.clone(),
            conn_id_on_b,
            version,
        ));
        ctx_b.emit_ibc_event(core_event);

        for module_event in extras.events {
            ctx_b.emit_ibc_event(IbcEvent::AppModule(module_event));
        }

        for log_message in extras.log {
            ctx_b.log_message(log_message);
        }
    }

    Ok(())
}
