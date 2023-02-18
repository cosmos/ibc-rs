use crate::core::ics04_channel::channel::Counterparty;
use crate::core::ics04_channel::channel::State;
use crate::core::ics04_channel::commitment::PacketCommitment;
use crate::core::ics04_channel::events::SendPacket;
use crate::core::ics04_channel::packet::Sequence;
use crate::core::ics04_channel::{context::SendPacketReader, error::PacketError, packet::Packet};
use crate::core::ics24_host::identifier::{ChannelId, PortId};
use crate::core::ics24_host::path::ChannelEndPath;
use crate::core::ics24_host::path::ClientConsensusStatePath;
use crate::core::ics24_host::path::SeqSendPath;
use crate::core::ContextError;
use crate::events::IbcEvent;
use crate::handler::{HandlerOutput, HandlerResult};
use crate::prelude::*;
use crate::timestamp::Expiry;

#[derive(Clone, Debug)]
pub struct SendPacketResult {
    pub port_id: PortId,
    pub channel_id: ChannelId,
    pub seq: Sequence,
    pub seq_number: Sequence,
    pub commitment: PacketCommitment,
}

/// Per our convention, this message is processed on chain A.
pub fn send_packet(
    ctx_a: &impl SendPacketReader,
    packet: Packet,
) -> HandlerResult<SendPacketResult, ContextError> {
    let mut output = HandlerOutput::builder();

    let chan_end_path_on_a = ChannelEndPath::new(&packet.port_on_a, &packet.chan_on_a);
    let chan_end_on_a = ctx_a.channel_end(&chan_end_path_on_a)?;

    if chan_end_on_a.state_matches(&State::Closed) {
        return Err(PacketError::ChannelClosed {
            channel_id: packet.chan_on_a,
        }
        .into());
    }

    let counterparty = Counterparty::new(packet.port_on_b.clone(), Some(packet.chan_on_b.clone()));

    if !chan_end_on_a.counterparty_matches(&counterparty) {
        return Err(PacketError::InvalidPacketCounterparty {
            port_id: packet.port_on_b.clone(),
            channel_id: packet.chan_on_b,
        }
        .into());
    }
    let conn_id_on_a = &chan_end_on_a.connection_hops()[0];
    let conn_end_on_a = ctx_a.connection_end(conn_id_on_a)?;

    let client_id_on_a = conn_end_on_a.client_id();

    let client_state_of_b_on_a = ctx_a.client_state(client_id_on_a)?;

    // prevent accidental sends with clients that cannot be updated
    if client_state_of_b_on_a.is_frozen() {
        return Err(PacketError::FrozenClient {
            client_id: conn_end_on_a.client_id().clone(),
        }
        .into());
    }

    let latest_height_on_a = client_state_of_b_on_a.latest_height();

    if packet.timeout_height_on_b.has_expired(latest_height_on_a) {
        return Err(PacketError::LowPacketHeight {
            chain_height: latest_height_on_a,
            timeout_height: packet.timeout_height_on_b,
        }
        .into());
    }

    let client_cons_state_path_on_a =
        ClientConsensusStatePath::new(client_id_on_a, &latest_height_on_a);
    let consensus_state_of_b_on_a = ctx_a.client_consensus_state(&client_cons_state_path_on_a)?;
    let latest_timestamp = consensus_state_of_b_on_a.timestamp();
    let packet_timestamp = packet.timeout_timestamp_on_b;
    if let Expiry::Expired = latest_timestamp.check_expiry(&packet_timestamp) {
        return Err(PacketError::LowPacketTimestamp.into());
    }

    let seq_send_path_on_a = SeqSendPath::new(&packet.port_on_a, &packet.chan_on_a);
    let next_seq_send_on_a = ctx_a.get_next_sequence_send(&seq_send_path_on_a)?;

    if packet.sequence != next_seq_send_on_a {
        return Err(PacketError::InvalidPacketSequence {
            given_sequence: packet.sequence,
            next_sequence: next_seq_send_on_a,
        }
        .into());
    }

    output.log("success: packet send ");

    let result = SendPacketResult {
        port_id: packet.port_on_a.clone(),
        channel_id: packet.chan_on_a.clone(),
        seq: packet.sequence,
        seq_number: next_seq_send_on_a.increment(),
        commitment: ctx_a.packet_commitment(
            &packet.data,
            &packet.timeout_height_on_b,
            &packet.timeout_timestamp_on_b,
        ),
    };

    output.emit(IbcEvent::SendPacket(SendPacket::new(
        packet,
        chan_end_on_a.ordering,
        conn_id_on_a.clone(),
    )));

    Ok(output.with_result(result))
}

#[cfg(test)]
mod tests {
    use core::ops::Add;
    use core::time::Duration;

    use test_log::test;

    use crate::core::ics02_client::height::Height;
    use crate::core::ics03_connection::connection::ConnectionEnd;
    use crate::core::ics03_connection::connection::Counterparty as ConnectionCounterparty;
    use crate::core::ics03_connection::connection::State as ConnectionState;
    use crate::core::ics03_connection::version::get_compatible_versions;
    use crate::core::ics04_channel::channel::{ChannelEnd, Counterparty, Order, State};
    use crate::core::ics04_channel::handler::send_packet::send_packet;
    use crate::core::ics04_channel::packet::test_utils::get_dummy_raw_packet;
    use crate::core::ics04_channel::packet::Packet;
    use crate::core::ics04_channel::Version;
    use crate::core::ics24_host::identifier::{ChannelId, ClientId, ConnectionId, PortId};
    use crate::events::IbcEvent;
    use crate::mock::context::MockContext;
    use crate::prelude::*;
    use crate::timestamp::Timestamp;
    use crate::timestamp::ZERO_DURATION;

    #[test]
    fn send_packet_processing() {
        struct Test {
            name: String,
            ctx: MockContext,
            packet: Packet,
            want_pass: bool,
        }

        let context = MockContext::default();

        let chan_end_on_a = ChannelEnd::new(
            State::TryOpen,
            Order::default(),
            Counterparty::new(PortId::default(), Some(ChannelId::default())),
            vec![ConnectionId::default()],
            Version::new("ics20-1".to_string()),
        );

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
        );

        let timestamp_future = Timestamp::now().add(Duration::from_secs(10)).unwrap();
        let timestamp_ns_past = 1;

        let timeout_height_future = 10;

        let mut packet: Packet =
            get_dummy_raw_packet(timeout_height_future, timestamp_future.nanoseconds())
                .try_into()
                .unwrap();
        packet.sequence = 1.into();
        packet.data = vec![0];

        let mut packet_with_timestamp_old: Packet =
            get_dummy_raw_packet(timeout_height_future, timestamp_ns_past)
                .try_into()
                .unwrap();
        packet_with_timestamp_old.sequence = 1.into();
        packet_with_timestamp_old.data = vec![0];

        let client_raw_height = 5;
        let packet_timeout_equal_client_height: Packet =
            get_dummy_raw_packet(client_raw_height, timestamp_future.nanoseconds())
                .try_into()
                .unwrap();
        let packet_timeout_one_before_client_height: Packet =
            get_dummy_raw_packet(client_raw_height - 1, timestamp_future.nanoseconds())
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
                    .with_client(&ClientId::default(), client_height)
                    .with_connection(ConnectionId::default(), conn_end_on_a.clone())
                    .with_channel(
                        PortId::default(),
                        ChannelId::default(),
                        chan_end_on_a.clone(),
                    )
                    .with_send_sequence(PortId::default(), ChannelId::default(), 1.into()),
                packet,
                want_pass: true,
            },
            Test {
                name: "Packet timeout height same as destination chain height".to_string(),
                ctx: context
                    .clone()
                    .with_client(&ClientId::default(), client_height)
                    .with_connection(ConnectionId::default(), conn_end_on_a.clone())
                    .with_channel(
                        PortId::default(),
                        ChannelId::default(),
                        chan_end_on_a.clone(),
                    )
                    .with_send_sequence(PortId::default(), ChannelId::default(), 1.into()),
                packet: packet_timeout_equal_client_height,
                want_pass: true,
            },
            Test {
                name: "Packet timeout height one more than destination chain height".to_string(),
                ctx: context
                    .clone()
                    .with_client(&ClientId::default(), client_height)
                    .with_connection(ConnectionId::default(), conn_end_on_a.clone())
                    .with_channel(
                        PortId::default(),
                        ChannelId::default(),
                        chan_end_on_a.clone(),
                    )
                    .with_send_sequence(PortId::default(), ChannelId::default(), 1.into()),
                packet: packet_timeout_one_before_client_height,
                want_pass: false,
            },
            Test {
                name: "Packet timeout due to timestamp".to_string(),
                ctx: context
                    .with_client(&ClientId::default(), client_height)
                    .with_connection(ConnectionId::default(), conn_end_on_a)
                    .with_channel(PortId::default(), ChannelId::default(), chan_end_on_a)
                    .with_send_sequence(PortId::default(), ChannelId::default(), 1.into()),
                packet: packet_with_timestamp_old,
                want_pass: false,
            },
        ]
        .into_iter()
        .collect();

        for test in tests {
            let res = send_packet(&test.ctx, test.packet.clone());
            // Additionally check the events and the output objects in the result.
            match res {
                Ok(proto_output) => {
                    assert!(
                        test.want_pass,
                        "send_packet: test passed but was supposed to fail for test: {}, \nparams {:?} {:?}",
                        test.name,
                        test.packet.clone(),
                        test.ctx.clone()
                    );

                    assert!(!proto_output.events.is_empty()); // Some events must exist.

                    // TODO: The object in the output is a PacketResult what can we check on it?
                    for e in proto_output.events.iter() {
                        assert!(matches!(e, &IbcEvent::SendPacket(_)));
                    }
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
}
