use crate::core::ics04_channel::events::CloseInit;
use crate::core::ics04_channel::handler::chan_close_init;
use crate::core::ics24_host::path::ChannelEndPath;
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
    let chan_end_path_on_a = ChannelEndPath::new(&msg.port_id_on_a, &msg.chan_id_on_a);
    let chan_end_on_a = ctx_a.channel_end(&chan_end_path_on_a)?;

    // state changes
    {
        let chan_end_on_a = {
            let mut chan_end_on_a = chan_end_on_a.clone();
            chan_end_on_a.set_state(State::Closed);
            chan_end_on_a
        };

        ctx_a.store_channel(&chan_end_path_on_a, chan_end_on_a)?;
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
        ctx_a.emit_ibc_event(IbcEvent::Message(core_event.event_type()));
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

#[cfg(test)]
mod tests {
    use crate::applications::transfer::MODULE_ID_STR;
    use crate::core::ics04_channel::msgs::chan_close_init::test_util::get_dummy_raw_msg_chan_close_init;
    use crate::core::ics04_channel::msgs::chan_close_init::MsgChannelCloseInit;
    use crate::core::ics26_routing::context::ModuleId;
    use crate::core::ValidationContext;
    use crate::events::{IbcEvent, IbcEventType};
    use crate::prelude::*;

    use crate::core::ics03_connection::connection::ConnectionEnd;
    use crate::core::ics03_connection::connection::Counterparty as ConnectionCounterparty;
    use crate::core::ics03_connection::connection::State as ConnectionState;
    use crate::core::ics03_connection::msgs::test_util::get_dummy_raw_counterparty;
    use crate::core::ics03_connection::version::get_compatible_versions;
    use crate::core::ics04_channel::channel::{
        ChannelEnd, Counterparty, Order, State as ChannelState,
    };
    use crate::core::ics04_channel::Version;
    use crate::core::ics24_host::identifier::{ClientId, ConnectionId};

    use crate::mock::client_state::client_type as mock_client_type;
    use crate::mock::context::MockContext;
    use crate::test_utils::DummyTransferModule;
    use crate::timestamp::ZERO_DURATION;

    use super::chan_close_init_execute;
    #[test]
    fn chan_close_init_event_height() {
        let client_id = ClientId::new(mock_client_type(), 24).unwrap();
        let conn_id = ConnectionId::new(2);

        let conn_end = ConnectionEnd::new(
            ConnectionState::Open,
            client_id.clone(),
            ConnectionCounterparty::try_from(get_dummy_raw_counterparty(Some(0))).unwrap(),
            get_compatible_versions(),
            ZERO_DURATION,
        );

        let msg_chan_close_init =
            MsgChannelCloseInit::try_from(get_dummy_raw_msg_chan_close_init()).unwrap();

        let chan_end = ChannelEnd::new(
            ChannelState::Open,
            Order::default(),
            Counterparty::new(
                msg_chan_close_init.port_id_on_a.clone(),
                Some(msg_chan_close_init.chan_id_on_a.clone()),
            ),
            vec![conn_id.clone()],
            Version::default(),
        );

        let mut context = {
            let mut default_context = MockContext::default();
            let client_consensus_state_height = default_context.host_height().unwrap();

            let module = DummyTransferModule::new();
            let module_id: ModuleId = MODULE_ID_STR.parse().unwrap();
            default_context.add_route(module_id, module).unwrap();

            default_context
                .with_client(&client_id, client_consensus_state_height)
                .with_connection(conn_id, conn_end)
                .with_channel(
                    msg_chan_close_init.port_id_on_a.clone(),
                    msg_chan_close_init.chan_id_on_a.clone(),
                    chan_end,
                )
        };

        let res = chan_close_init_execute(
            &mut context,
            MODULE_ID_STR.parse().unwrap(),
            msg_chan_close_init,
        );
        assert!(res.is_ok(), "Execution happy path");

        assert_eq!(context.events.len(), 2);
        assert!(matches!(
            context.events[0],
            IbcEvent::Message(IbcEventType::CloseInitChannel)
        ));
        assert!(matches!(context.events[1], IbcEvent::CloseInitChannel(_)));
    }
}
