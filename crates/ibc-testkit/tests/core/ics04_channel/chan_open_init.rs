use ibc::clients::ics07_tendermint::client_type as tm_client_type;
use ibc::core::events::{IbcEvent, MessageEvent};
use ibc::core::ics02_client::height::Height;
use ibc::core::ics03_connection::connection::{ConnectionEnd, State as ConnectionState};
use ibc::core::ics03_connection::msgs::conn_open_init::MsgConnectionOpenInit;
use ibc::core::ics03_connection::version::get_compatible_versions;
use ibc::core::ics04_channel::msgs::chan_open_init::test_util::get_dummy_raw_msg_chan_open_init;
use ibc::core::ics04_channel::msgs::chan_open_init::MsgChannelOpenInit;
use ibc::core::ics04_channel::msgs::ChannelMsg;
use ibc::core::ics24_host::identifier::{ClientId, ConnectionId};
use ibc::core::{execute, validate, MsgEnvelope, ValidationContext};
use ibc::prelude::*;
use ibc_testkit::testapp::ibc::core::router::MockRouter;
use ibc_testkit::testapp::ibc::core::types::MockContext;
use rstest::*;
use test_log::test;

pub struct Fixture {
    pub ctx: MockContext,
    pub router: MockRouter,
    pub msg: MsgEnvelope,
}

#[fixture]
fn fixture() -> Fixture {
    let msg_chan_open_init =
        MsgChannelOpenInit::try_from(get_dummy_raw_msg_chan_open_init(None)).unwrap();

    let msg = MsgEnvelope::from(ChannelMsg::from(msg_chan_open_init));

    let default_ctx = MockContext::default();
    let router = MockRouter::new_with_transfer();

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

    Fixture { ctx, router, msg }
}

#[rstest]
fn chan_open_init_validate_happy_path(fixture: Fixture) {
    let Fixture {
        ctx, router, msg, ..
    } = fixture;

    let res = validate(&ctx, &router, msg);

    assert!(res.is_ok(), "Validation succeeds; good parameters")
}

#[rstest]
fn chan_open_init_validate_counterparty_chan_id_set(fixture: Fixture) {
    let Fixture { ctx, router, .. } = fixture;

    let msg = MsgChannelOpenInit::try_from(get_dummy_raw_msg_chan_open_init(None)).unwrap();

    let msg_envelope = MsgEnvelope::from(ChannelMsg::from(msg));

    let res = validate(&ctx, &router, msg_envelope);

    assert!(
        res.is_ok(),
        "Validation succeeds even if counterparty channel id is set by relayer"
    )
}

#[rstest]
fn chan_open_init_execute_happy_path(fixture: Fixture) {
    let Fixture {
        mut ctx,
        mut router,
        msg,
        ..
    } = fixture;

    let res = execute(&mut ctx, &mut router, msg);

    assert!(res.is_ok(), "Execution succeeds; good parameters");

    assert_eq!(ctx.channel_counter().unwrap(), 1);

    assert_eq!(ctx.events.len(), 2);

    assert!(matches!(
        ctx.events[0],
        IbcEvent::Message(MessageEvent::Channel)
    ));
    assert!(matches!(ctx.events[1], IbcEvent::OpenInitChannel(_)));
}

#[rstest]
fn chan_open_init_fail_no_connection(fixture: Fixture) {
    let Fixture { router, msg, .. } = fixture;

    let res = validate(&MockContext::default(), &router, msg);

    assert!(
        res.is_err(),
        "Validation fails because no connection exists in the context"
    )
}
