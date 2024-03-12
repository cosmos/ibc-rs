use core::ops::Add;
use core::time::Duration;

use ibc::core::channel::handler::send_packet;
use ibc::core::channel::types::channel::{ChannelEnd, Counterparty, Order, State};
use ibc::core::channel::types::packet::Packet;
use ibc::core::channel::types::Version;
use ibc::core::client::types::Height;
use ibc::core::commitment_types::commitment::CommitmentPrefix;
use ibc::core::connection::types::version::Version as ConnectionVersion;
use ibc::core::connection::types::{
    ConnectionEnd, Counterparty as ConnectionCounterparty, State as ConnectionState,
};
use ibc::core::handler::types::events::{IbcEvent, MessageEvent};
use ibc::core::host::types::identifiers::{ChannelId, ClientId, ConnectionId, PortId};
use ibc::core::primitives::*;
use ibc_testkit::fixtures::core::channel::dummy_raw_packet;
use ibc_testkit::testapp::ibc::core::types::{MockClientConfig, MockContext};
use test_log::test;

#[test]
fn send_packet_processing() {
    let default_client_id = ClientId::new("07-tendermint", 0).expect("no error");

    struct Test {
        name: String,
        ctx: MockContext,
        packet: Packet,
        want_pass: bool,
    }

    let context = MockContext::default();

    let chan_end_on_a = ChannelEnd::new(
        State::Open,
        Order::Unordered,
        Counterparty::new(PortId::transfer(), Some(ChannelId::zero())),
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

    let timestamp_future = Timestamp::now().add(Duration::from_secs(10)).unwrap();
    let timestamp_ns_past = 1;

    let timeout_height_future = 10;

    let mut packet: Packet =
        dummy_raw_packet(timeout_height_future, timestamp_future.nanoseconds())
            .try_into()
            .unwrap();
    packet.seq_on_a = 1.into();
    packet.data = vec![0];

    let mut packet_with_timestamp_old: Packet =
        dummy_raw_packet(timeout_height_future, timestamp_ns_past)
            .try_into()
            .unwrap();
    packet_with_timestamp_old.seq_on_a = 1.into();
    packet_with_timestamp_old.data = vec![0];

    let client_raw_height = 5;
    let packet_timeout_equal_client_height: Packet =
        dummy_raw_packet(client_raw_height, timestamp_future.nanoseconds())
            .try_into()
            .unwrap();
    let packet_timeout_one_before_client_height: Packet =
        dummy_raw_packet(client_raw_height - 1, timestamp_future.nanoseconds())
            .try_into()
            .unwrap();

    let client_height = Height::new(0, client_raw_height).unwrap();

    let tests: Vec<Test> = vec![
        Test {
            name: "Processing fails because no channel exists in the context".to_string(),
            ctx: context.clone(),
            packet: packet.clone(),
            want_pass: false,
        },
        Test {
            name: "Good parameters".to_string(),
            ctx: context
                .clone()
                .with_client_config(
                    MockClientConfig::builder()
                        .latest_height(client_height)
                        .build(),
                )
                .with_connection(ConnectionId::zero(), conn_end_on_a.clone())
                .with_channel(PortId::transfer(), ChannelId::zero(), chan_end_on_a.clone())
                .with_send_sequence(PortId::transfer(), ChannelId::zero(), 1.into()),
            packet,
            want_pass: true,
        },
        Test {
            name: "Packet timeout height same as destination chain height".to_string(),
            ctx: context
                .clone()
                .with_client_config(
                    MockClientConfig::builder()
                        .latest_height(client_height)
                        .build(),
                )
                .with_connection(ConnectionId::zero(), conn_end_on_a.clone())
                .with_channel(PortId::transfer(), ChannelId::zero(), chan_end_on_a.clone())
                .with_send_sequence(PortId::transfer(), ChannelId::zero(), 1.into()),
            packet: packet_timeout_equal_client_height,
            want_pass: true,
        },
        Test {
            name: "Packet timeout height one more than destination chain height".to_string(),
            ctx: context
                .clone()
                .with_client_config(
                    MockClientConfig::builder()
                        .latest_height(client_height)
                        .build(),
                )
                .with_connection(ConnectionId::zero(), conn_end_on_a.clone())
                .with_channel(PortId::transfer(), ChannelId::zero(), chan_end_on_a.clone())
                .with_send_sequence(PortId::transfer(), ChannelId::zero(), 1.into()),
            packet: packet_timeout_one_before_client_height,
            want_pass: false,
        },
        Test {
            name: "Packet timeout due to timestamp".to_string(),
            ctx: context
                .with_client_config(
                    MockClientConfig::builder()
                        .latest_height(client_height)
                        .build(),
                )
                .with_connection(ConnectionId::zero(), conn_end_on_a)
                .with_channel(PortId::transfer(), ChannelId::zero(), chan_end_on_a)
                .with_send_sequence(PortId::transfer(), ChannelId::zero(), 1.into()),
            packet: packet_with_timestamp_old,
            want_pass: false,
        },
    ]
    .into_iter()
    .collect();

    for mut test in tests {
        let res = send_packet(&mut test.ctx, test.packet.clone());
        // Additionally check the events and the output objects in the result.
        match res {
            Ok(()) => {
                assert!(
                        test.want_pass,
                        "send_packet: test passed but was supposed to fail for test: {}, \nparams {:?} {:?}",
                        test.name,
                        test.packet.clone(),
                        test.ctx.clone()
                    );

                let ibc_events = test.ctx.get_events();

                assert!(!ibc_events.is_empty()); // Some events must exist.

                assert_eq!(ibc_events.len(), 2);
                assert!(matches!(
                    &ibc_events[0],
                    &IbcEvent::Message(MessageEvent::Channel)
                ));
                // TODO: The object in the output is a PacketResult what can we check on it?
                assert!(matches!(&ibc_events[1], &IbcEvent::SendPacket(_)));
            }
            Err(e) => {
                assert!(
                    !test.want_pass,
                    "send_packet: did not pass test: {}, \nparams {:?} {:?} error: {:?}",
                    test.name,
                    test.packet.clone(),
                    test.ctx.clone(),
                    e,
                );
            }
        }
    }
}
