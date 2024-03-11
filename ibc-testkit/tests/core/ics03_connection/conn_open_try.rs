use ibc::core::client::types::Height;
use ibc::core::connection::types::msgs::{ConnectionMsg, MsgConnectionOpenTry};
use ibc::core::connection::types::State;
use ibc::core::entrypoint::{execute, validate};
use ibc::core::handler::types::events::{IbcEvent, MessageEvent};
use ibc::core::handler::types::msgs::MsgEnvelope;
use ibc::core::host::ValidationContext;
use ibc::core::primitives::prelude::*;
use ibc_testkit::fixtures::core::connection::dummy_msg_conn_open_try;
use ibc_testkit::fixtures::core::context::MockContextConfig;
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
    HeightAdvanced,
    HeightOld,
    ProofHeightMissing,
}

fn conn_open_try_fixture(ctx_variant: Ctx, msg_variant: Msg) -> Fixture<MsgConnectionOpenTry> {
    let max_history_size = 5;
    let client_cons_state_height = 10;
    let host_chain_height = Height::new(0, 35).unwrap();
    let pruned_height = host_chain_height
        .sub(max_history_size + 1)
        .unwrap()
        .revision_height();

    let msg = match msg_variant {
        Msg::Default => dummy_msg_conn_open_try(
            client_cons_state_height,
            host_chain_height.revision_height(),
        ),
        Msg::HeightAdvanced => dummy_msg_conn_open_try(
            client_cons_state_height,
            host_chain_height.increment().revision_height(),
        ),
        Msg::HeightOld => dummy_msg_conn_open_try(client_cons_state_height, pruned_height),
        Msg::ProofHeightMissing => dummy_msg_conn_open_try(
            client_cons_state_height - 1,
            host_chain_height.revision_height(),
        ),
    };

    let ctx_new = MockContextConfig::builder()
        .max_history_size(max_history_size)
        .latest_height(host_chain_height)
        .build();
    let ctx = match ctx_variant {
        Ctx::Default => MockContext::default(),
        Ctx::WithClient => ctx_new.with_client_config(
            MockClientConfig::builder()
                .client_id(msg.client_id_on_b.clone())
                .latest_height(Height::new(0, client_cons_state_height).unwrap())
                .build(),
        ),
    };
    Fixture { ctx, msg }
}

fn conn_open_try_validate(fxt: &Fixture<MsgConnectionOpenTry>, expect: Expect) {
    let router = MockRouter::new_with_transfer();
    let msg_envelope = MsgEnvelope::from(ConnectionMsg::from(fxt.msg.clone()));
    let res = validate(&fxt.ctx, &router, msg_envelope);
    let err_msg = fxt.generate_error_msg(&expect, "validation", &res);
    match expect {
        Expect::Failure(_) => {
            assert!(res.is_err(), "{err_msg}")
        }
        Expect::Success => {
            assert!(res.is_ok(), "{err_msg}");
        }
    }
}

fn conn_open_try_execute(fxt: &mut Fixture<MsgConnectionOpenTry>, expect: Expect) {
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

            let ibc_events = fxt.ctx.get_events();

            assert_eq!(ibc_events.len(), 2);

            assert!(matches!(
                ibc_events[0],
                IbcEvent::Message(MessageEvent::Connection)
            ));
            let event = &ibc_events[1];
            assert!(matches!(event, &IbcEvent::OpenTryConnection(_)));

            let IbcEvent::OpenTryConnection(conn_open_try_event) = event else {
                unreachable!()
            };
            let conn_end =
                ValidationContext::connection_end(&fxt.ctx, conn_open_try_event.conn_id_on_b())
                    .unwrap();
            assert_eq!(conn_end.state().clone(), State::TryOpen);
        }
    }
}

#[test]
fn conn_open_try_healthy() {
    let mut fxt = conn_open_try_fixture(Ctx::WithClient, Msg::Default);
    conn_open_try_validate(&fxt, Expect::Success);
    conn_open_try_execute(&mut fxt, Expect::Success);
}

#[test]
fn conn_open_try_height_advanced() {
    let fxt = conn_open_try_fixture(Ctx::WithClient, Msg::HeightAdvanced);
    conn_open_try_validate(&fxt, Expect::Failure(None));
}

#[test]
fn conn_open_try_height_old() {
    let fxt = conn_open_try_fixture(Ctx::WithClient, Msg::HeightOld);
    conn_open_try_validate(&fxt, Expect::Failure(None));
}

#[test]
fn conn_open_try_proof_height_missing() {
    let fxt = conn_open_try_fixture(Ctx::WithClient, Msg::ProofHeightMissing);
    conn_open_try_validate(&fxt, Expect::Failure(None));
}

#[test]
fn conn_open_try_no_client() {
    let fxt = conn_open_try_fixture(Ctx::Default, Msg::Default);
    conn_open_try_validate(&fxt, Expect::Failure(None));
}
