use ibc::clients::tendermint::types::client_type as tm_client_type;
use ibc::core::channel::types::msgs::{ChannelMsg, MsgChannelOpenInit};
use ibc::core::client::types::Height;
use ibc::core::connection::types::version::Version as ConnectionVersion;
use ibc::core::connection::types::{ConnectionEnd, State as ConnectionState};
use ibc::core::entrypoint::{execute, validate};
use ibc::core::handler::types::events::{IbcEvent, MessageEvent};
use ibc::core::handler::types::msgs::MsgEnvelope;
use ibc::core::host::types::identifiers::ConnectionId;
use ibc::core::host::ValidationContext;
use ibc_testkit::fixtures::core::channel::dummy_raw_msg_chan_open_init;
use ibc_testkit::fixtures::core::connection::dummy_msg_conn_open_init;
use ibc_testkit::testapp::ibc::core::router::MockRouter;
use ibc_testkit::testapp::ibc::core::types::{MockClientConfig, MockContext};
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
        MsgChannelOpenInit::try_from(dummy_raw_msg_chan_open_init(None)).unwrap();

    let msg = MsgEnvelope::from(ChannelMsg::from(msg_chan_open_init));

    let default_ctx = MockContext::default();
    let router = MockRouter::new_with_transfer();

    let msg_conn_init = dummy_msg_conn_open_init();

    let client_id_on_a = tm_client_type().build_client_id(0);
    let client_height = Height::new(0, 10).unwrap();

    let conn_end_on_a = ConnectionEnd::new(
        ConnectionState::Init,
        msg_conn_init.client_id_on_a.clone(),
        msg_conn_init.counterparty.clone(),
        ConnectionVersion::compatibles(),
        msg_conn_init.delay_period,
    )
    .unwrap();

    let ctx = default_ctx
        .with_client_config(
            MockClientConfig::builder()
                .client_id(client_id_on_a.clone())
                .latest_height(client_height)
                .build(),
        )
        .with_connection(ConnectionId::zero(), conn_end_on_a);

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

    let msg = MsgChannelOpenInit::try_from(dummy_raw_msg_chan_open_init(None)).unwrap();

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

    let ibc_events = ctx.get_events();

    assert_eq!(ibc_events.len(), 2);

    assert!(matches!(
        ibc_events[0],
        IbcEvent::Message(MessageEvent::Channel)
    ));
    assert!(matches!(ibc_events[1], IbcEvent::OpenInitChannel(_)));
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
