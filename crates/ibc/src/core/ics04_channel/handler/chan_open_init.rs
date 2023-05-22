//! Protocol logic specific to ICS4 messages of type `MsgChannelOpenInit`.

use crate::core::ics02_client::client_state::StaticClientStateBase;
use crate::prelude::*;

use crate::core::events::{IbcEvent, MessageEvent};
use crate::core::ics04_channel::channel::{ChannelEnd, Counterparty, State};
use crate::core::ics04_channel::error::ChannelError;
use crate::core::ics04_channel::events::OpenInit;
use crate::core::ics04_channel::msgs::chan_open_init::MsgChannelOpenInit;
use crate::core::ics24_host::identifier::ChannelId;
use crate::core::ics24_host::path::{ChannelEndPath, SeqAckPath, SeqRecvPath, SeqSendPath};
use crate::core::router::ModuleId;
use crate::core::{ContextError, StaticExecutionContext, StaticValidationContext};

pub(crate) fn chan_open_init_validate<ValCtx>(
    ctx_a: &ValCtx,
    module_id: ModuleId,
    msg: MsgChannelOpenInit,
) -> Result<(), ContextError>
where
    ValCtx: StaticValidationContext,
{
    validate(ctx_a, &msg)?;
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

pub(crate) fn chan_open_init_execute<ExecCtx>(
    ctx_a: &mut ExecCtx,
    module_id: ModuleId,
    msg: MsgChannelOpenInit,
) -> Result<(), ContextError>
where
    ExecCtx: StaticExecutionContext,
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
        )?;
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
        ctx_a.emit_ibc_event(IbcEvent::Message(MessageEvent::Channel));
        ctx_a.emit_ibc_event(core_event);

        for module_event in extras.events {
            ctx_a.emit_ibc_event(IbcEvent::Module(module_event));
        }

        for log_message in extras.log {
            ctx_a.log_message(log_message);
        }
    }

    Ok(())
}

fn validate<Ctx>(ctx_a: &Ctx, msg: &MsgChannelOpenInit) -> Result<(), ContextError>
where
    Ctx: StaticValidationContext,
{
    ctx_a.validate_message_signer(&msg.signer)?;

    msg.verify_connection_hops_length()?;
    // An IBC connection running on the local (host) chain should exist.
    let conn_end_on_a = ctx_a.connection_end(&msg.connection_hops_on_a[0])?;

    // Note: Not needed check if the connection end is OPEN. Optimistic channel handshake is allowed.

    let client_id_on_a = conn_end_on_a.client_id();
    let client_state_of_b_on_a = ctx_a.client_state(client_id_on_a)?;
    client_state_of_b_on_a.confirm_not_frozen()?;

    let conn_version = conn_end_on_a.versions();

    conn_version[0].verify_feature_supported(msg.ordering.to_string())?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    use crate::clients::ics07_tendermint::client_type as tm_client_type;
    use crate::core::ics02_client::height::Height;
    use crate::core::ics03_connection::connection::ConnectionEnd;
    use crate::core::ics03_connection::connection::State as ConnectionState;
    use crate::core::ics03_connection::msgs::conn_open_init::MsgConnectionOpenInit;
    use crate::core::ics03_connection::version::get_compatible_versions;
    use crate::core::ics04_channel::handler::chan_open_init::validate;
    use crate::core::ics04_channel::msgs::chan_open_init::test_util::get_dummy_raw_msg_chan_open_init;
    use crate::core::ics04_channel::msgs::chan_open_init::MsgChannelOpenInit;
    use crate::core::ics24_host::identifier::ClientId;
    use crate::core::ics24_host::identifier::ConnectionId;

    use crate::applications::transfer::MODULE_ID_STR;
    use crate::mock::context::MockContext;
    use crate::test_utils::DummyTransferModule;
    use test_log::test;

    pub struct Fixture {
        pub ctx: MockContext,
        pub module_id: ModuleId,
        pub msg: MsgChannelOpenInit,
    }

    #[fixture]
    fn fixture() -> Fixture {
        let msg = MsgChannelOpenInit::try_from(get_dummy_raw_msg_chan_open_init(None)).unwrap();

        let mut default_ctx = MockContext::default();
        let module_id: ModuleId = ModuleId::new(MODULE_ID_STR.to_string());
        let module = DummyTransferModule::new();
        default_ctx.add_route(module_id.clone(), module).unwrap();

        let msg_conn_init = MsgConnectionOpenInit::new_dummy();

        let client_id_on_a = ClientId::new(tm_client_type(), 0).unwrap();
        let client_height = Height::new(0, 10).unwrap();

        let conn_end_on_a = ConnectionEnd::new(
            ConnectionState::Init,
            msg_conn_init.client_id_on_a.clone(),
            msg_conn_init.counterparty.clone(),
            get_compatible_versions(),
            msg_conn_init.delay_period,
        )
        .unwrap();

        let ctx = default_ctx
            .with_client(&client_id_on_a, client_height)
            .with_connection(ConnectionId::default(), conn_end_on_a);

        Fixture {
            ctx,
            module_id,
            msg,
        }
    }

    #[rstest]
    fn chan_open_init_fail_no_connection(fixture: Fixture) {
        let Fixture { msg, .. } = fixture;

        let res = validate(&MockContext::default(), &msg);

        assert!(
            res.is_err(),
            "Validation fails because no connection exists in the context"
        )
    }

    #[rstest]
    fn chan_open_init_validate_happy_path(fixture: Fixture) {
        let Fixture { ctx, msg, .. } = fixture;

        let res = validate(&ctx, &msg);

        assert!(res.is_ok(), "Validation succeeds; good parameters")
    }

    #[rstest]
    fn chan_open_init_validate_counterparty_chan_id_set(fixture: Fixture) {
        let Fixture { ctx, .. } = fixture;

        let msg = MsgChannelOpenInit::try_from(get_dummy_raw_msg_chan_open_init(None)).unwrap();

        let res = validate(&ctx, &msg);

        assert!(
            res.is_ok(),
            "Validation succeeds even if counterparty channel id is set by relayer"
        )
    }

    #[rstest]
    fn chan_open_init_execute_happy_path(fixture: Fixture) {
        let Fixture {
            mut ctx,
            module_id,
            msg,
        } = fixture;

        let res = chan_open_init_execute(&mut ctx, module_id, msg);

        assert!(res.is_ok(), "Execution succeeds; good parameters");

        assert_eq!(ctx.events.len(), 2);
        assert!(matches!(
            ctx.events[0],
            IbcEvent::Message(MessageEvent::Channel)
        ));
        assert!(matches!(ctx.events[1], IbcEvent::OpenInitChannel(_)));
    }
}
