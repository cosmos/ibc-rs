use ibc::core::channel::types::channel::{ChannelEnd, Counterparty, Order, State};
use ibc::core::channel::types::commitment::{compute_packet_commitment, PacketCommitment};
use ibc::core::channel::types::msgs::{MsgAcknowledgement, PacketMsg};
use ibc::core::channel::types::Version;
use ibc::core::client::types::Height;
use ibc::core::commitment_types::commitment::CommitmentPrefix;
use ibc::core::connection::types::version::Version as ConnectionVersion;
use ibc::core::connection::types::{
    ConnectionEnd, Counterparty as ConnectionCounterparty, State as ConnectionState,
};
use ibc::core::entrypoint::{execute, validate};
use ibc::core::handler::types::events::{IbcEvent, MessageEvent};
use ibc::core::handler::types::msgs::MsgEnvelope;
use ibc::core::host::types::identifiers::{ChannelId, ClientId, ConnectionId, PortId};
use ibc::core::primitives::*;
use ibc_testkit::context::MockContext;
use ibc_testkit::fixtures::core::channel::dummy_raw_msg_acknowledgement;
use ibc_testkit::hosts::MockHost;
use ibc_testkit::testapp::ibc::core::router::MockRouter;
use ibc_testkit::testapp::ibc::core::types::LightClientState;
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
    let default_client_id = ClientId::new("07-tendermint", 0).expect("no error");

    let client_height = Height::new(0, 2).unwrap();
    let ctx = MockContext::default().with_light_client(
        &ClientId::new("07-tendermint", 0).expect("no error"),
        LightClientState::<MockHost>::with_latest_height(client_height),
    );

    let router = MockRouter::new_with_transfer();

    let msg = MsgAcknowledgement::try_from(dummy_raw_msg_acknowledgement(
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
        vec![ConnectionId::zero()],
        Version::new("ics20-1".to_string()),
    )
    .unwrap();

    let mut chan_end_on_a_ordered = chan_end_on_a_unordered.clone();
    chan_end_on_a_ordered.ordering = Order::Ordered;

    let conn_end_on_a = ConnectionEnd::new(
        ConnectionState::Open,
        default_client_id.clone(),
        ConnectionCounterparty::new(
            default_client_id,
            Some(ConnectionId::zero()),
            CommitmentPrefix::try_from(vec![0]).expect("no error"),
        ),
        ConnectionVersion::compatibles(),
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
        chan_end_on_a_ordered,
        chan_end_on_a_unordered,
    }
}

#[rstest]
fn ack_fail_no_channel(fixture: Fixture) {
    let Fixture {
        ctx, router, msg, ..
    } = fixture;

    let msg_envelope = MsgEnvelope::from(PacketMsg::from(msg));

    let res = validate(&ctx.ibc_store, &router, msg_envelope);

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
        .with_light_client(
            &ClientId::new("07-tendermint", 0).expect("no error"),
            LightClientState::<MockHost>::with_latest_height(client_height),
        )
        .with_channel(
            PortId::transfer(),
            ChannelId::zero(),
            chan_end_on_a_unordered,
        )
        .with_connection(ConnectionId::zero(), conn_end_on_a);

    let msg_envelope = MsgEnvelope::from(PacketMsg::from(msg));

    let res = validate(&ctx.ibc_store, &router, msg_envelope);

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
    let ctx = ctx
        .with_light_client(
            &ClientId::new("07-tendermint", 0).expect("no error"),
            LightClientState::<MockHost>::with_latest_height(client_height),
        )
        .with_channel(
            PortId::transfer(),
            ChannelId::zero(),
            chan_end_on_a_unordered,
        )
        .with_connection(ConnectionId::zero(), conn_end_on_a)
        .with_packet_commitment(
            msg.packet.port_id_on_a.clone(),
            msg.packet.chan_id_on_a.clone(),
            msg.packet.seq_on_a,
            packet_commitment,
        );

    let msg_envelope = MsgEnvelope::from(PacketMsg::from(msg));

    let res = validate(&ctx.ibc_store, &router, msg_envelope);

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
            ChannelId::zero(),
            chan_end_on_a_unordered,
        )
        .with_connection(ConnectionId::zero(), conn_end_on_a)
        .with_packet_commitment(
            msg.packet.port_id_on_a.clone(),
            msg.packet.chan_id_on_a.clone(),
            msg.packet.seq_on_a,
            packet_commitment,
        );

    let msg_envelope = MsgEnvelope::from(PacketMsg::from(msg));

    let res = execute(&mut ctx.ibc_store, &mut router, msg_envelope);

    assert!(res.is_ok());

    let ibc_events = ctx.get_events();

    assert_eq!(ibc_events.len(), 2);
    assert!(matches!(
        ibc_events[0],
        IbcEvent::Message(MessageEvent::Channel)
    ));
    assert!(matches!(ibc_events[1], IbcEvent::AcknowledgePacket(_)));
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
        .with_channel(PortId::transfer(), ChannelId::zero(), chan_end_on_a_ordered)
        .with_connection(ConnectionId::zero(), conn_end_on_a)
        .with_packet_commitment(
            msg.packet.port_id_on_a.clone(),
            msg.packet.chan_id_on_a.clone(),
            msg.packet.seq_on_a,
            packet_commitment,
        );

    let msg_envelope = MsgEnvelope::from(PacketMsg::from(msg));

    let res = execute(&mut ctx.ibc_store, &mut router, msg_envelope);

    assert!(res.is_ok());

    let ibc_events = ctx.get_events();

    assert_eq!(ibc_events.len(), 2);
    assert!(matches!(
        ibc_events[0],
        IbcEvent::Message(MessageEvent::Channel)
    ));
    assert!(matches!(ibc_events[1], IbcEvent::AcknowledgePacket(_)));
}
