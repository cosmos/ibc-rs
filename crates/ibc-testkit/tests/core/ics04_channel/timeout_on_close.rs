use ibc::core::ics02_client::ClientExecutionContext;
use ibc::core::ics03_connection::connection::{
    ConnectionEnd, Counterparty as ConnectionCounterparty, State as ConnectionState,
};
use ibc::core::ics03_connection::version::get_compatible_versions;
use ibc::core::ics04_channel::channel::{ChannelEnd, Counterparty, Order, State};
use ibc::core::ics04_channel::commitment::{compute_packet_commitment, PacketCommitment};
use ibc::core::ics04_channel::msgs::timeout_on_close::test_util::get_dummy_raw_msg_timeout_on_close;
use ibc::core::ics04_channel::msgs::timeout_on_close::MsgTimeoutOnClose;
use ibc::core::ics04_channel::msgs::PacketMsg;
use ibc::core::ics04_channel::Version;
use ibc::core::ics24_host::identifier::{ChannelId, ClientId, ConnectionId, PortId};
use ibc::core::timestamp::{Timestamp, ZERO_DURATION};
use ibc::core::{validate, ExecutionContext, MsgEnvelope};
use ibc::prelude::*;
use ibc::Height;
use ibc_testkit::testapp::ibc::core::router::MockRouter;
use ibc_testkit::testapp::ibc::core::types::MockContext;
use rstest::*;

pub struct Fixture {
    pub context: MockContext,
    pub router: MockRouter,
    pub msg: MsgTimeoutOnClose,
    pub packet_commitment: PacketCommitment,
    pub conn_end_on_a: ConnectionEnd,
    pub chan_end_on_a: ChannelEnd,
}

#[fixture]
fn fixture() -> Fixture {
    let client_height = Height::new(0, 2).unwrap();
    let context = MockContext::default().with_client(&ClientId::default(), client_height);
    let router = MockRouter::new_with_transfer();

    let height = 2;
    let timeout_timestamp = 5;

    let msg = MsgTimeoutOnClose::try_from(get_dummy_raw_msg_timeout_on_close(
        height,
        timeout_timestamp,
    ))
    .unwrap();

    let packet = msg.packet.clone();

    let packet_commitment = compute_packet_commitment(
        &msg.packet.data,
        &msg.packet.timeout_height_on_b,
        &msg.packet.timeout_timestamp_on_b,
    );

    let chan_end_on_a = ChannelEnd::new(
        State::Open,
        Order::Ordered,
        Counterparty::new(packet.port_id_on_b.clone(), Some(packet.chan_id_on_b)),
        vec![ConnectionId::default()],
        Version::new("ics20-1".to_string()),
    )
    .unwrap();

    let conn_end_on_a = ConnectionEnd::new(
        ConnectionState::Open,
        ClientId::default(),
        ConnectionCounterparty::new(
            ClientId::default(),
            Some(ConnectionId::default()),
            Default::default(),
        ),
        get_compatible_versions(),
        ZERO_DURATION,
    )
    .unwrap();

    Fixture {
        context,
        router,
        msg,
        packet_commitment,
        conn_end_on_a,
        chan_end_on_a,
    }
}

#[rstest]
fn timeout_on_close_fail_no_channel(fixture: Fixture) {
    let Fixture {
        context,
        router,
        msg,
        ..
    } = fixture;

    let msg_envelope = MsgEnvelope::from(PacketMsg::from(msg));

    let res = validate(&context, &router, msg_envelope);

    assert!(
        res.is_err(),
        "Validation fails because no channel exists in the context"
    )
}

/// NO-OP case
#[rstest]
fn timeout_on_close_success_no_packet_commitment(fixture: Fixture) {
    let Fixture {
        context,
        router,
        msg,
        conn_end_on_a,
        chan_end_on_a,
        ..
    } = fixture;
    let context = context
        .with_channel(PortId::transfer(), ChannelId::default(), chan_end_on_a)
        .with_connection(ConnectionId::default(), conn_end_on_a);

    let msg_envelope = MsgEnvelope::from(PacketMsg::from(msg));

    let res = validate(&context, &router, msg_envelope);

    assert!(
        res.is_ok(),
        "Validation should succeed when no packet commitment is present"
    )
}

#[rstest]
fn timeout_on_close_success_happy_path(fixture: Fixture) {
    let Fixture {
        context,
        router,
        msg,
        packet_commitment,
        conn_end_on_a,
        chan_end_on_a,
        ..
    } = fixture;
    let mut context = context
        .with_channel(PortId::transfer(), ChannelId::default(), chan_end_on_a)
        .with_connection(ConnectionId::default(), conn_end_on_a)
        .with_packet_commitment(
            msg.packet.port_id_on_a.clone(),
            msg.packet.chan_id_on_a.clone(),
            msg.packet.seq_on_a,
            packet_commitment,
        );

    context
        .get_client_execution_context()
        .store_update_time(
            ClientId::default(),
            Height::new(0, 2).unwrap(),
            Timestamp::from_nanoseconds(5000).unwrap(),
        )
        .unwrap();
    context
        .get_client_execution_context()
        .store_update_height(
            ClientId::default(),
            Height::new(0, 2).unwrap(),
            Height::new(0, 5).unwrap(),
        )
        .unwrap();

    let msg_envelope = MsgEnvelope::from(PacketMsg::from(msg));

    let res = validate(&context, &router, msg_envelope);

    assert!(
        res.is_ok(),
        "Happy path: validation should succeed. err: {res:?}"
    )
}
