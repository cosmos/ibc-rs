use crate::{core::ics24_host::path::ChannelEndPath, prelude::*};

use crate::core::ics04_channel::channel::State;
use crate::core::ics04_channel::error::ChannelError;
use crate::core::ics04_channel::events::CloseConfirm;
use crate::core::ics04_channel::handler::chan_close_confirm;
use crate::core::ics04_channel::msgs::chan_close_confirm::MsgChannelCloseConfirm;
use crate::core::ics26_routing::context::ModuleId;

use crate::events::IbcEvent;

use super::{ContextError, ExecutionContext, ValidationContext};

pub(super) fn chan_close_confirm_validate<ValCtx>(
    ctx_b: &ValCtx,
    module_id: ModuleId,
    msg: MsgChannelCloseConfirm,
) -> Result<(), ContextError>
where
    ValCtx: ValidationContext,
{
    chan_close_confirm::validate(ctx_b, &msg)?;

    let module = ctx_b
        .get_route(&module_id)
        .ok_or(ChannelError::RouteNotFound)?;
    module.on_chan_close_confirm_validate(&msg.port_id_on_b, &msg.chan_id_on_b)?;

    Ok(())
}

pub(super) fn chan_close_confirm_execute<ExecCtx>(
    ctx_b: &mut ExecCtx,
    module_id: ModuleId,
    msg: MsgChannelCloseConfirm,
) -> Result<(), ContextError>
where
    ExecCtx: ExecutionContext,
{
    let module = ctx_b
        .get_route_mut(&module_id)
        .ok_or(ChannelError::RouteNotFound)?;
    let extras = module.on_chan_close_confirm_execute(&msg.port_id_on_b, &msg.chan_id_on_b)?;
    let chan_end_path_on_b = ChannelEndPath::new(&msg.port_id_on_b, &msg.chan_id_on_b);
    let chan_end_on_b = ctx_b.channel_end(&chan_end_path_on_b)?;

    // state changes
    {
        let chan_end_on_b = {
            let mut chan_end_on_b = chan_end_on_b.clone();
            chan_end_on_b.set_state(State::Closed);
            chan_end_on_b
        };
        ctx_b.store_channel(&chan_end_path_on_b, chan_end_on_b)?;
    }

    // emit events and logs
    {
        ctx_b.log_message("success: channel close confirm".to_string());

        let core_event = {
            let port_id_on_a = chan_end_on_b.counterparty().port_id.clone();
            let chan_id_on_a = chan_end_on_b
                .counterparty()
                .channel_id
                .clone()
                .ok_or(ContextError::ChannelError(ChannelError::Other {
                description:
                    "internal error: ChannelEnd doesn't have a counterparty channel id in CloseInit"
                        .to_string(),
            }))?;
            let conn_id_on_b = chan_end_on_b.connection_hops[0].clone();

            IbcEvent::CloseConfirmChannel(CloseConfirm::new(
                msg.port_id_on_b.clone(),
                msg.chan_id_on_b.clone(),
                port_id_on_a,
                chan_id_on_a,
                conn_id_on_b,
            ))
        };
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

#[cfg(test)]
mod tests {
    use crate::applications::transfer::MODULE_ID_STR;
    use crate::core::context::chan_close_confirm::chan_close_confirm_execute;
    use crate::core::ics04_channel::msgs::chan_close_confirm::test_util::get_dummy_raw_msg_chan_close_confirm;
    use crate::core::ics04_channel::msgs::chan_close_confirm::MsgChannelCloseConfirm;
    use crate::core::ics26_routing::context::ModuleId;
    use crate::core::ValidationContext;
    use crate::events::IbcEvent;
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

    #[test]
    fn chan_close_confirm_event_height() {
        let client_id = ClientId::new(mock_client_type(), 24).unwrap();
        let conn_id = ConnectionId::new(2);
        let default_context = MockContext::default();
        let client_consensus_state_height = default_context.host_height().unwrap();

        let conn_end = ConnectionEnd::new(
            ConnectionState::Open,
            client_id.clone(),
            ConnectionCounterparty::try_from(get_dummy_raw_counterparty(Some(0))).unwrap(),
            get_compatible_versions(),
            ZERO_DURATION,
        );

        let msg_chan_close_confirm = MsgChannelCloseConfirm::try_from(
            get_dummy_raw_msg_chan_close_confirm(client_consensus_state_height.revision_height()),
        )
        .unwrap();

        let chan_end = ChannelEnd::new(
            ChannelState::Open,
            Order::default(),
            Counterparty::new(
                msg_chan_close_confirm.port_id_on_b.clone(),
                Some(msg_chan_close_confirm.chan_id_on_b.clone()),
            ),
            vec![conn_id.clone()],
            Version::default(),
        );

        let mut context = default_context
            .with_client(&client_id, client_consensus_state_height)
            .with_connection(conn_id, conn_end)
            .with_channel(
                msg_chan_close_confirm.port_id_on_b.clone(),
                msg_chan_close_confirm.chan_id_on_b.clone(),
                chan_end,
            );

        let module = DummyTransferModule::new(context.ibc_store_share());
        let module_id: ModuleId = MODULE_ID_STR.parse().unwrap();
        context.add_route(module_id.clone(), module).unwrap();

        let res = chan_close_confirm_execute(&mut context, module_id, msg_chan_close_confirm);
        assert!(res.is_ok(), "Execution success: happy path");

        assert_eq!(context.events.len(), 1);
        assert!(matches!(
            context.events.first().unwrap(),
            &IbcEvent::CloseConfirmChannel(_)
        ));
    }
}
