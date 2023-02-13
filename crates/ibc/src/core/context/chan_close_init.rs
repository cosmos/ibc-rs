use crate::core::ics04_channel::events::CloseInit;
use crate::core::ics04_channel::handler::chan_close_init;
use crate::{core::ics04_channel::msgs::chan_close_init::MsgChannelCloseInit, prelude::*};

use crate::core::ics04_channel::channel::State;
use crate::core::ics04_channel::error::ChannelError;
use crate::core::ics26_routing::context::ModuleId;

use crate::events::IbcEvent;

use super::{ContextError, ExecutionContext, ValidationContext};
pub(super) fn chan_close_init_validate<ValCtx>(
    ctx_a: &ValCtx,
    module_id: ModuleId,
    msg: MsgChannelCloseInit,
) -> Result<(), ContextError>
where
    ValCtx: ValidationContext,
{
    chan_close_init::validate(ctx_a, &msg)?;

    let module = ctx_a
        .get_route(&module_id)
        .ok_or(ChannelError::RouteNotFound)?;
    module.on_chan_close_init_validate(&msg.port_id_on_a, &msg.chan_id_on_a)?;

    Ok(())
}

pub(super) fn chan_close_init_execute<ExecCtx>(
    ctx_a: &mut ExecCtx,
    module_id: ModuleId,
    msg: MsgChannelCloseInit,
) -> Result<(), ContextError>
where
    ExecCtx: ExecutionContext,
{
    let module = ctx_a
        .get_route_mut(&module_id)
        .ok_or(ChannelError::RouteNotFound)?;
    let extras = module.on_chan_close_init_execute(&msg.port_id_on_a, &msg.chan_id_on_a)?;

    let chan_end_on_a = ctx_a.channel_end(&(msg.port_id_on_a.clone(), msg.chan_id_on_a.clone()))?;

    // state changes
    {
        let chan_end_on_a = {
            let mut chan_end_on_a = chan_end_on_a.clone();
            chan_end_on_a.set_state(State::Closed);
            chan_end_on_a
        };

        let port_channel_id_on_a = (msg.port_id_on_a.clone(), msg.chan_id_on_a.clone());
        ctx_a.store_channel(port_channel_id_on_a, chan_end_on_a)?;
    }

    // emit events and logs
    {
        ctx_a.log_message("success: channel close init".to_string());

        let core_event = {
            let port_id_on_b = chan_end_on_a.counterparty().port_id.clone();
            let chan_id_on_b = chan_end_on_a
                .counterparty()
                .channel_id
                .clone()
                .ok_or(ContextError::ChannelError(ChannelError::Other {
                description:
                    "internal error: ChannelEnd doesn't have a counterparty channel id in CloseInit"
                        .to_string(),
            }))?;
            let conn_id_on_a = chan_end_on_a.connection_hops[0].clone();

            IbcEvent::CloseInitChannel(CloseInit::new(
                msg.port_id_on_a.clone(),
                msg.chan_id_on_a.clone(),
                port_id_on_b,
                chan_id_on_b,
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
