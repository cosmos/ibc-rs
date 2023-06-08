//! Protocol logic specific to ICS4 messages of type `MsgChannelOpenAck`.

use crate::prelude::*;
use ibc_proto::protobuf::Protobuf;

use crate::core::events::{IbcEvent, MessageEvent};
use crate::core::ics03_connection::connection::State as ConnectionState;
use crate::core::ics04_channel::channel::State;
use crate::core::ics04_channel::channel::{ChannelEnd, Counterparty, State as ChannelState};
use crate::core::ics04_channel::error::ChannelError;
use crate::core::ics04_channel::events::OpenAck;
use crate::core::ics04_channel::msgs::chan_open_ack::MsgChannelOpenAck;
use crate::core::ics24_host::path::Path;
use crate::core::ics24_host::path::{ChannelEndPath, ClientConsensusStatePath};
use crate::core::module::ModuleId;
use crate::core::router::{RouterMut, RouterRef};
use crate::core::{ContextError, ExecutionContext, ValidationContext};

pub(crate) fn chan_open_ack_validate<ValCtx>(
    ctx_a: &ValCtx,
    module_id: ModuleId,
    msg: MsgChannelOpenAck,
) -> Result<(), ContextError>
where
    ValCtx: ValidationContext,
{
    validate(ctx_a, &msg)?;

    let module = ctx_a
        .get_route(&module_id)
        .ok_or(ChannelError::RouteNotFound)?;
    module.on_chan_open_ack_validate(&msg.port_id_on_a, &msg.chan_id_on_a, &msg.version_on_b)?;

    Ok(())
}

pub(crate) fn chan_open_ack_execute<ExecCtx>(
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

fn validate<Ctx>(ctx_a: &Ctx, msg: &MsgChannelOpenAck) -> Result<(), ContextError>
where
    Ctx: ValidationContext,
{
    ctx_a.validate_message_signer(&msg.signer)?;

    let chan_end_path_on_a = ChannelEndPath::new(&msg.port_id_on_a, &msg.chan_id_on_a);
    let chan_end_on_a = ctx_a.channel_end(&chan_end_path_on_a)?;

    // Validate that the channel end is in a state where it can be ack.
    chan_end_on_a.verify_state_matches(&ChannelState::Init)?;

    // An OPEN IBC connection running on the local (host) chain should exist.
    chan_end_on_a.verify_connection_hops_length()?;

    let conn_end_on_a = ctx_a.connection_end(&chan_end_on_a.connection_hops()[0])?;

    conn_end_on_a.verify_state_matches(&ConnectionState::Open)?;

    // Verify proofs
    {
        let client_id_on_a = conn_end_on_a.client_id();
        let client_state_of_b_on_a = ctx_a.client_state(client_id_on_a)?;

        client_state_of_b_on_a.confirm_not_frozen()?;
        client_state_of_b_on_a.validate_proof_height(msg.proof_height_on_b)?;

        let client_cons_state_path_on_a =
            ClientConsensusStatePath::new(client_id_on_a, &msg.proof_height_on_b);
        let consensus_state_of_b_on_a = ctx_a.consensus_state(&client_cons_state_path_on_a)?;
        let prefix_on_b = conn_end_on_a.counterparty().prefix();
        let port_id_on_b = &chan_end_on_a.counterparty().port_id;
        let conn_id_on_b = conn_end_on_a.counterparty().connection_id().ok_or(
            ChannelError::UndefinedConnectionCounterparty {
                connection_id: chan_end_on_a.connection_hops()[0].clone(),
            },
        )?;

        let expected_chan_end_on_b = ChannelEnd::new(
            ChannelState::TryOpen,
            // Note: Both ends of a channel must have the same ordering, so it's
            // fine to use A's ordering here
            *chan_end_on_a.ordering(),
            Counterparty::new(msg.port_id_on_a.clone(), Some(msg.chan_id_on_a.clone())),
            vec![conn_id_on_b.clone()],
            msg.version_on_b.clone(),
        )?;
        let chan_end_path_on_b = ChannelEndPath::new(port_id_on_b, &msg.chan_id_on_b);

        // Verify the proof for the channel state against the expected channel end.
        // A counterparty channel id of None in not possible, and is checked by validate_basic in msg.
        client_state_of_b_on_a
            .verify_membership(
                prefix_on_b,
                &msg.proof_chan_end_on_b,
                consensus_state_of_b_on_a.root(),
                Path::ChannelEnd(chan_end_path_on_b),
                expected_chan_end_on_b.encode_vec(),
            )
            .map_err(ChannelError::VerifyChannelFailed)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {

    use super::*;
    use rstest::*;
    use test_log::test;

    use crate::applications::transfer::MODULE_ID_STR;
    use crate::core::ics03_connection::connection::ConnectionEnd;
    use crate::core::ics03_connection::connection::Counterparty as ConnectionCounterparty;
    use crate::core::ics03_connection::connection::State as ConnectionState;
    use crate::core::ics03_connection::msgs::test_util::get_dummy_raw_counterparty;
    use crate::core::ics03_connection::version::get_compatible_versions;
    use crate::core::ics04_channel::channel::Order;
    use crate::core::ics04_channel::channel::{ChannelEnd, Counterparty, State};
    use crate::core::ics04_channel::msgs::chan_open_ack::test_util::get_dummy_raw_msg_chan_open_ack;
    use crate::core::ics04_channel::msgs::chan_open_ack::MsgChannelOpenAck;
    use crate::core::ics24_host::identifier::ClientId;
    use crate::core::ics24_host::identifier::ConnectionId;
    use crate::core::timestamp::ZERO_DURATION;
    use crate::Height;

    use crate::mock::client_state::client_type as mock_client_type;
    use crate::mock::context::MockContext;
    use crate::test_utils::DummyTransferModule;

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
        let module = DummyTransferModule::default();
        let module_id: ModuleId = ModuleId::new(MODULE_ID_STR.to_string());
        context
            .add_route(module_id.clone(), Box::new(module))
            .unwrap();

        let client_id_on_a = ClientId::new(mock_client_type(), 45).unwrap();
        let conn_id_on_a = ConnectionId::new(2);
        let conn_end_on_a = ConnectionEnd::new(
            ConnectionState::Open,
            client_id_on_a.clone(),
            ConnectionCounterparty::try_from(get_dummy_raw_counterparty(Some(0))).unwrap(),
            get_compatible_versions(),
            ZERO_DURATION,
        )
        .unwrap();

        let msg =
            MsgChannelOpenAck::try_from(get_dummy_raw_msg_chan_open_ack(proof_height)).unwrap();

        let chan_end_on_a = ChannelEnd::new(
            State::Init,
            Order::Unordered,
            Counterparty::new(msg.port_id_on_a.clone(), Some(msg.chan_id_on_b.clone())),
            vec![conn_id_on_a.clone()],
            msg.version_on_b.clone(),
        )
        .unwrap();

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
    fn chan_open_ack_fail_no_channel(fixture: Fixture) {
        let Fixture {
            context,
            msg,
            client_id_on_a,
            conn_id_on_a,
            conn_end_on_a,
            proof_height,
            ..
        } = fixture;
        let context = context
            .with_client(&client_id_on_a, Height::new(0, proof_height).unwrap())
            .with_connection(conn_id_on_a, conn_end_on_a);

        let res = validate(&context, &msg);

        assert!(
            res.is_err(),
            "Validation fails because no channel exists in the context"
        )
    }

    #[rstest]
    fn chan_open_ack_fail_channel_wrong_state(fixture: Fixture) {
        let Fixture {
            context,
            msg,
            client_id_on_a,
            conn_id_on_a,
            conn_end_on_a,
            proof_height,
            ..
        } = fixture;

        let wrong_chan_end = ChannelEnd::new(
            State::Open,
            Order::Unordered,
            Counterparty::new(msg.port_id_on_a.clone(), Some(msg.chan_id_on_b.clone())),
            vec![conn_id_on_a.clone()],
            msg.version_on_b.clone(),
        )
        .unwrap();
        let context = context
            .with_client(&client_id_on_a, Height::new(0, proof_height).unwrap())
            .with_connection(conn_id_on_a, conn_end_on_a)
            .with_channel(
                msg.port_id_on_a.clone(),
                msg.chan_id_on_a.clone(),
                wrong_chan_end,
            );

        let res = validate(&context, &msg);

        assert!(
            res.is_err(),
            "Validation fails because channel is in the wrong state"
        )
    }

    #[rstest]
    fn chan_open_ack_fail_no_connection(fixture: Fixture) {
        let Fixture {
            context,
            msg,
            client_id_on_a,
            chan_end_on_a,
            proof_height,
            ..
        } = fixture;

        let context = context
            .with_client(&client_id_on_a, Height::new(0, proof_height).unwrap())
            .with_channel(
                msg.port_id_on_a.clone(),
                msg.chan_id_on_a.clone(),
                chan_end_on_a,
            );

        let res = validate(&context, &msg);

        assert!(
            res.is_err(),
            "Validation fails because no connection exists in the context"
        )
    }

    #[rstest]
    fn chan_open_ack_happy_path(fixture: Fixture) {
        let Fixture {
            context,
            msg,
            client_id_on_a,
            conn_id_on_a,
            conn_end_on_a,
            chan_end_on_a,
            proof_height,
            ..
        } = fixture;

        let context = context
            .with_client(&client_id_on_a, Height::new(0, proof_height).unwrap())
            .with_connection(conn_id_on_a, conn_end_on_a)
            .with_channel(
                msg.port_id_on_a.clone(),
                msg.chan_id_on_a.clone(),
                chan_end_on_a,
            );

        let res = validate(&context, &msg);

        assert!(res.is_ok(), "Validation happy path")
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
            IbcEvent::Message(MessageEvent::Channel)
        ));
        assert!(matches!(context.events[1], IbcEvent::OpenAckChannel(_)));
    }
}
