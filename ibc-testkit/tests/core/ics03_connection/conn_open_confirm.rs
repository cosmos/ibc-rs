use core::str::FromStr;

use ibc::core::client::types::Height;
use ibc::core::commitment_types::commitment::CommitmentPrefix;
use ibc::core::connection::types::msgs::{ConnectionMsg, MsgConnectionOpenConfirm};
use ibc::core::connection::types::{ConnectionEnd, Counterparty, State};
use ibc::core::entrypoint::{execute, validate};
use ibc::core::handler::types::events::{IbcEvent, MessageEvent};
use ibc::core::handler::types::msgs::MsgEnvelope;
use ibc::core::host::types::identifiers::ClientId;
use ibc::core::host::ValidationContext;
use ibc::core::primitives::prelude::*;
use ibc::core::primitives::ZERO_DURATION;
use ibc_testkit::fixtures::core::connection::dummy_conn_open_confirm;
use ibc_testkit::fixtures::{Expect, Fixture};
use ibc_testkit::testapp::ibc::core::router::MockRouter;
use ibc_testkit::testapp::ibc::core::types::{MockClientConfig, MockContext};
use test_log::test;

enum Ctx {
    Default,
    CorrectConnection,
    IncorrectConnection,
}

fn conn_open_confirm_fixture(ctx: Ctx) -> Fixture<MsgConnectionOpenConfirm> {
    let client_id = ClientId::from_str("mock_clientid").unwrap();
    let msg = dummy_conn_open_confirm();
    let counterparty = Counterparty::new(
        client_id.clone(),
        Some(msg.conn_id_on_b.clone()),
        CommitmentPrefix::try_from(b"ibc".to_vec()).unwrap(),
    );

    let ctx_default = MockContext::default();

    let incorrect_conn_end_state = ConnectionEnd::new(
        State::Init,
        client_id.clone(),
        counterparty,
        ValidationContext::get_compatible_versions(&ctx_default),
        ZERO_DURATION,
    )
    .unwrap();

    let mut correct_conn_end = incorrect_conn_end_state.clone();
    correct_conn_end.set_state(State::TryOpen);

    let ctx = match ctx {
        Ctx::Default => ctx_default,
        Ctx::IncorrectConnection => ctx_default
            .with_client_config(
                MockClientConfig::builder()
                    .client_id(client_id.clone())
                    .latest_height(Height::new(0, 10).unwrap())
                    .build(),
            )
            .with_connection(msg.conn_id_on_b.clone(), incorrect_conn_end_state),
        Ctx::CorrectConnection => ctx_default
            .with_client_config(
                MockClientConfig::builder()
                    .client_id(client_id.clone())
                    .latest_height(Height::new(0, 10).unwrap())
                    .build(),
            )
            .with_connection(msg.conn_id_on_b.clone(), correct_conn_end),
    };

    Fixture { ctx, msg }
}

fn conn_open_confirm_validate(fxt: &Fixture<MsgConnectionOpenConfirm>, expect: Expect) {
    let router = MockRouter::new_with_transfer();
    let msg_envelope = MsgEnvelope::from(ConnectionMsg::from(fxt.msg.clone()));
    let res = validate(&fxt.ctx, &router, msg_envelope);
    let err_msg = fxt.generate_error_msg(&expect, "validation", &res);
    match expect {
        Expect::Failure(_) => {
            assert!(res.is_err(), "{err_msg}");
        }
        Expect::Success => {
            assert!(res.is_ok(), "{err_msg}");
        }
    };
}

fn conn_open_confirm_execute(fxt: &mut Fixture<MsgConnectionOpenConfirm>, expect: Expect) {
    let mut router = MockRouter::new_with_transfer();
    let msg_envelope = MsgEnvelope::from(ConnectionMsg::from(fxt.msg.clone()));
    let res = execute(&mut fxt.ctx, &mut router, msg_envelope);
    let err_msg = fxt.generate_error_msg(&expect, "execution", &res);
    match expect {
        Expect::Failure(_) => {
            assert!(res.is_err(), "{err_msg}");
        }
        Expect::Success => {
            let ibc_events = fxt.ctx.get_events();
            assert!(res.is_ok(), "{err_msg}");
            assert_eq!(ibc_events.len(), 2);

            assert!(matches!(
                ibc_events[0],
                IbcEvent::Message(MessageEvent::Connection)
            ));
            let event = &ibc_events[1];
            assert!(matches!(event, &IbcEvent::OpenConfirmConnection(_)));

            let IbcEvent::OpenConfirmConnection(conn_open_try_event) = event else {
                unreachable!()
            };
            let conn_end =
                ValidationContext::connection_end(&fxt.ctx, conn_open_try_event.conn_id_on_b())
                    .unwrap();
            assert_eq!(conn_end.state().clone(), State::Open);
        }
    }
}

#[test]
fn conn_open_confirm_healthy() {
    let mut fxt = conn_open_confirm_fixture(Ctx::CorrectConnection);
    conn_open_confirm_validate(&fxt, Expect::Success);
    conn_open_confirm_execute(&mut fxt, Expect::Success);
}

#[test]
fn conn_open_confirm_no_connection() {
    let fxt = conn_open_confirm_fixture(Ctx::Default);
    conn_open_confirm_validate(&fxt, Expect::Failure(None));
}

#[test]
fn conn_open_confirm_connection_mismatch() {
    let fxt = conn_open_confirm_fixture(Ctx::IncorrectConnection);
    conn_open_confirm_validate(&fxt, Expect::Failure(None));
}
