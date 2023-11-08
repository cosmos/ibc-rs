use ibc::core::events::{IbcEvent, MessageEvent};
use ibc::core::ics03_connection::connection::State;
use ibc::core::ics03_connection::msgs::conn_open_try::MsgConnectionOpenTry;
use ibc::core::ics03_connection::msgs::ConnectionMsg;
use ibc::core::ics24_host::identifier::ChainId;
use ibc::core::{execute, validate, MsgEnvelope, ValidationContext};
use ibc::Height;
use ibc_mocks::core::definition::MockContext;
use ibc_mocks::host::block::HostType;
use ibc_mocks::router::definition::MockRouter;
use ibc_mocks::utils::fixture::{Expect, Fixture};
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
        Msg::Default => MsgConnectionOpenTry::new_dummy(
            client_cons_state_height,
            host_chain_height.revision_height(),
        ),
        Msg::HeightAdvanced => MsgConnectionOpenTry::new_dummy(
            client_cons_state_height,
            host_chain_height.increment().revision_height(),
        ),
        Msg::HeightOld => MsgConnectionOpenTry::new_dummy(client_cons_state_height, pruned_height),
        Msg::ProofHeightMissing => MsgConnectionOpenTry::new_dummy(
            client_cons_state_height - 1,
            host_chain_height.revision_height(),
        ),
    };

    let ctx_new = MockContext::new(
        ChainId::new("mockgaia-0").unwrap(),
        HostType::Mock,
        max_history_size,
        host_chain_height,
    );
    let ctx = match ctx_variant {
        Ctx::Default => MockContext::default(),
        Ctx::WithClient => ctx_new.with_client(
            &msg.client_id_on_b,
            Height::new(0, client_cons_state_height).unwrap(),
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

            assert_eq!(fxt.ctx.events.len(), 2);

            assert!(matches!(
                fxt.ctx.events[0],
                IbcEvent::Message(MessageEvent::Connection)
            ));
            let event = &fxt.ctx.events[1];
            assert!(matches!(event, &IbcEvent::OpenTryConnection(_)));

            let conn_open_try_event = match event {
                IbcEvent::OpenTryConnection(e) => e,
                _ => unreachable!(),
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
