use crate::core::ics04_channel::events::OpenInit;
use crate::core::ics04_channel::handler::chan_open_init;
use crate::core::ics04_channel::msgs::chan_open_init::MsgChannelOpenInit;
use crate::prelude::*;

use crate::core::ics24_host::identifier::ChannelId;

use crate::core::ics04_channel::channel::{ChannelEnd, Counterparty, State};
use crate::core::ics04_channel::error::ChannelError;
use crate::core::ics26_routing::context::ModuleId;

use crate::events::IbcEvent;

use super::{ContextError, ExecutionContext, ValidationContext};
pub(super) fn chan_open_init_validate<ValCtx>(
    ctx_a: &ValCtx,
    module_id: ModuleId,
    msg: MsgChannelOpenInit,
) -> Result<(), ContextError>
where
    ValCtx: ValidationContext,
{
    chan_open_init::validate(ctx_a, &msg)?;
    let chan_id_on_a = ChannelId::new(ctx_a.channel_counter()?);

    let module = ctx_a
        .get_route(&module_id)
        .ok_or(ChannelError::RouteNotFound)?;
    module.on_chan_open_init_validate(
        msg.ordering,
        &msg.connection_hops_on_a,
        &msg.port_id_on_a,
        &chan_id_on_a,
        &Counterparty::new(msg.port_id_on_b.clone(), None),
        &msg.version_proposal,
    )?;

    Ok(())
}

pub(super) fn chan_open_init_execute<ExecCtx>(
    ctx_a: &mut ExecCtx,
    module_id: ModuleId,
    msg: MsgChannelOpenInit,
) -> Result<(), ContextError>
where
    ExecCtx: ExecutionContext,
{
    let chan_id_on_a = ChannelId::new(ctx_a.channel_counter()?);
    let module = ctx_a
        .get_route_mut(&module_id)
        .ok_or(ChannelError::RouteNotFound)?;
    let (extras, version) = module.on_chan_open_init_execute(
        msg.ordering,
        &msg.connection_hops_on_a,
        &msg.port_id_on_a,
        &chan_id_on_a,
        &Counterparty::new(msg.port_id_on_b.clone(), None),
        &msg.version_proposal,
    )?;

    let conn_id_on_a = msg.connection_hops_on_a[0].clone();

    // state changes
    {
        let port_channel_id_on_a = (msg.port_id_on_a.clone(), chan_id_on_a.clone());
        let chan_end_on_a = ChannelEnd::new(
            State::Init,
            msg.ordering,
            Counterparty::new(msg.port_id_on_b.clone(), None),
            msg.connection_hops_on_a.clone(),
            msg.version_proposal.clone(),
        );

        ctx_a.store_channel(port_channel_id_on_a.clone(), chan_end_on_a)?;

        ctx_a.increase_channel_counter();

        // Associate also the channel end to its connection.
        ctx_a.store_connection_channels(conn_id_on_a.clone(), port_channel_id_on_a.clone())?;

        // Initialize send, recv, and ack sequence numbers.
        ctx_a.store_next_sequence_send(port_channel_id_on_a.clone(), 1.into())?;
        ctx_a.store_next_sequence_recv(port_channel_id_on_a.clone(), 1.into())?;
        ctx_a.store_next_sequence_ack(port_channel_id_on_a, 1.into())?;
    }

    // emit events and logs
    {
        ctx_a.log_message(format!(
            "success: channel open init with channel identifier: {chan_id_on_a}"
        ));
        let core_event = IbcEvent::OpenInitChannel(OpenInit::new(
            msg.port_id_on_a.clone(),
            chan_id_on_a.clone(),
            msg.port_id_on_b,
            conn_id_on_a,
            version,
        ));
        ctx_a.emit_ibc_event(core_event);

        for module_event in extras.events {
            ctx_a.emit_ibc_event(IbcEvent::AppModule(module_event));
        }

        for log_message in extras.log {
            ctx_a.log_message(log_message);
        }
    }

    Ok(())
}
