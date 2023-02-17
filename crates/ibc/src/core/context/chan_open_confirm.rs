use crate::core::ics04_channel::events::OpenConfirm;
use crate::core::ics04_channel::handler::chan_open_confirm;
use crate::core::ics24_host::path::ChannelEndPath;
use crate::{core::ics04_channel::msgs::chan_open_confirm::MsgChannelOpenConfirm, prelude::*};

use crate::core::ics04_channel::channel::State;
use crate::core::ics04_channel::error::ChannelError;
use crate::core::ics26_routing::context::ModuleId;

use crate::events::IbcEvent;

use super::{ContextError, ExecutionContext, ValidationContext};

pub(super) fn chan_open_confirm_validate<ValCtx>(
    ctx_b: &ValCtx,
    module_id: ModuleId,
    msg: MsgChannelOpenConfirm,
) -> Result<(), ContextError>
where
    ValCtx: ValidationContext,
{
    chan_open_confirm::validate(ctx_b, &msg)?;

    let module = ctx_b
        .get_route(&module_id)
        .ok_or(ChannelError::RouteNotFound)?;
    module.on_chan_open_confirm_validate(&msg.port_id_on_b, &msg.chan_id_on_b)?;

    Ok(())
}

pub(super) fn chan_open_confirm_execute<ExecCtx>(
    ctx_b: &mut ExecCtx,
    module_id: ModuleId,
    msg: MsgChannelOpenConfirm,
) -> Result<(), ContextError>
where
    ExecCtx: ExecutionContext,
{
    let module = ctx_b
        .get_route_mut(&module_id)
        .ok_or(ChannelError::RouteNotFound)?;

    let extras = module.on_chan_open_confirm_execute(&msg.port_id_on_b, &msg.chan_id_on_b)?;
    let chan_end_path_on_b = ChannelEndPath::new(&msg.port_id_on_b, &msg.chan_id_on_b);
    let chan_end_on_b = ctx_b.channel_end(&chan_end_path_on_b)?;

    // state changes
    {
        let chan_end_on_b = {
            let mut chan_end_on_b = chan_end_on_b.clone();
            chan_end_on_b.set_state(State::Open);

            chan_end_on_b
        };
        ctx_b.store_channel(&chan_end_path_on_b, chan_end_on_b)?;
    }

    // emit events and logs
    {
        ctx_b.log_message("success: channel open confirm".to_string());

        let conn_id_on_b = chan_end_on_b.connection_hops[0].clone();
        let port_id_on_a = chan_end_on_b.counterparty().port_id.clone();
        let chan_id_on_a = chan_end_on_b
            .counterparty()
            .channel_id
            .clone()
            .ok_or(ContextError::ChannelError(ChannelError::Other {
            description:
                "internal error: ChannelEnd doesn't have a counterparty channel id in OpenConfirm"
                    .to_string(),
        }))?;

        let core_event = IbcEvent::OpenConfirmChannel(OpenConfirm::new(
            msg.port_id_on_b.clone(),
            msg.chan_id_on_b.clone(),
            port_id_on_a,
            chan_id_on_a,
            conn_id_on_b,
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

#[cfg(test)]
mod tests {
    use crate::{
        core::{context::chan_open_confirm::chan_open_confirm_execute, ics04_channel::Version},
        events::IbcEvent,
        prelude::*,
        Height,
    };
    use rstest::*;

    use crate::{
        applications::transfer::MODULE_ID_STR,
        core::{
            ics03_connection::{
                connection::ConnectionEnd, msgs::test_util::get_dummy_raw_counterparty,
                version::get_compatible_versions,
            },
            ics04_channel::{
                channel::{ChannelEnd, Counterparty, Order, State},
                msgs::chan_open_confirm::{
                    test_util::get_dummy_raw_msg_chan_open_confirm, MsgChannelOpenConfirm,
                },
            },
            ics24_host::identifier::{ChannelId, ClientId, ConnectionId},
            ics26_routing::context::ModuleId,
        },
        mock::context::MockContext,
        test_utils::DummyTransferModule,
        timestamp::ZERO_DURATION,
    };

    use crate::core::ics03_connection::connection::Counterparty as ConnectionCounterparty;
    use crate::core::ics03_connection::connection::State as ConnectionState;
    use crate::mock::client_state::client_type as mock_client_type;

    pub struct Fixture {
        pub context: MockContext,
        pub module_id: ModuleId,
        pub msg: MsgChannelOpenConfirm,
        pub client_id_on_b: ClientId,
        pub conn_id_on_b: ConnectionId,
        pub conn_end_on_b: ConnectionEnd,
        pub chan_end_on_b: ChannelEnd,
        pub proof_height: u64,
    }

    #[fixture]
    fn fixture() -> Fixture {
        let proof_height = 10;
        let mut context = MockContext::default();
        let module = DummyTransferModule::new(context.ibc_store_share());
        let module_id: ModuleId = MODULE_ID_STR.parse().unwrap();
        context.add_route(module_id.clone(), module).unwrap();

        let client_id_on_b = ClientId::new(mock_client_type(), 45).unwrap();
        let conn_id_on_b = ConnectionId::new(2);
        let conn_end_on_b = ConnectionEnd::new(
            ConnectionState::Open,
            client_id_on_b.clone(),
            ConnectionCounterparty::try_from(get_dummy_raw_counterparty(Some(0))).unwrap(),
            get_compatible_versions(),
            ZERO_DURATION,
        );

        let msg =
            MsgChannelOpenConfirm::try_from(get_dummy_raw_msg_chan_open_confirm(proof_height))
                .unwrap();

        let chan_end_on_b = ChannelEnd::new(
            State::TryOpen,
            Order::Unordered,
            Counterparty::new(msg.port_id_on_b.clone(), Some(ChannelId::default())),
            vec![conn_id_on_b.clone()],
            Version::default(),
        );

        Fixture {
            context,
            module_id,
            msg,
            client_id_on_b,
            conn_id_on_b,
            conn_end_on_b,
            chan_end_on_b,
            proof_height,
        }
    }

    #[rstest]
    fn chan_open_confirm_execute_happy_path(fixture: Fixture) {
        let Fixture {
            context,
            module_id,
            msg,
            client_id_on_b,
            conn_id_on_b,
            conn_end_on_b,
            chan_end_on_b,
            proof_height,
            ..
        } = fixture;

        let mut context = context
            .with_client(&client_id_on_b, Height::new(0, proof_height).unwrap())
            .with_connection(conn_id_on_b, conn_end_on_b)
            .with_channel(
                msg.port_id_on_b.clone(),
                ChannelId::default(),
                chan_end_on_b,
            );

        let res = chan_open_confirm_execute(&mut context, module_id, msg);

        assert!(res.is_ok(), "Execution happy path");

        assert_eq!(context.events.len(), 1);
        assert!(matches!(
            context.events.first().unwrap(),
            &IbcEvent::OpenConfirmChannel(_)
        ));
    }
}
