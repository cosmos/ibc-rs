use ibc::core::client::types::Height;
use ibc::core::connection::types::msgs::{ConnectionMsg, MsgConnectionOpenInit};
use ibc::core::connection::types::version::Version;
use ibc::core::connection::types::State;
use ibc::core::entrypoint::{execute, validate};
use ibc::core::handler::types::events::{IbcEvent, MessageEvent};
use ibc::core::handler::types::msgs::MsgEnvelope;
use ibc::core::host::ValidationContext;
use ibc::core::primitives::prelude::*;
use ibc_testkit::fixtures::core::connection::{
    dummy_msg_conn_open_init, msg_conn_open_init_with_counterparty_conn_id,
    msg_conn_open_with_version,
};
use ibc_testkit::fixtures::{Expect, Fixture};
use ibc_testkit::testapp::ibc::core::router::MockRouter;
use ibc_testkit::testapp::ibc::core::types::{MockClientConfig, MockContext};
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
    let msg_default = dummy_msg_conn_open_init();
    let msg = match msg_variant {
        Msg::Default => msg_default.clone(),
        Msg::NoVersion => msg_conn_open_with_version(msg_default, None),
        Msg::BadVersion => {
            msg_conn_open_with_version(msg_default, Some("random identifier 424242"))
        }
        Msg::WithCounterpartyConnId => msg_conn_open_init_with_counterparty_conn_id(msg_default, 2),
    };

    let ctx_default = MockContext::default();
    let ctx = match ctx_variant {
        Ctx::WithClient => ctx_default.with_client_config(
            MockClientConfig::builder()
                .client_id(msg.client_id_on_a.clone())
                .latest_height(Height::new(0, 10).unwrap())
                .build(),
        ),
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
            let ibc_events = fxt.ctx.get_events();

            assert!(res.is_ok(), "{err_msg}");

            assert_eq!(fxt.ctx.connection_counter().unwrap(), 1);

            assert_eq!(ibc_events.len(), 2);

            assert!(matches!(
                ibc_events[0],
                IbcEvent::Message(MessageEvent::Connection)
            ));
            let event = &ibc_events[1];
            assert!(matches!(event, &IbcEvent::OpenInitConnection(_)));

            let IbcEvent::OpenInitConnection(conn_open_init_event) = event else {
                unreachable!()
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
