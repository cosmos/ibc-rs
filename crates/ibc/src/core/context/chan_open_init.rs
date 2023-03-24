use crate::core::ics04_channel::events::OpenInit;
use crate::core::ics04_channel::handler::chan_open_init;
use crate::core::ics04_channel::msgs::chan_open_init::MsgChannelOpenInit;
use crate::core::ics24_host::path::{ChannelEndPath, SeqAckPath, SeqRecvPath, SeqSendPath};
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
        let chan_end_on_a = ChannelEnd::new(
            State::Init,
            msg.ordering,
            Counterparty::new(msg.port_id_on_b.clone(), None),
            msg.connection_hops_on_a.clone(),
            msg.version_proposal.clone(),
        );
        let chan_end_path_on_a = ChannelEndPath::new(&msg.port_id_on_a, &chan_id_on_a);
        ctx_a.store_channel(&chan_end_path_on_a, chan_end_on_a)?;

        ctx_a.increase_channel_counter();

        // Initialize send, recv, and ack sequence numbers.
        let seq_send_path = SeqSendPath::new(&msg.port_id_on_a, &chan_id_on_a);
        ctx_a.store_next_sequence_send(&seq_send_path, 1.into())?;

        let seq_recv_path = SeqRecvPath::new(&msg.port_id_on_a, &chan_id_on_a);
        ctx_a.store_next_sequence_recv(&seq_recv_path, 1.into())?;

        let seq_ack_path = SeqAckPath::new(&msg.port_id_on_a, &chan_id_on_a);
        ctx_a.store_next_sequence_ack(&seq_ack_path, 1.into())?;
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
    use crate::core::ics03_connection::connection::State as ConnectionState;
    use crate::core::ics24_host::identifier::ConnectionId;
    use crate::test_utils::DummyTransferModule;
    use crate::{
        core::{
            ics03_connection::{
                connection::ConnectionEnd,
                msgs::conn_open_init::{
                    test_util::get_dummy_raw_msg_conn_open_init, MsgConnectionOpenInit,
                },
                version::get_compatible_versions,
            },
            ics04_channel::msgs::chan_open_init::test_util::get_dummy_raw_msg_chan_open_init,
        },
        events::IbcEventType,
        mock::context::MockContext,
    };

    use super::*;
    use rstest::*;

    pub struct Fixture {
        pub context: MockContext,
        pub module_id: ModuleId,
        pub msg: MsgChannelOpenInit,
        pub conn_end_on_a: ConnectionEnd,
    }

    #[fixture]
    fn fixture() -> Fixture {
        let msg = MsgChannelOpenInit::try_from(get_dummy_raw_msg_chan_open_init(None)).unwrap();

        let mut context = MockContext::default();
        let module_id: ModuleId = MODULE_ID_STR.parse().unwrap();
        let module = DummyTransferModule::new();
        context.add_route(module_id.clone(), module).unwrap();

        let msg_conn_init =
            MsgConnectionOpenInit::try_from(get_dummy_raw_msg_conn_open_init()).unwrap();

        let conn_end_on_a = ConnectionEnd::new(
            ConnectionState::Init,
            msg_conn_init.client_id_on_a.clone(),
            msg_conn_init.counterparty.clone(),
            get_compatible_versions(),
            msg_conn_init.delay_period,
        );

        Fixture {
            context,
            module_id,
            msg,
            conn_end_on_a,
        }
    }

    #[rstest]
    fn chan_open_init_execute_events(fixture: Fixture) {
        let Fixture {
            context,
            module_id,
            msg,
            conn_end_on_a,
        } = fixture;

        let mut context = context.with_connection(ConnectionId::default(), conn_end_on_a);

        let res = chan_open_init_execute(&mut context, module_id, msg);

        assert!(res.is_ok(), "Execution succeeds; good parameters");

        assert_eq!(context.events.len(), 2);
        assert!(matches!(
            context.events[0],
            IbcEvent::Message(IbcEventType::OpenInitChannel)
        ));
        assert!(matches!(context.events[1], IbcEvent::OpenInitChannel(_)));
    }
}
