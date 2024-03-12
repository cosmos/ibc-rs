use ibc::core::channel::types::channel::{ChannelEnd, Counterparty, Order, State};
use ibc::core::channel::types::commitment::{compute_packet_commitment, PacketCommitment};
use ibc::core::channel::types::msgs::{MsgTimeout, PacketMsg};
use ibc::core::channel::types::Version;
use ibc::core::client::context::ClientExecutionContext;
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
use ibc::core::host::ExecutionContext;
use ibc::core::primitives::*;
use ibc_testkit::fixtures::core::channel::dummy_raw_msg_timeout;
use ibc_testkit::testapp::ibc::core::router::MockRouter;
use ibc_testkit::testapp::ibc::core::types::{MockClientConfig, MockContext};
use rstest::*;

struct Fixture {
    ctx: MockContext,
    pub router: MockRouter,
    client_height: Height,
    msg: MsgTimeout,
    packet_commitment: PacketCommitment,
    conn_end_on_a: ConnectionEnd,
    chan_end_on_a_ordered: ChannelEnd,
    chan_end_on_a_unordered: ChannelEnd,
    client_id: ClientId,
}

#[fixture]
fn fixture() -> Fixture {
    let client_id = ClientId::new("07-tendermint", 0).expect("no error");

    let client_height = Height::new(0, 2).unwrap();
    let ctx = MockContext::default().with_client_config(
        MockClientConfig::builder()
            .latest_height(client_height)
            .build(),
    );

    let client_height = Height::new(0, 2).unwrap();

    let router = MockRouter::new_with_transfer();

    let msg_proof_height = 2;
    let msg_timeout_height = 5;
    let timeout_timestamp = Timestamp::now().nanoseconds();

    let msg = MsgTimeout::try_from(dummy_raw_msg_timeout(
        msg_proof_height,
        msg_timeout_height,
        timeout_timestamp,
    ))
    .unwrap();

    let packet = msg.packet.clone();

    let packet_commitment = compute_packet_commitment(
        &msg.packet.data,
        &msg.packet.timeout_height_on_b,
        &msg.packet.timeout_timestamp_on_b,
    );

    let chan_end_on_a_unordered = ChannelEnd::new(
        State::Open,
        Order::Unordered,
        Counterparty::new(packet.port_id_on_b.clone(), Some(packet.chan_id_on_b)),
        vec![ConnectionId::zero()],
        Version::new("ics20-1".to_string()),
    )
    .unwrap();

    let mut chan_end_on_a_ordered = chan_end_on_a_unordered.clone();
    chan_end_on_a_ordered.ordering = Order::Ordered;

    let conn_end_on_a = ConnectionEnd::new(
        ConnectionState::Open,
        client_id.clone(),
        ConnectionCounterparty::new(
            client_id.clone(),
            Some(ConnectionId::zero()),
            CommitmentPrefix::empty(),
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
        client_id,
    }
}

#[rstest]
fn timeout_fail_no_channel(fixture: Fixture) {
    let Fixture {
        ctx,
        router,
        msg,
        client_height,
        ..
    } = fixture;
    let ctx = ctx.with_client_config(
        MockClientConfig::builder()
            .latest_height(client_height)
            .build(),
    );
    let msg_envelope = MsgEnvelope::from(PacketMsg::from(msg));
    let res = validate(&ctx, &router, msg_envelope);

    assert!(
        res.is_err(),
        "Validation fails because no channel exists in the context"
    )
}

#[rstest]
fn timeout_fail_no_consensus_state_for_height(fixture: Fixture) {
    let Fixture {
        ctx,
        router,
        msg,
        chan_end_on_a_unordered,
        conn_end_on_a,
        packet_commitment,
        ..
    } = fixture;

    let packet = msg.packet.clone();

    let ctx = ctx
        .with_channel(
            PortId::transfer(),
            ChannelId::zero(),
            chan_end_on_a_unordered,
        )
        .with_connection(ConnectionId::zero(), conn_end_on_a)
        .with_packet_commitment(
            packet.port_id_on_a,
            packet.chan_id_on_a,
            packet.seq_on_a,
            packet_commitment,
        );

    let msg_envelope = MsgEnvelope::from(PacketMsg::from(msg));

    let res = validate(&ctx, &router, msg_envelope);

    assert!(
            res.is_err(),
            "Validation fails because the client does not have a consensus state for the required height"
        )
}

#[rstest]
fn timeout_fail_proof_timeout_not_reached(fixture: Fixture) {
    let Fixture {
        ctx,
        router,
        mut msg,
        chan_end_on_a_unordered,
        conn_end_on_a,
        client_height,
        client_id,
        ..
    } = fixture;

    // timeout timestamp has not reached yet
    let timeout_timestamp_on_b =
        (msg.packet.timeout_timestamp_on_b + core::time::Duration::new(10, 0)).unwrap();
    msg.packet.timeout_timestamp_on_b = timeout_timestamp_on_b;
    let packet_commitment = compute_packet_commitment(
        &msg.packet.data,
        &msg.packet.timeout_height_on_b,
        &msg.packet.timeout_timestamp_on_b,
    );

    let packet = msg.packet.clone();

    let mut ctx = ctx
        .with_client_config(
            MockClientConfig::builder()
                .latest_height(client_height)
                .build(),
        )
        .with_connection(ConnectionId::zero(), conn_end_on_a)
        .with_channel(
            PortId::transfer(),
            ChannelId::zero(),
            chan_end_on_a_unordered,
        )
        .with_packet_commitment(
            packet.port_id_on_a,
            packet.chan_id_on_a,
            packet.seq_on_a,
            packet_commitment,
        );

    ctx.store_update_meta(
        client_id,
        client_height,
        Timestamp::from_nanoseconds(5).unwrap(),
        Height::new(0, 4).unwrap(),
    )
    .unwrap();

    let msg_envelope = MsgEnvelope::from(PacketMsg::from(msg));

    let res = validate(&ctx, &router, msg_envelope);

    assert!(
            res.is_err(),
            "Validation should fail because the timeout height was reached, but the timestamp hasn't been reached. Both the height and timestamp need to be reached for the packet to be considered timed out"
        )
}

/// NO-OP case
#[rstest]
fn timeout_success_no_packet_commitment(fixture: Fixture) {
    let Fixture {
        ctx,
        router,
        msg,
        conn_end_on_a,
        chan_end_on_a_unordered,
        ..
    } = fixture;
    let ctx = ctx
        .with_channel(
            PortId::transfer(),
            ChannelId::zero(),
            chan_end_on_a_unordered,
        )
        .with_connection(ConnectionId::zero(), conn_end_on_a);

    let msg_envelope = MsgEnvelope::from(PacketMsg::from(msg));

    let res = validate(&ctx, &router, msg_envelope);

    assert!(
        res.is_ok(),
        "Validation should succeed when no packet commitment is present"
    )
}

#[rstest]
fn timeout_unordered_channel_validate(fixture: Fixture) {
    let Fixture {
        ctx,
        router,
        msg,
        chan_end_on_a_unordered,
        conn_end_on_a,
        packet_commitment,
        client_height,
        client_id,
        ..
    } = fixture;

    let packet = msg.packet.clone();

    let mut ctx = ctx
        .with_client_config(
            MockClientConfig::builder()
                .latest_height(client_height)
                .build(),
        )
        .with_connection(ConnectionId::zero(), conn_end_on_a)
        .with_channel(
            PortId::transfer(),
            ChannelId::zero(),
            chan_end_on_a_unordered,
        )
        .with_packet_commitment(
            packet.port_id_on_a,
            packet.chan_id_on_a,
            packet.seq_on_a,
            packet_commitment,
        );

    ctx.get_client_execution_context()
        .store_update_meta(
            client_id,
            client_height,
            Timestamp::from_nanoseconds(1000).unwrap(),
            Height::new(0, 5).unwrap(),
        )
        .unwrap();

    let msg_envelope = MsgEnvelope::from(PacketMsg::from(msg));

    let res = validate(&ctx, &router, msg_envelope);

    assert!(res.is_ok(), "Good parameters for unordered channels")
}

#[rstest]
fn timeout_ordered_channel_validate(fixture: Fixture) {
    let Fixture {
        ctx,
        router,
        msg,
        chan_end_on_a_ordered,
        conn_end_on_a,
        packet_commitment,
        client_height,
        client_id,
        ..
    } = fixture;

    let packet = msg.packet.clone();

    let mut ctx = ctx
        .with_client_config(
            MockClientConfig::builder()
                .latest_height(client_height)
                .build(),
        )
        .with_connection(ConnectionId::zero(), conn_end_on_a)
        .with_channel(PortId::transfer(), ChannelId::zero(), chan_end_on_a_ordered)
        .with_packet_commitment(
            packet.port_id_on_a,
            packet.chan_id_on_a,
            packet.seq_on_a,
            packet_commitment,
        );

    ctx.store_update_meta(
        client_id,
        client_height,
        Timestamp::from_nanoseconds(1000).unwrap(),
        Height::new(0, 4).unwrap(),
    )
    .unwrap();

    let msg_envelope = MsgEnvelope::from(PacketMsg::from(msg));

    let res = validate(&ctx, &router, msg_envelope);

    assert!(res.is_ok(), "Good parameters for unordered channels")
}

#[rstest]
fn timeout_unordered_chan_execute(fixture: Fixture) {
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

    let res = execute(&mut ctx, &mut router, msg_envelope);

    assert!(res.is_ok());

    let ibc_events = ctx.get_events();

    // Unordered channels only emit one event
    assert_eq!(ibc_events.len(), 2);
    assert!(matches!(
        ibc_events[0],
        IbcEvent::Message(MessageEvent::Channel)
    ));
    assert!(matches!(ibc_events[1], IbcEvent::TimeoutPacket(_)));
}

#[rstest]
fn timeout_ordered_chan_execute(fixture: Fixture) {
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

    let res = execute(&mut ctx, &mut router, msg_envelope);

    assert!(res.is_ok());

    let ibc_events = ctx.get_events();

    // Ordered channels emit 2 events
    assert_eq!(ibc_events.len(), 4);
    assert!(matches!(
        ibc_events[0],
        IbcEvent::Message(MessageEvent::Channel)
    ));
    assert!(matches!(ibc_events[1], IbcEvent::TimeoutPacket(_)));
    assert!(matches!(
        ibc_events[2],
        IbcEvent::Message(MessageEvent::Channel)
    ));
    assert!(matches!(ibc_events[3], IbcEvent::ChannelClosed(_)));
}
