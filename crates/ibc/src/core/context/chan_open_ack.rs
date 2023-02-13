use crate::{core::ics04_channel::events::OpenAck, prelude::*};

use crate::core::ics04_channel::handler::chan_open_ack;
use crate::core::ics04_channel::msgs::chan_open_ack::MsgChannelOpenAck;

use crate::core::ics04_channel::channel::State;
use crate::core::ics04_channel::error::ChannelError;
use crate::core::ics26_routing::context::ModuleId;

use crate::events::IbcEvent;

use super::{ContextError, ExecutionContext, ValidationContext};

pub(super) fn chan_open_ack_validate<ValCtx>(
    ctx_a: &ValCtx,
    module_id: ModuleId,
    msg: MsgChannelOpenAck,
) -> Result<(), ContextError>
where
    ValCtx: ValidationContext,
{
    chan_open_ack::validate(ctx_a, &msg)?;

    let module = ctx_a
        .get_route(&module_id)
        .ok_or(ChannelError::RouteNotFound)?;
    module.on_chan_open_ack_validate(&msg.port_id_on_a, &msg.chan_id_on_a, &msg.version_on_b)?;

    Ok(())
}

pub(super) fn chan_open_ack_execute<ExecCtx>(
    ctx_a: &mut ExecCtx,
    module_id: ModuleId,
    msg: MsgChannelOpenAck,
) -> Result<(), ContextError>
where
    ExecCtx: ExecutionContext,
{
    let module = ctx_a
        .get_route_mut(&module_id)
        .ok_or(ChannelError::RouteNotFound)?;
    let extras =
        module.on_chan_open_ack_execute(&msg.port_id_on_a, &msg.chan_id_on_a, &msg.version_on_b)?;

    let chan_end_on_a = ctx_a.channel_end(&(msg.port_id_on_a.clone(), msg.chan_id_on_a.clone()))?;

    // state changes
    {
        let port_channel_id_on_a = (msg.port_id_on_a.clone(), msg.chan_id_on_a.clone());
        let chan_end_on_a = {
            let mut chan_end_on_a = chan_end_on_a.clone();

            chan_end_on_a.set_state(State::Open);
            chan_end_on_a.set_version(msg.version_on_b.clone());
            chan_end_on_a.set_counterparty_channel_id(msg.chan_id_on_b.clone());

            chan_end_on_a
        };

        ctx_a.store_channel(port_channel_id_on_a, chan_end_on_a)?;
    }

    // emit events and logs
    {
        ctx_a.log_message("success: channel open ack".to_string());

        let core_event = {
            let port_id_on_b = chan_end_on_a.counterparty().port_id.clone();
            let conn_id_on_a = chan_end_on_a.connection_hops[0].clone();

            IbcEvent::OpenAckChannel(OpenAck::new(
                msg.port_id_on_a.clone(),
                msg.chan_id_on_a.clone(),
                port_id_on_b,
                msg.chan_id_on_b,
                conn_id_on_a,
            ))
        };
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
