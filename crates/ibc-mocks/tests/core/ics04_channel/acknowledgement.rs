use ibc::core::events::{IbcEvent, MessageEvent};
use ibc::core::ics02_client::height::Height;
use ibc::core::ics02_client::ClientExecutionContext;
use ibc::core::ics03_connection::connection::{
    ConnectionEnd, Counterparty as ConnectionCounterparty, State as ConnectionState,
};
use ibc::core::ics03_connection::version::get_compatible_versions;
use ibc::core::ics04_channel::channel::{ChannelEnd, Counterparty, Order, State};
use ibc::core::ics04_channel::commitment::{compute_packet_commitment, PacketCommitment};
use ibc::core::ics04_channel::msgs::acknowledgement::test_util::get_dummy_raw_msg_acknowledgement;
use ibc::core::ics04_channel::msgs::acknowledgement::MsgAcknowledgement;
use ibc::core::ics04_channel::msgs::PacketMsg;
use ibc::core::ics04_channel::Version;
use ibc::core::ics24_host::identifier::{ChannelId, ClientId, ConnectionId, PortId};
use ibc::core::timestamp::{Timestamp, ZERO_DURATION};
use ibc::core::{execute, validate, ExecutionContext, MsgEnvelope};
use ibc::prelude::*;
use ibc_mocks::core::definition::MockContext;
use ibc_mocks::router::definition::MockRouter;
use rstest::*;
use test_log::test;

struct Fixture {
    ctx: MockContext,
    router: MockRouter,
    client_height: Height,
    msg: MsgAcknowledgement,
    packet_commitment: PacketCommitment,
    conn_end_on_a: ConnectionEnd,
    chan_end_on_a_ordered: ChannelEnd,
    chan_end_on_a_unordered: ChannelEnd,
}

#[fixture]
fn fixture() -> Fixture {
    let client_height = Height::new(0, 2).unwrap();
    let ctx = MockContext::default().with_client(&ClientId::default(), client_height);

    let router = MockRouter::new_with_transfer();

    let msg = MsgAcknowledgement::try_from(get_dummy_raw_msg_acknowledgement(
        client_height.revision_height(),
    ))
    .unwrap();

    let packet = msg.packet.clone();

    let packet_commitment = compute_packet_commitment(
        &packet.data,
        &packet.timeout_height_on_b,
        &packet.timeout_timestamp_on_b,
    );

    let chan_end_on_a_unordered = ChannelEnd::new(
        State::Open,
        Order::Unordered,
        Counterparty::new(packet.port_id_on_b, Some(packet.chan_id_on_b)),
        vec![ConnectionId::default()],
        Version::new("ics20-1".to_string()),
    )
    .unwrap();

    let mut chan_end_on_a_ordered = chan_end_on_a_unordered.clone();
    chan_end_on_a_ordered.ordering = Order::Ordered;

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
        ctx,
        router,
        client_height,
        msg,
        packet_commitment,
        conn_end_on_a,
        chan_end_on_a_unordered,
        chan_end_on_a_ordered,
    }
}

#[rstest]
fn ack_fail_no_channel(fixture: Fixture) {
    let Fixture {
        ctx, router, msg, ..
    } = fixture;

    let msg_envelope = MsgEnvelope::from(PacketMsg::from(msg));

    let res = validate(&ctx, &router, msg_envelope);

    assert!(
        res.is_err(),
        "Validation fails because no channel exists in the context"
    )
}

/// NO-OP case
#[rstest]
fn ack_success_no_packet_commitment(fixture: Fixture) {
    let Fixture {
        ctx,
        router,
        msg,
        conn_end_on_a,
        chan_end_on_a_unordered,
        client_height,
        ..
    } = fixture;
    let ctx = ctx
        .with_client(&ClientId::default(), client_height)
        .with_channel(
            PortId::transfer(),
            ChannelId::default(),
            chan_end_on_a_unordered,
        )
        .with_connection(ConnectionId::default(), conn_end_on_a);

    let msg_envelope = MsgEnvelope::from(PacketMsg::from(msg));

    let res = validate(&ctx, &router, msg_envelope);

    assert!(
        res.is_ok(),
        "Validation should succeed when no packet commitment is present"
    )
}

#[rstest]
fn ack_success_happy_path(fixture: Fixture) {
    let Fixture {
        ctx,
        router,
        msg,
        packet_commitment,
        conn_end_on_a,
        chan_end_on_a_unordered,
        client_height,
        ..
    } = fixture;
    let mut ctx: MockContext = ctx
        .with_client(&ClientId::default(), client_height)
        .with_channel(
            PortId::transfer(),
            ChannelId::default(),
            chan_end_on_a_unordered,
        )
        .with_connection(ConnectionId::default(), conn_end_on_a)
        .with_packet_commitment(
            msg.packet.port_id_on_a.clone(),
            msg.packet.chan_id_on_a.clone(),
            msg.packet.seq_on_a,
            packet_commitment,
        );
    ctx.get_client_execution_context()
        .store_update_time(
            ClientId::default(),
            client_height,
            Timestamp::from_nanoseconds(1000).unwrap(),
        )
        .unwrap();
    ctx.get_client_execution_context()
        .store_update_height(
            ClientId::default(),
            client_height,
            Height::new(0, 4).unwrap(),
        )
        .unwrap();

    let msg_envelope = MsgEnvelope::from(PacketMsg::from(msg));

    let res = validate(&ctx, &router, msg_envelope);

    assert!(
        res.is_ok(),
        "Happy path: validation should succeed. err: {res:?}"
    )
}

#[rstest]
fn ack_unordered_chan_execute(fixture: Fixture) {
    let Fixture {
        ctx,
        mut router,
        msg,
        packet_commitment,
        conn_end_on_a,
        chan_end_on_a_unordered,
        ..
    } = fixture;
    let mut ctx = ctx
        .with_channel(
            PortId::transfer(),
            ChannelId::default(),
            chan_end_on_a_unordered,
        )
        .with_connection(ConnectionId::default(), conn_end_on_a)
        .with_packet_commitment(
            msg.packet.port_id_on_a.clone(),
            msg.packet.chan_id_on_a.clone(),
            msg.packet.seq_on_a,
            packet_commitment,
        );

    let msg_envelope = MsgEnvelope::from(PacketMsg::from(msg));

    let res = execute(&mut ctx, &mut router, msg_envelope);

    assert!(res.is_ok());

    assert_eq!(ctx.events.len(), 2);
    assert!(matches!(
        ctx.events[0],
        IbcEvent::Message(MessageEvent::Channel)
    ));
    assert!(matches!(ctx.events[1], IbcEvent::AcknowledgePacket(_)));
}

#[rstest]
fn ack_ordered_chan_execute(fixture: Fixture) {
    let Fixture {
        ctx,
        mut router,
        msg,
        packet_commitment,
        conn_end_on_a,
        chan_end_on_a_ordered,
        ..
    } = fixture;
    let mut ctx = ctx
        .with_channel(
            PortId::transfer(),
            ChannelId::default(),
            chan_end_on_a_ordered,
        )
        .with_connection(ConnectionId::default(), conn_end_on_a)
        .with_packet_commitment(
            msg.packet.port_id_on_a.clone(),
            msg.packet.chan_id_on_a.clone(),
            msg.packet.seq_on_a,
            packet_commitment,
        );

    let msg_envelope = MsgEnvelope::from(PacketMsg::from(msg));

    let res = execute(&mut ctx, &mut router, msg_envelope);

    assert!(res.is_ok());

    assert_eq!(ctx.events.len(), 2);
    assert!(matches!(
        ctx.events[0],
        IbcEvent::Message(MessageEvent::Channel)
    ));
    assert!(matches!(ctx.events[1], IbcEvent::AcknowledgePacket(_)));
}
