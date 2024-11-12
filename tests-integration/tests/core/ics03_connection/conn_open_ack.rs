use core::str::FromStr;

use ibc::core::client::types::Height;
use ibc::core::commitment_types::commitment::CommitmentPrefix;
use ibc::core::connection::types::error::ConnectionError;
use ibc::core::connection::types::msgs::{ConnectionMsg, MsgConnectionOpenAck};
use ibc::core::connection::types::{ConnectionEnd, Counterparty, State};
use ibc::core::entrypoint::{execute, validate};
use ibc::core::handler::types::error::HandlerError;
use ibc::core::handler::types::events::{IbcEvent, MessageEvent};
use ibc::core::handler::types::msgs::MsgEnvelope;
use ibc::core::host::types::identifiers::{ChainId, ClientId};
use ibc::core::host::ValidationContext;
use ibc::core::primitives::prelude::*;
use ibc::core::primitives::ZERO_DURATION;
use ibc_core_host_types::error::HostError;
use ibc_testkit::context::MockContext;
use ibc_testkit::fixtures::core::connection::dummy_msg_conn_open_ack;
use ibc_testkit::fixtures::core::context::dummy_store_generic_test_context;
use ibc_testkit::fixtures::{Expect, Fixture};
use ibc_testkit::hosts::MockHost;
use ibc_testkit::testapp::ibc::core::router::MockRouter;
use ibc_testkit::testapp::ibc::core::types::{DefaultIbcStore, LightClientState};
use test_log::test;

enum Ctx {
    New,
    NewWithConnection,
    NewWithConnectionEndOpen,
    DefaultWithConnection,
}

fn conn_open_ack_fixture(ctx: Ctx) -> Fixture<MsgConnectionOpenAck> {
    let msg = dummy_msg_conn_open_ack(10, 10);

    // Client parameters -- identifier and correct height (matching the proof height)
    let client_id = ClientId::from_str("mock_clientid").unwrap();
    let proof_height = msg.proofs_height_on_b;
    let conn_id = msg.conn_id_on_a.clone();

    // Parametrize the host chain to have a height at least as recent as the
    // the height of the proofs in the Ack msg.
    let latest_height = proof_height.increment();

    // A connection end that will exercise the successful path.
    let default_conn_end = ConnectionEnd::new(
        State::Init,
        client_id.clone(),
        Counterparty::new(
            client_id.clone(),
            Some(msg.conn_id_on_b.clone()),
            CommitmentPrefix::from(b"ibc".to_vec()),
        ),
        vec![msg.version.clone()],
        ZERO_DURATION,
    )
    .unwrap();

    // A connection end with incorrect state `Open`; will be part of the context.
    let mut conn_end_open = default_conn_end.clone();
    conn_end_open.set_state(State::Open); // incorrect field

    let ctx_default = MockContext::default();
    let ctx_new: MockContext = dummy_store_generic_test_context()
        .host(
            MockHost::builder()
                .chain_id(
                    ChainId::new(&format!("mockgaia-{}", latest_height.revision_number())).unwrap(),
                )
                .build(),
        )
        .latest_height(latest_height)
        .call();
    let ctx = match ctx {
        Ctx::New => ctx_new.ibc_store,
        Ctx::NewWithConnection => {
            ctx_new
                .with_light_client(
                    &client_id,
                    LightClientState::<MockHost>::with_latest_height(proof_height),
                )
                .with_connection(conn_id, default_conn_end)
                .ibc_store
        }
        Ctx::DefaultWithConnection => {
            ctx_default
                .with_light_client(
                    &client_id,
                    LightClientState::<MockHost>::with_latest_height(proof_height),
                )
                .with_connection(conn_id, default_conn_end)
                .ibc_store
        }
        Ctx::NewWithConnectionEndOpen => {
            ctx_new
                .with_light_client(
                    &client_id,
                    LightClientState::<MockHost>::with_latest_height(proof_height),
                )
                .with_connection(conn_id, conn_end_open)
                .ibc_store
        }
    };

    Fixture { ctx, msg }
}

fn conn_open_ack_validate(fxt: &Fixture<MsgConnectionOpenAck>, expect: Expect) {
    let router = MockRouter::new_with_transfer();
    let msg_envelope = MsgEnvelope::from(ConnectionMsg::from(fxt.msg.clone()));
    let res = validate(&fxt.ctx, &router, msg_envelope);
    let err_msg = fxt.generate_error_msg(&expect, "validation", &res);
    match expect {
        Expect::Failure(err) => {
            assert!(res.is_err(), "{err_msg}");
            assert_eq!(
                core::mem::discriminant(res.as_ref().unwrap_err()),
                core::mem::discriminant(&err.unwrap())
            );
        }
        Expect::Success => {
            assert!(res.is_ok(), "{err_msg}");
            return;
        }
    };
    let right_connection_id = fxt.msg.conn_id_on_a.clone();
    let cons_state_height = fxt.msg.consensus_height_of_a_on_b;

    match res.unwrap_err() {
        HandlerError::Connection(ConnectionError::Host(HostError::MissingState {
            ref description,
        })) => {
            assert!(description.contains(right_connection_id.to_string().as_str()))
        }
        HandlerError::Connection(ConnectionError::InsufficientConsensusHeight {
            target_height,
            current_height: _,
        }) => {
            assert_eq!(cons_state_height, target_height);
        }
        HandlerError::Connection(ConnectionError::MismatchedConnectionStates {
            expected: _,
            actual: _,
        }) => {}
        _ => unreachable!(),
    }
}

fn conn_open_ack_execute(fxt: &mut Fixture<MsgConnectionOpenAck>, expect: Expect) {
    let mut router = MockRouter::new_with_transfer();
    let msg_envelope = MsgEnvelope::from(ConnectionMsg::from(fxt.msg.clone()));
    let res = execute(&mut fxt.ctx, &mut router, msg_envelope);
    let err_msg = fxt.generate_error_msg(&expect, "execution", &res);
    match expect {
        Expect::Failure(_) => {
            assert!(res.is_err(), "{err_msg}");
        }
        Expect::Success => {
            let ibc_events = fxt.ctx.events.lock();
            assert!(res.is_ok(), "{err_msg}");
            assert_eq!(ibc_events.len(), 2);

            assert!(matches!(
                ibc_events[0],
                IbcEvent::Message(MessageEvent::Connection)
            ));
            let event = &ibc_events[1];
            assert!(matches!(event, &IbcEvent::OpenAckConnection(_)));

            let IbcEvent::OpenAckConnection(conn_open_try_event) = event else {
                unreachable!()
            };
            let conn_end = <DefaultIbcStore as ValidationContext>::connection_end(
                &fxt.ctx,
                conn_open_try_event.conn_id_on_a(),
            )
            .unwrap();
            assert_eq!(conn_end.state().clone(), State::Open);
        }
    }
}

#[test]
fn conn_open_ack_healthy() {
    let mut fxt = conn_open_ack_fixture(Ctx::NewWithConnection);
    conn_open_ack_validate(&fxt, Expect::Success);
    conn_open_ack_execute(&mut fxt, Expect::Success);
}

#[test]
fn conn_open_ack_no_connection() {
    let fxt = conn_open_ack_fixture(Ctx::New);
    let expected_err: HandlerError = ConnectionError::Host(HostError::missing_state(format!(
        "missing connection end for connection {}",
        fxt.msg.conn_id_on_a.clone()
    )))
    .into();
    conn_open_ack_validate(&fxt, Expect::Failure(Some(expected_err)));
}

#[test]
fn conn_open_ack_invalid_consensus_height() {
    let fxt = conn_open_ack_fixture(Ctx::DefaultWithConnection);
    let expected_err = HandlerError::Connection(ConnectionError::InsufficientConsensusHeight {
        target_height: fxt.msg.consensus_height_of_a_on_b,
        current_height: Height::new(0, 10).unwrap(),
    });
    conn_open_ack_validate(&fxt, Expect::Failure(Some(expected_err)));
}

#[test]
fn conn_open_ack_connection_mismatch() {
    let fxt = conn_open_ack_fixture(Ctx::NewWithConnectionEndOpen);
    let expected_err = HandlerError::Connection(ConnectionError::MismatchedConnectionStates {
        expected: State::Init.to_string(),
        actual: State::Open.to_string(),
    });
    conn_open_ack_validate(&fxt, Expect::Failure(Some(expected_err)));
}
