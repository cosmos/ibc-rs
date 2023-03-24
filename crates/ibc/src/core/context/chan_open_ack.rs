use crate::core::ics24_host::path::ChannelEndPath;
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
    let chan_end_path_on_a = ChannelEndPath::new(&msg.port_id_on_a, &msg.chan_id_on_a);
    let chan_end_on_a = ctx_a.channel_end(&chan_end_path_on_a)?;

    // state changes
    {
        let chan_end_on_a = {
            let mut chan_end_on_a = chan_end_on_a.clone();

            chan_end_on_a.set_state(State::Open);
            chan_end_on_a.set_version(msg.version_on_b.clone());
            chan_end_on_a.set_counterparty_channel_id(msg.chan_id_on_b.clone());

            chan_end_on_a
        };
        ctx_a.store_channel(&chan_end_path_on_a, chan_end_on_a)?;
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
    use crate::{
        core::context::chan_open_ack::chan_open_ack_execute, events::IbcEvent, prelude::*, Height,
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
                msgs::chan_open_ack::{
                    test_util::get_dummy_raw_msg_chan_open_ack, MsgChannelOpenAck,
                },
            },
            ics24_host::identifier::{ClientId, ConnectionId},
            ics26_routing::context::ModuleId,
        },
        events::IbcEventType,
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
        pub msg: MsgChannelOpenAck,
        pub client_id_on_a: ClientId,
        pub conn_id_on_a: ConnectionId,
        pub conn_end_on_a: ConnectionEnd,
        pub chan_end_on_a: ChannelEnd,
        pub proof_height: u64,
    }

    #[fixture]
    fn fixture() -> Fixture {
        let proof_height = 10;
        let mut context = MockContext::default();
        let module = DummyTransferModule::new();
        let module_id: ModuleId = MODULE_ID_STR.parse().unwrap();
        context.add_route(module_id.clone(), module).unwrap();

        let client_id_on_a = ClientId::new(mock_client_type(), 45).unwrap();
        let conn_id_on_a = ConnectionId::new(2);
        let conn_end_on_a = ConnectionEnd::new(
            ConnectionState::Open,
            client_id_on_a.clone(),
            ConnectionCounterparty::try_from(get_dummy_raw_counterparty(Some(0))).unwrap(),
            get_compatible_versions(),
            ZERO_DURATION,
        );

        let msg =
            MsgChannelOpenAck::try_from(get_dummy_raw_msg_chan_open_ack(proof_height)).unwrap();

        let chan_end_on_a = ChannelEnd::new(
            State::Init,
            Order::Unordered,
            Counterparty::new(msg.port_id_on_a.clone(), Some(msg.chan_id_on_b.clone())),
            vec![conn_id_on_a.clone()],
            msg.version_on_b.clone(),
        );

        Fixture {
            context,
            module_id,
            msg,
            client_id_on_a,
            conn_id_on_a,
            conn_end_on_a,
            chan_end_on_a,
            proof_height,
        }
    }

    #[rstest]
    fn chan_open_ack_execute_happy_path(fixture: Fixture) {
        let Fixture {
            context,
            module_id,
            msg,
            client_id_on_a,
            conn_id_on_a,
            conn_end_on_a,
            chan_end_on_a,
            proof_height,
            ..
        } = fixture;

        let mut context = context
            .with_client(&client_id_on_a, Height::new(0, proof_height).unwrap())
            .with_connection(conn_id_on_a, conn_end_on_a)
            .with_channel(
                msg.port_id_on_a.clone(),
                msg.chan_id_on_a.clone(),
                chan_end_on_a,
            );

        let res = chan_open_ack_execute(&mut context, module_id, msg);

        assert!(res.is_ok(), "Execution happy path");

        assert_eq!(context.events.len(), 2);
        assert!(matches!(
            context.events[0],
            IbcEvent::Message(IbcEventType::OpenAckChannel)
        ));
        assert!(matches!(context.events[1], IbcEvent::OpenAckChannel(_)));
    }
}
