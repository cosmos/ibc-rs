use crate::core::ics04_channel::events::OpenTry;
use crate::core::ics04_channel::handler::chan_open_try;
use crate::core::ics24_host::identifier::ChannelId;
use crate::core::ics24_host::path::{ChannelEndPath, SeqAckPath, SeqRecvPath, SeqSendPath};
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
        let chan_end_on_b = ChannelEnd::new(
            State::TryOpen,
            msg.ordering,
            Counterparty::new(msg.port_id_on_a.clone(), Some(msg.chan_id_on_a.clone())),
            msg.connection_hops_on_b.clone(),
            version.clone(),
        );

        let chan_end_path_on_b = ChannelEndPath::new(&msg.port_id_on_b, &chan_id_on_b);
        ctx_b.store_channel(&chan_end_path_on_b, chan_end_on_b)?;
        ctx_b.increase_channel_counter();

        // Initialize send, recv, and ack sequence numbers.
        let seq_send_path = SeqSendPath::new(&msg.port_id_on_b, &chan_id_on_b);
        ctx_b.store_next_sequence_send(&seq_send_path, 1.into())?;

        let seq_recv_path = SeqRecvPath::new(&msg.port_id_on_b, &chan_id_on_b);
        ctx_b.store_next_sequence_recv(&seq_recv_path, 1.into())?;

        let seq_ack_path = SeqAckPath::new(&msg.port_id_on_b, &chan_id_on_b);
        ctx_b.store_next_sequence_ack(&seq_ack_path, 1.into())?;
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
        ctx_b.emit_ibc_event(IbcEvent::Message(core_event.event_type()));
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
        applications::transfer::MODULE_ID_STR,
        core::{context::chan_open_try::chan_open_try_execute, ics26_routing::context::ModuleId},
        events::{IbcEvent, IbcEventType},
        prelude::*,
        test_utils::DummyTransferModule,
        Height,
    };
    use rstest::*;

    use crate::{
        core::{
            ics03_connection::{
                connection::ConnectionEnd, msgs::test_util::get_dummy_raw_counterparty,
                version::get_compatible_versions,
            },
            ics04_channel::msgs::chan_open_try::{
                test_util::get_dummy_raw_msg_chan_open_try, MsgChannelOpenTry,
            },
            ics24_host::identifier::{ClientId, ConnectionId},
        },
        mock::context::MockContext,
        timestamp::ZERO_DURATION,
    };

    use crate::core::ics03_connection::connection::Counterparty as ConnectionCounterparty;
    use crate::core::ics03_connection::connection::State as ConnectionState;
    use crate::mock::client_state::client_type as mock_client_type;

    pub struct Fixture {
        pub context: MockContext,
        pub module_id: ModuleId,
        pub msg: MsgChannelOpenTry,
        pub client_id_on_b: ClientId,
        pub conn_id_on_b: ConnectionId,
        pub conn_end_on_b: ConnectionEnd,
        pub proof_height: u64,
    }

    #[fixture]
    fn fixture() -> Fixture {
        let proof_height = 10;
        let conn_id_on_b = ConnectionId::new(2);
        let client_id_on_b = ClientId::new(mock_client_type(), 45).unwrap();

        // This is the connection underlying the channel we're trying to open.
        let conn_end_on_b = ConnectionEnd::new(
            ConnectionState::Open,
            client_id_on_b.clone(),
            ConnectionCounterparty::try_from(get_dummy_raw_counterparty(Some(0))).unwrap(),
            get_compatible_versions(),
            ZERO_DURATION,
        );

        // We're going to test message processing against this message.
        // Note: we make the counterparty's channel_id `None`.
        let mut msg =
            MsgChannelOpenTry::try_from(get_dummy_raw_msg_chan_open_try(proof_height)).unwrap();

        let hops = vec![conn_id_on_b.clone()];
        msg.connection_hops_on_b = hops;

        let mut context = MockContext::default();
        let module = DummyTransferModule::new();
        let module_id: ModuleId = MODULE_ID_STR.parse().unwrap();
        context.add_route(module_id.clone(), module).unwrap();

        Fixture {
            context,
            module_id,
            msg,
            client_id_on_b,
            conn_id_on_b,
            conn_end_on_b,
            proof_height,
        }
    }

    #[rstest]
    fn chan_open_try_execute_events(fixture: Fixture) {
        let Fixture {
            context,
            module_id,
            msg,
            client_id_on_b,
            conn_id_on_b,
            conn_end_on_b,
            proof_height,
            ..
        } = fixture;

        let mut context = context
            .with_client(&client_id_on_b, Height::new(0, proof_height).unwrap())
            .with_connection(conn_id_on_b, conn_end_on_b);

        let res = chan_open_try_execute(&mut context, module_id, msg);

        assert!(res.is_ok(), "Execution success: happy path");

        assert_eq!(context.events.len(), 2);
        assert!(matches!(
            context.events[0],
            IbcEvent::Message(IbcEventType::OpenTryChannel)
        ));
        assert!(matches!(context.events[1], IbcEvent::OpenTryChannel(_)));
    }
}
