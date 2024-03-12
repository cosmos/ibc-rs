use ibc::core::channel::types::channel::{ChannelEnd, Counterparty, Order, State};
use ibc::core::channel::types::commitment::{compute_packet_commitment, PacketCommitment};
use ibc::core::channel::types::msgs::{MsgTimeoutOnClose, PacketMsg};
use ibc::core::channel::types::Version;
use ibc::core::client::context::ClientExecutionContext;
use ibc::core::client::types::Height;
use ibc::core::commitment_types::commitment::CommitmentPrefix;
use ibc::core::connection::types::version::Version as ConnectionVersion;
use ibc::core::connection::types::{
    ConnectionEnd, Counterparty as ConnectionCounterparty, State as ConnectionState,
};
use ibc::core::entrypoint::validate;
use ibc::core::handler::types::msgs::MsgEnvelope;
use ibc::core::host::types::identifiers::{ChannelId, ClientId, ConnectionId, PortId};
use ibc::core::host::ExecutionContext;
use ibc::core::primitives::*;
use ibc_testkit::fixtures::core::channel::dummy_raw_msg_timeout_on_close;
use ibc_testkit::testapp::ibc::core::router::MockRouter;
use ibc_testkit::testapp::ibc::core::types::{MockClientConfig, MockContext};
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
    let default_client_id = ClientId::new("07-tendermint", 0).expect("no error");

    let client_height = Height::new(0, 2).unwrap();
    let context = MockContext::default().with_client_config(
        MockClientConfig::builder()
            .latest_height(client_height)
            .build(),
    );
    let router = MockRouter::new_with_transfer();

    let height = 2;
    let timeout_timestamp = 5;

    let msg =
        MsgTimeoutOnClose::try_from(dummy_raw_msg_timeout_on_close(height, timeout_timestamp))
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
        vec![ConnectionId::zero()],
        Version::new("ics20-1".to_string()),
    )
    .unwrap();

    let conn_end_on_a = ConnectionEnd::new(
        ConnectionState::Open,
        default_client_id.clone(),
        ConnectionCounterparty::new(
            default_client_id.clone(),
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
        .with_channel(PortId::transfer(), ChannelId::zero(), chan_end_on_a)
        .with_connection(ConnectionId::zero(), conn_end_on_a);

    let msg_envelope = MsgEnvelope::from(PacketMsg::from(msg));

    let res = validate(&context, &router, msg_envelope);

    assert!(
        res.is_ok(),
        "Validation should succeed when no packet commitment is present"
    )
}

#[rstest]
fn timeout_on_close_success_happy_path(fixture: Fixture) {
    let default_client_id = ClientId::new("07-tendermint", 0).expect("no error");

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
        .with_channel(PortId::transfer(), ChannelId::zero(), chan_end_on_a)
        .with_connection(ConnectionId::zero(), conn_end_on_a)
        .with_packet_commitment(
            msg.packet.port_id_on_a.clone(),
            msg.packet.chan_id_on_a.clone(),
            msg.packet.seq_on_a,
            packet_commitment,
        );

    context
        .get_client_execution_context()
        .store_update_meta(
            default_client_id,
            Height::new(0, 2).unwrap(),
            Timestamp::from_nanoseconds(5000).unwrap(),
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
