use ibc::core::events::{IbcEvent, MessageEvent};
use ibc::core::ics03_connection::connection::State;
use ibc::core::ics03_connection::msgs::conn_open_init::MsgConnectionOpenInit;
use ibc::core::ics03_connection::msgs::ConnectionMsg;
use ibc::core::ics03_connection::version::Version;
use ibc::core::{execute, validate, MsgEnvelope, ValidationContext};
use ibc::prelude::*;
use ibc::Height;
use ibc_testkit::testapp::ibc::core::router::MockRouter;
use ibc_testkit::testapp::ibc::core::types::MockContext;
use ibc_testkit::utils::fixture::{Expect, Fixture};
use test_log::test;

enum Ctx {
    Default,
    WithClient,
}

enum Msg {
    Default,
    NoVersion,
    BadVersion,
    WithCounterpartyConnId,
}

fn conn_open_init_fixture(ctx_variant: Ctx, msg_variant: Msg) -> Fixture<MsgConnectionOpenInit> {
    let msg_default = MsgConnectionOpenInit::new_dummy();
    let msg = match msg_variant {
        Msg::Default => msg_default,
        Msg::NoVersion => msg_default.with_version(None),
        Msg::BadVersion => msg_default.with_version(Some("random identifier 424242")),
        Msg::WithCounterpartyConnId => msg_default.with_counterparty_conn_id(2),
    };

    let ctx_default = MockContext::default();
    let ctx = match ctx_variant {
        Ctx::WithClient => {
            ctx_default.with_client(&msg.client_id_on_a, Height::new(0, 10).unwrap())
        }
        _ => ctx_default,
    };

    Fixture { ctx, msg }
}

fn conn_open_init_validate(fxt: &Fixture<MsgConnectionOpenInit>, expect: Expect) {
    let router = MockRouter::new_with_transfer();
    let msg_envelope = MsgEnvelope::from(ConnectionMsg::from(fxt.msg.clone()));
    let res = validate(&fxt.ctx, &router, msg_envelope);
    let err_msg = fxt.generate_error_msg(&expect, "validation", &res);
    match expect {
        Expect::Failure(_) => {
            assert!(res.is_err(), "{err_msg}")
        }
        Expect::Success => {
            assert!(res.is_ok(), "{err_msg}")
        }
    }
}

fn conn_open_init_execute(
    fxt: &mut Fixture<MsgConnectionOpenInit>,
    expect: Expect,
    expected_version: Vec<Version>,
) {
    let mut router = MockRouter::new_with_transfer();
    let msg_envelope = MsgEnvelope::from(ConnectionMsg::from(fxt.msg.clone()));
    let res = execute(&mut fxt.ctx, &mut router, msg_envelope);
    let err_msg = fxt.generate_error_msg(&expect, "execution", &res);
    match expect {
        Expect::Failure(_) => {
            assert!(res.is_err(), "{err_msg}")
        }
        Expect::Success => {
            assert!(res.is_ok(), "{err_msg}");

            assert_eq!(fxt.ctx.connection_counter().unwrap(), 1);

            assert_eq!(fxt.ctx.events.len(), 2);

            assert!(matches!(
                fxt.ctx.events[0],
                IbcEvent::Message(MessageEvent::Connection)
            ));
            let event = &fxt.ctx.events[1];
            assert!(matches!(event, &IbcEvent::OpenInitConnection(_)));

            let conn_open_init_event = match event {
                IbcEvent::OpenInitConnection(e) => e,
                _ => unreachable!(),
            };
            let conn_end =
                ValidationContext::connection_end(&fxt.ctx, conn_open_init_event.conn_id_on_a())
                    .unwrap();
            assert_eq!(conn_end.state().clone(), State::Init);
            assert_eq!(conn_end.versions(), expected_version);
        }
    }
}

#[test]
fn conn_open_init_healthy() {
    let mut fxt = conn_open_init_fixture(Ctx::WithClient, Msg::Default);
    conn_open_init_validate(&fxt, Expect::Success);
    let expected_version = vec![fxt.msg.version.clone().unwrap()];
    conn_open_init_execute(&mut fxt, Expect::Success, expected_version);
}

#[test]
fn conn_open_init_no_context() {
    let fxt = conn_open_init_fixture(Ctx::Default, Msg::Default);
    conn_open_init_validate(&fxt, Expect::Failure(None));
}

#[test]
fn conn_open_init_no_version() {
    let mut fxt = conn_open_init_fixture(Ctx::WithClient, Msg::NoVersion);
    conn_open_init_validate(&fxt, Expect::Success);
    let expected_version = ValidationContext::get_compatible_versions(&fxt.ctx.clone());
    conn_open_init_execute(&mut fxt, Expect::Success, expected_version);
}
#[test]
fn conn_open_init_incompatible_version() {
    let fxt = conn_open_init_fixture(Ctx::WithClient, Msg::BadVersion);
    conn_open_init_validate(&fxt, Expect::Failure(None));
}

#[test]
fn conn_open_init_with_counterparty_conn_id() {
    let mut fxt = conn_open_init_fixture(Ctx::WithClient, Msg::WithCounterpartyConnId);
    conn_open_init_validate(&fxt, Expect::Success);
    let expected_version = vec![fxt.msg.version.clone().unwrap()];
    conn_open_init_execute(&mut fxt, Expect::Success, expected_version);
}
