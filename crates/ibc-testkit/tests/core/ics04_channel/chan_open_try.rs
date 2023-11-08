use ibc::core::events::{IbcEvent, MessageEvent};
use ibc::core::ics03_connection::connection::{
    ConnectionEnd, Counterparty as ConnectionCounterparty, State as ConnectionState,
};
use ibc::core::ics03_connection::msgs::test_util::get_dummy_raw_counterparty;
use ibc::core::ics03_connection::version::get_compatible_versions;
use ibc::core::ics04_channel::msgs::chan_open_try::test_util::get_dummy_raw_msg_chan_open_try;
use ibc::core::ics04_channel::msgs::chan_open_try::MsgChannelOpenTry;
use ibc::core::ics04_channel::msgs::ChannelMsg;
use ibc::core::ics24_host::identifier::{ClientId, ConnectionId};
use ibc::core::timestamp::ZERO_DURATION;
use ibc::core::{execute, validate, MsgEnvelope, ValidationContext};
use ibc::mock::client_state::client_type as mock_client_type;
use ibc::prelude::*;
use ibc::Height;
use ibc_testkit::testapp::ibc::core::router::MockRouter;
use ibc_testkit::testapp::ibc::core::types::MockContext;
use rstest::*;
use test_log::test;

pub struct Fixture {
    pub ctx: MockContext,
    pub router: MockRouter,
    pub msg: MsgEnvelope,
    pub client_id_on_b: ClientId,
    pub conn_id_on_b: ConnectionId,
    pub conn_end_on_b: ConnectionEnd,
    pub proof_height: u64,
}

#[fixture]
fn fixture() -> Fixture {
    let proof_height = 10;
    let conn_id_on_b = ConnectionId::new(2);
    let client_id_on_b = ClientId::new(mock_client_type(), 45).unwrap();

    // This is the connection underlying the channel we're trying to open.
    let conn_end_on_b = ConnectionEnd::new(
        ConnectionState::Open,
        client_id_on_b.clone(),
        ConnectionCounterparty::try_from(get_dummy_raw_counterparty(Some(0))).unwrap(),
        get_compatible_versions(),
        ZERO_DURATION,
    )
    .unwrap();

    // We're going to test message processing against this message.
    // Note: we make the counterparty's channel_id `None`.
    let mut msg_chan_open_try =
        MsgChannelOpenTry::try_from(get_dummy_raw_msg_chan_open_try(proof_height)).unwrap();

    let hops = vec![conn_id_on_b.clone()];
    msg_chan_open_try.connection_hops_on_b = hops;

    let msg = MsgEnvelope::from(ChannelMsg::from(msg_chan_open_try));

    let ctx = MockContext::default();

    let router = MockRouter::new_with_transfer();

    Fixture {
        ctx,
        router,
        msg,
        client_id_on_b,
        conn_id_on_b,
        conn_end_on_b,
        proof_height,
    }
}

#[rstest]
fn chan_open_try_validate_happy_path(fixture: Fixture) {
    let Fixture {
        ctx,
        router,
        msg,
        client_id_on_b,
        conn_id_on_b,
        conn_end_on_b,
        proof_height,
        ..
    } = fixture;

    let ctx = ctx
        .with_client(&client_id_on_b, Height::new(0, proof_height).unwrap())
        .with_connection(conn_id_on_b, conn_end_on_b);

    let res = validate(&ctx, &router, msg);

    assert!(res.is_ok(), "Validation success: happy path")
}

#[rstest]
fn chan_open_try_execute_happy_path(fixture: Fixture) {
    let Fixture {
        ctx,
        mut router,
        msg,
        client_id_on_b,
        conn_id_on_b,
        conn_end_on_b,
        proof_height,
        ..
    } = fixture;

    let mut ctx = ctx
        .with_client(&client_id_on_b, Height::new(0, proof_height).unwrap())
        .with_connection(conn_id_on_b, conn_end_on_b);

    let res = execute(&mut ctx, &mut router, msg);

    assert!(res.is_ok(), "Execution success: happy path");

    assert_eq!(ctx.channel_counter().unwrap(), 1);

    assert_eq!(ctx.events.len(), 2);

    assert!(matches!(
        ctx.events[0],
        IbcEvent::Message(MessageEvent::Channel)
    ));
    assert!(matches!(ctx.events[1], IbcEvent::OpenTryChannel(_)));
}

#[rstest]
fn chan_open_try_fail_no_connection(fixture: Fixture) {
    let Fixture {
        ctx, router, msg, ..
    } = fixture;

    let res = validate(&ctx, &router, msg);

    assert!(
        res.is_err(),
        "Validation fails because no connection exists in the context"
    )
}

#[rstest]
fn chan_open_try_fail_no_client_state(fixture: Fixture) {
    let Fixture {
        ctx,
        router,
        msg,
        conn_id_on_b,
        conn_end_on_b,
        ..
    } = fixture;
    let ctx = ctx.with_connection(conn_id_on_b, conn_end_on_b);

    let res = validate(&ctx, &router, msg);

    assert!(
        res.is_err(),
        "Validation fails because the context has no client state"
    )
}
