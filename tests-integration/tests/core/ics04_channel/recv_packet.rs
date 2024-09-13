use ibc::core::channel::types::channel::{ChannelEnd, Counterparty, Order, State};
use ibc::core::channel::types::msgs::{MsgRecvPacket, PacketMsg};
use ibc::core::channel::types::packet::Packet;
use ibc::core::channel::types::timeout::TimeoutTimestamp;
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
use ibc_testkit::fixtures::core::channel::{dummy_msg_recv_packet, dummy_raw_msg_recv_packet};
use ibc_testkit::fixtures::core::signer::dummy_account_id;
use ibc_testkit::hosts::MockHost;
use ibc_testkit::testapp::ibc::core::router::MockRouter;
use ibc_testkit::testapp::ibc::core::types::LightClientState;
use rstest::*;
use test_log::test;

pub struct Fixture {
    pub context: MockContext,
    pub router: MockRouter,
    pub client_height: Height,
    pub host_height: Height,
    pub msg: MsgRecvPacket,
    pub conn_end_on_b: ConnectionEnd,
    pub chan_end_on_b: ChannelEnd,
    pub client_id: ClientId,
}

#[fixture]
fn fixture() -> Fixture {
    let client_id = ClientId::new("07-tendermint", 0).expect("no error");

    let context = MockContext::default();

    let router = MockRouter::new_with_transfer();

    let host_height = context.latest_height().increment();

    let client_height = host_height.increment();

    let msg = MsgRecvPacket::try_from(dummy_raw_msg_recv_packet(client_height.revision_height()))
        .unwrap();

    let packet = msg.packet.clone();

    let chan_end_on_b = ChannelEnd::new(
        State::Open,
        Order::Unordered,
        Counterparty::new(packet.port_id_on_a, Some(packet.chan_id_on_a)),
        vec![ConnectionId::zero()],
        Version::new("ics20-1".to_string()),
    )
    .unwrap();

    let conn_end_on_b = ConnectionEnd::new(
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
        context,
        router,
        client_height,
        host_height,
        msg,
        conn_end_on_b,
        chan_end_on_b,
        client_id,
    }
}

#[rstest]
fn recv_packet_fail_no_channel(fixture: Fixture) {
    let Fixture {
        context,
        router,
        msg,
        ..
    } = fixture;

    let msg_envelope = MsgEnvelope::from(PacketMsg::from(msg));

    let res = validate(&context.ibc_store, &router, msg_envelope);

    assert!(
        res.is_err(),
        "Validation fails because no channel exists in the context"
    )
}

#[rstest]
fn recv_packet_validate_happy_path(fixture: Fixture) {
    let Fixture {
        context,
        router,
        msg,
        conn_end_on_b,
        chan_end_on_b,
        client_height,
        host_height,
        ..
    } = fixture;

    let packet = &msg.packet;
    let context = context
        .with_light_client(
            &ClientId::new("07-tendermint", 0).expect("no error"),
            LightClientState::<MockHost>::with_latest_height(client_height),
        )
        .with_connection(ConnectionId::zero(), conn_end_on_b)
        .with_channel(
            packet.port_id_on_b.clone(),
            packet.chan_id_on_b.clone(),
            chan_end_on_b,
        )
        .with_send_sequence(
            packet.port_id_on_b.clone(),
            packet.chan_id_on_b.clone(),
            1.into(),
        )
        .advance_block_up_to_height(host_height)
        // This `with_recv_sequence` is required for ordered channels
        .with_recv_sequence(
            packet.port_id_on_b.clone(),
            packet.chan_id_on_b.clone(),
            packet.seq_on_a,
        );

    // Note: For unordered channels, there's no need to set a packet receipt.
    // The validation will pass whether the receipt exists or not.

    let msg_envelope = MsgEnvelope::from(PacketMsg::from(msg));

    let res = validate(&context.ibc_store, &router, msg_envelope);

    assert!(
        res.is_ok(),
        "Happy path: validation should succeed. err: {res:?}"
    )
}

#[rstest]
fn recv_packet_timeout_expired(fixture: Fixture) {
    let Fixture {
        context,
        router,
        msg,
        conn_end_on_b,
        chan_end_on_b,
        client_height,
        host_height,
        ..
    } = fixture;

    let packet_old = Packet {
        seq_on_a: 1.into(),
        port_id_on_a: PortId::transfer(),
        chan_id_on_a: ChannelId::zero(),
        port_id_on_b: PortId::transfer(),
        chan_id_on_b: ChannelId::zero(),
        data: Vec::new(),
        timeout_height_on_b: client_height.into(),
        timeout_timestamp_on_b: TimeoutTimestamp::from_nanoseconds(1),
    };

    let msg_packet_old = dummy_msg_recv_packet(
        packet_old,
        msg.proof_commitment_on_a.clone(),
        msg.proof_height_on_a,
        dummy_account_id(),
    );

    let msg_envelope = MsgEnvelope::from(PacketMsg::from(msg_packet_old));

    let context = context
        .with_light_client(
            &ClientId::new("07-tendermint", 0).expect("no error"),
            LightClientState::<MockHost>::with_latest_height(client_height),
        )
        .with_connection(ConnectionId::zero(), conn_end_on_b)
        .with_channel(PortId::transfer(), ChannelId::zero(), chan_end_on_b)
        .with_send_sequence(PortId::transfer(), ChannelId::zero(), 1.into())
        .advance_block_up_to_height(host_height);

    let res = validate(&context.ibc_store, &router, msg_envelope);

    assert!(
        res.is_err(),
        "recv_packet validation should fail when the packet has timed out"
    )
}

#[rstest]
fn recv_packet_execute_happy_path(fixture: Fixture) {
    let Fixture {
        context,
        mut router,
        msg,
        conn_end_on_b,
        chan_end_on_b,
        client_height,
        ..
    } = fixture;
    let mut ctx = context
        .with_light_client(
            &ClientId::new("07-tendermint", 0).expect("no error"),
            LightClientState::<MockHost>::with_latest_height(client_height),
        )
        .with_connection(ConnectionId::zero(), conn_end_on_b)
        .with_channel(PortId::transfer(), ChannelId::zero(), chan_end_on_b);

    let msg_env = MsgEnvelope::from(PacketMsg::from(msg));

    let res = execute(&mut ctx.ibc_store, &mut router, msg_env);

    assert!(res.is_ok());

    let ibc_events = ctx.get_events();

    assert_eq!(ibc_events.len(), 4);
    assert!(matches!(
        &ibc_events[0],
        &IbcEvent::Message(MessageEvent::Channel)
    ));
    assert!(matches!(&ibc_events[1], &IbcEvent::ReceivePacket(_)));
    assert!(matches!(
        &ibc_events[2],
        &IbcEvent::Message(MessageEvent::Channel)
    ));
    assert!(matches!(&ibc_events[3], &IbcEvent::WriteAcknowledgement(_)));
}
