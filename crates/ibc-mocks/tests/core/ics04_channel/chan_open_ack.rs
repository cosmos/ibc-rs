use ibc::applications::transfer::MODULE_ID_STR;
use ibc::core::events::{IbcEvent, MessageEvent};
use ibc::core::ics03_connection::connection::{
    ConnectionEnd, Counterparty as ConnectionCounterparty, State as ConnectionState,
};
use ibc::core::ics03_connection::msgs::test_util::get_dummy_raw_counterparty;
use ibc::core::ics03_connection::version::get_compatible_versions;
use ibc::core::ics04_channel::channel::{ChannelEnd, Counterparty, Order, State};
use ibc::core::ics04_channel::msgs::chan_open_ack::test_util::get_dummy_raw_msg_chan_open_ack;
use ibc::core::ics04_channel::msgs::chan_open_ack::MsgChannelOpenAck;
use ibc::core::ics04_channel::msgs::ChannelMsg;
use ibc::core::ics24_host::identifier::{ClientId, ConnectionId};
use ibc::core::router::ModuleId;
use ibc::core::timestamp::ZERO_DURATION;
use ibc::core::{execute, validate, MsgEnvelope};
use ibc::mock::client_state::client_type as mock_client_type;
use ibc::prelude::*;
use ibc::Height;
use ibc_mocks::core::definition::MockContext;
use ibc_mocks::router::definition::MockRouter;
use rstest::*;
use test_log::test;

pub struct Fixture {
    pub context: MockContext,
    pub router: MockRouter,
    pub module_id: ModuleId,
    pub msg: MsgChannelOpenAck,
    pub client_id_on_a: ClientId,
    pub conn_id_on_a: ConnectionId,
    pub conn_end_on_a: ConnectionEnd,
    pub chan_end_on_a: ChannelEnd,
    pub proof_height: u64,
}

#[fixture]
fn fixture() -> Fixture {
    let proof_height = 10;
    let context = MockContext::default();

    let module_id = ModuleId::new(MODULE_ID_STR.to_string());
    let router = MockRouter::new_with_transfer();

    let client_id_on_a = ClientId::new(mock_client_type(), 45).unwrap();
    let conn_id_on_a = ConnectionId::new(2);
    let conn_end_on_a = ConnectionEnd::new(
        ConnectionState::Open,
        client_id_on_a.clone(),
        ConnectionCounterparty::try_from(get_dummy_raw_counterparty(Some(0))).unwrap(),
        get_compatible_versions(),
        ZERO_DURATION,
    )
    .unwrap();

    let msg = MsgChannelOpenAck::try_from(get_dummy_raw_msg_chan_open_ack(proof_height)).unwrap();

    let chan_end_on_a = ChannelEnd::new(
        State::Init,
        Order::Unordered,
        Counterparty::new(msg.port_id_on_a.clone(), Some(msg.chan_id_on_b.clone())),
        vec![conn_id_on_a.clone()],
        msg.version_on_b.clone(),
    )
    .unwrap();

    Fixture {
        context,
        router,
        module_id,
        msg,
        client_id_on_a,
        conn_id_on_a,
        conn_end_on_a,
        chan_end_on_a,
        proof_height,
    }
}

#[rstest]
fn chan_open_ack_happy_path(fixture: Fixture) {
    let Fixture {
        context,
        router,
        msg,
        client_id_on_a,
        conn_id_on_a,
        conn_end_on_a,
        chan_end_on_a,
        proof_height,
        ..
    } = fixture;

    let context = context
        .with_client(&client_id_on_a, Height::new(0, proof_height).unwrap())
        .with_connection(conn_id_on_a, conn_end_on_a)
        .with_channel(
            msg.port_id_on_a.clone(),
            msg.chan_id_on_a.clone(),
            chan_end_on_a,
        );

    let msg_envelope = MsgEnvelope::from(ChannelMsg::from(msg));

    let res = validate(&context, &router, msg_envelope);

    assert!(res.is_ok(), "Validation happy path")
}

#[rstest]
fn chan_open_ack_execute_happy_path(fixture: Fixture) {
    let Fixture {
        context,
        mut router,
        msg,
        client_id_on_a,
        conn_id_on_a,
        conn_end_on_a,
        chan_end_on_a,
        proof_height,
        ..
    } = fixture;

    let mut context = context
        .with_client(&client_id_on_a, Height::new(0, proof_height).unwrap())
        .with_connection(conn_id_on_a, conn_end_on_a)
        .with_channel(
            msg.port_id_on_a.clone(),
            msg.chan_id_on_a.clone(),
            chan_end_on_a,
        );

    let msg_envelope = MsgEnvelope::from(ChannelMsg::from(msg.clone()));

    let res = execute(&mut context, &mut router, msg_envelope);

    assert!(res.is_ok(), "Execution happy path");

    assert_eq!(context.events.len(), 2);
    assert!(matches!(
        context.events[0],
        IbcEvent::Message(MessageEvent::Channel)
    ));
    assert!(matches!(context.events[1], IbcEvent::OpenAckChannel(_)));
}

#[rstest]
fn chan_open_ack_fail_no_connection(fixture: Fixture) {
    let Fixture {
        context,
        router,
        msg,
        client_id_on_a,
        chan_end_on_a,
        proof_height,
        ..
    } = fixture;

    let context = context
        .with_client(&client_id_on_a, Height::new(0, proof_height).unwrap())
        .with_channel(
            msg.port_id_on_a.clone(),
            msg.chan_id_on_a.clone(),
            chan_end_on_a,
        );

    let msg_envelope = MsgEnvelope::from(ChannelMsg::from(msg));

    let res = validate(&context, &router, msg_envelope);

    assert!(
        res.is_err(),
        "Validation fails because no connection exists in the context"
    )
}

#[rstest]
fn chan_open_ack_fail_no_channel(fixture: Fixture) {
    let Fixture {
        context,
        router,
        msg,
        client_id_on_a,
        conn_id_on_a,
        conn_end_on_a,
        proof_height,
        ..
    } = fixture;
    let context = context
        .with_client(&client_id_on_a, Height::new(0, proof_height).unwrap())
        .with_connection(conn_id_on_a, conn_end_on_a);

    let msg_envelope = MsgEnvelope::from(ChannelMsg::from(msg));

    let res = validate(&context, &router, msg_envelope);

    assert!(
        res.is_err(),
        "Validation fails because no channel exists in the context"
    )
}

#[rstest]
fn chan_open_ack_fail_channel_wrong_state(fixture: Fixture) {
    let Fixture {
        context,
        router,
        msg,
        client_id_on_a,
        conn_id_on_a,
        conn_end_on_a,
        proof_height,
        ..
    } = fixture;

    let wrong_chan_end = ChannelEnd::new(
        State::Open,
        Order::Unordered,
        Counterparty::new(msg.port_id_on_a.clone(), Some(msg.chan_id_on_b.clone())),
        vec![conn_id_on_a.clone()],
        msg.version_on_b.clone(),
    )
    .unwrap();
    let context = context
        .with_client(&client_id_on_a, Height::new(0, proof_height).unwrap())
        .with_connection(conn_id_on_a, conn_end_on_a)
        .with_channel(
            msg.port_id_on_a.clone(),
            msg.chan_id_on_a.clone(),
            wrong_chan_end,
        );

    let msg_envelope = MsgEnvelope::from(ChannelMsg::from(msg));

    let res = validate(&context, &router, msg_envelope);

    assert!(
        res.is_err(),
        "Validation fails because channel is in the wrong state"
    )
}
