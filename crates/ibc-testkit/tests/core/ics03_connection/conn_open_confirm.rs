use core::str::FromStr;

use ibc::core::events::{IbcEvent, MessageEvent};
use ibc::core::ics03_connection::connection::{ConnectionEnd, Counterparty, State};
use ibc::core::ics03_connection::msgs::conn_open_confirm::MsgConnectionOpenConfirm;
use ibc::core::ics03_connection::msgs::ConnectionMsg;
use ibc::core::ics23_commitment::commitment::CommitmentPrefix;
use ibc::core::ics24_host::identifier::ClientId;
use ibc::core::timestamp::ZERO_DURATION;
use ibc::core::{execute, validate, MsgEnvelope, ValidationContext};
use ibc::Height;
use ibc_testkit::testapp::ibc::core::router::MockRouter;
use ibc_testkit::testapp::ibc::core::types::MockContext;
use ibc_testkit::utils::dummies::core::connection::dummy_conn_open_confirm;
use ibc_testkit::utils::fixture::{Expect, Fixture};
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
            .with_client(&client_id, Height::new(0, 10).unwrap())
            .with_connection(msg.conn_id_on_b.clone(), incorrect_conn_end_state),
        Ctx::CorrectConnection => ctx_default
            .with_client(&client_id, Height::new(0, 10).unwrap())
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
            assert!(res.is_ok(), "{err_msg}");
            assert_eq!(fxt.ctx.events.len(), 2);

            assert!(matches!(
                fxt.ctx.events[0],
                IbcEvent::Message(MessageEvent::Connection)
            ));
            let event = &fxt.ctx.events[1];
            assert!(matches!(event, &IbcEvent::OpenConfirmConnection(_)));

            let conn_open_try_event = match event {
                IbcEvent::OpenConfirmConnection(e) => e,
                _ => unreachable!(),
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
