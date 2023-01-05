use crate::core::ics03_connection::connection::State as ConnectionState;
use crate::core::ics04_channel::channel::{Counterparty, Order, State};
use crate::core::ics04_channel::context::ChannelReader;
use crate::core::ics04_channel::error::ChannelError;
use crate::core::ics04_channel::error::PacketError;
use crate::core::ics04_channel::events::ReceivePacket;
use crate::core::ics04_channel::msgs::recv_packet::MsgRecvPacket;
use crate::core::ics04_channel::packet::{PacketResult, Receipt, Sequence};
use crate::core::ics24_host::identifier::{ChannelId, PortId};
use crate::events::IbcEvent;
use crate::handler::{HandlerOutput, HandlerResult};
use crate::timestamp::Expiry;
use alloc::string::ToString;

#[derive(Clone, Debug)]
pub enum RecvPacketResult {
    NoOp,
    Unordered {
        port_id: PortId,
        channel_id: ChannelId,
        sequence: Sequence,
        receipt: Receipt,
    },
    Ordered {
        port_id: PortId,
        channel_id: ChannelId,
        next_seq_recv: Sequence,
    },
}

pub fn process<Ctx: ChannelReader>(
    ctx_b: &Ctx,
    msg: &MsgRecvPacket,
) -> HandlerResult<PacketResult, PacketError> {
    let mut output = HandlerOutput::builder();

    let packet = &msg.packet;

    let chan_end_on_b = ctx_b
        .channel_end(&packet.destination_port, &packet.destination_channel)
        .map_err(PacketError::Channel)?;

    if !chan_end_on_b.state_matches(&State::Open) {
        return Err(PacketError::InvalidChannelState {
            channel_id: packet.source_channel.clone(),
            state: chan_end_on_b.state,
        });
    }

    let counterparty = Counterparty::new(
        packet.source_port.clone(),
        Some(packet.source_channel.clone()),
    );

    if !chan_end_on_b.counterparty_matches(&counterparty) {
        return Err(PacketError::InvalidPacketCounterparty {
            port_id: packet.source_port.clone(),
            channel_id: packet.source_channel.clone(),
        });
    }

    let conn_id_on_b = &chan_end_on_b.connection_hops()[0];
    let conn_end_on_b = ctx_b
        .connection_end(conn_id_on_b)
        .map_err(PacketError::Channel)?;

    if !conn_end_on_b.state_matches(&ConnectionState::Open) {
        return Err(PacketError::ConnectionNotOpen {
            connection_id: chan_end_on_b.connection_hops()[0].clone(),
        });
    }

    let latest_height = ChannelReader::host_height(ctx_b).map_err(PacketError::Channel)?;
    if packet.timeout_height.has_expired(latest_height) {
        return Err(PacketError::LowPacketHeight {
            chain_height: latest_height,
            timeout_height: packet.timeout_height,
        });
    }

    let latest_timestamp = ChannelReader::host_timestamp(ctx_b).map_err(PacketError::Channel)?;
    if let Expiry::Expired = latest_timestamp.check_expiry(&packet.timeout_timestamp) {
        return Err(PacketError::LowPacketTimestamp);
    }

    // Verify proofs
    {
        let client_id_on_b = conn_end_on_b.client_id();
        let client_state_of_a_on_b = ctx_b
            .client_state(client_id_on_b)
            .map_err(PacketError::Channel)?;

        // The client must not be frozen.
        if client_state_of_a_on_b.is_frozen() {
            return Err(PacketError::FrozenClient {
                client_id: client_id_on_b.clone(),
            });
        }

        let consensus_state_of_a_on_b = ctx_b
            .client_consensus_state(client_id_on_b, &msg.proofs.height())
            .map_err(PacketError::Channel)?;

        let commitment = ctx_b.packet_commitment(
            &packet.data,
            &packet.timeout_height,
            &packet.timeout_timestamp,
        );
        // Verify the proof for the packet against the chain store.
        client_state_of_a_on_b
            .verify_packet_data(
                ctx_b,
                msg.proofs.height(),
                &conn_end_on_b,
                msg.proofs.object_proof(),
                consensus_state_of_a_on_b.root(),
                &packet.source_port,
                &packet.source_channel,
                packet.sequence,
                commitment,
            )
            .map_err(|e| ChannelError::PacketVerificationFailed {
                sequence: packet.sequence,
                client_error: e,
            })
            .map_err(PacketError::Channel)?;
    }

    let result = if chan_end_on_b.order_matches(&Order::Ordered) {
        let next_seq_recv =
            ctx_b.get_next_sequence_recv(&packet.destination_port, &packet.destination_channel)?;
        if packet.sequence > next_seq_recv {
            return Err(PacketError::InvalidPacketSequence {
                given_sequence: packet.sequence,
                next_sequence: next_seq_recv,
            });
        }

        if packet.sequence < next_seq_recv {
            PacketResult::Recv(RecvPacketResult::NoOp)
        } else {
            PacketResult::Recv(RecvPacketResult::Ordered {
                port_id: packet.destination_port.clone(),
                channel_id: packet.destination_channel.clone(),
                next_seq_recv: next_seq_recv.increment(),
            })
        }
    } else {
        let packet_rec = ctx_b.get_packet_receipt(
            &packet.destination_port,
            &packet.destination_channel,
            &packet.sequence,
        );

        match packet_rec {
            Ok(_receipt) => PacketResult::Recv(RecvPacketResult::NoOp),
            Err(e)
                if e.to_string()
                    == PacketError::PacketReceiptNotFound {
                        sequence: packet.sequence,
                    }
                    .to_string() =>
            {
                // store a receipt that does not contain any data
                PacketResult::Recv(RecvPacketResult::Unordered {
                    port_id: packet.destination_port.clone(),
                    channel_id: packet.destination_channel.clone(),
                    sequence: packet.sequence,
                    receipt: Receipt::Ok,
                })
            }
            Err(_) => return Err(PacketError::ImplementationSpecific),
        }
    };

    output.log("success: packet receive");

    output.emit(IbcEvent::ReceivePacket(ReceivePacket::new(
        msg.packet.clone(),
        chan_end_on_b.ordering,
        conn_id_on_b.clone(),
    )));

    Ok(output.with_result(result))
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    use test_log::test;

    use crate::core::ics03_connection::connection::ConnectionEnd;
    use crate::core::ics03_connection::connection::Counterparty as ConnectionCounterparty;
    use crate::core::ics03_connection::connection::State as ConnectionState;
    use crate::core::ics03_connection::version::get_compatible_versions;
    use crate::core::ics04_channel::channel::{ChannelEnd, Counterparty, Order, State};
    use crate::core::ics04_channel::handler::recv_packet::process;
    use crate::core::ics04_channel::msgs::recv_packet::test_util::get_dummy_raw_msg_recv_packet;
    use crate::core::ics04_channel::msgs::recv_packet::MsgRecvPacket;
    use crate::core::ics04_channel::Version;
    use crate::core::ics24_host::identifier::{ChannelId, ClientId, ConnectionId, PortId};
    use crate::mock::context::MockContext;
    use crate::mock::ics18_relayer::context::RelayerContext;
    use crate::test_utils::get_dummy_account_id;
    use crate::timestamp::Timestamp;
    use crate::timestamp::ZERO_DURATION;
    use crate::{core::ics04_channel::packet::Packet, events::IbcEvent};

    #[test]
    fn recv_packet_processing() {
        struct Test {
            name: String,
            ctx: MockContext,
            msg: MsgRecvPacket,
            want_pass: bool,
        }

        let context = MockContext::default();

        let host_height = context.query_latest_height().unwrap().increment();

        let client_height = host_height.increment();

        let msg = MsgRecvPacket::try_from(get_dummy_raw_msg_recv_packet(
            client_height.revision_height(),
        ))
        .unwrap();

        let packet = msg.packet.clone();

        let packet_old = Packet {
            sequence: 1.into(),
            source_port: PortId::default(),
            source_channel: ChannelId::default(),
            destination_port: PortId::default(),
            destination_channel: ChannelId::default(),
            data: Vec::new(),
            timeout_height: client_height.into(),
            timeout_timestamp: Timestamp::from_nanoseconds(1).unwrap(),
        };

        let msg_packet_old =
            MsgRecvPacket::new(packet_old, msg.proofs.clone(), get_dummy_account_id());

        let chan_end_on_b = ChannelEnd::new(
            State::Open,
            Order::default(),
            Counterparty::new(packet.source_port.clone(), Some(packet.source_channel)),
            vec![ConnectionId::default()],
            Version::ics20(),
        );

        let conn_end_on_b = ConnectionEnd::new(
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

        let tests: Vec<Test> = vec![
            Test {
                name: "Processing fails because no channel exists in the context".to_string(),
                ctx: context.clone(),
                msg: msg.clone(),
                want_pass: false,
            },
            Test {
                name: "Good parameters".to_string(),
                ctx: context
                    .clone()
                    .with_client(&ClientId::default(), client_height)
                    .with_connection(ConnectionId::default(), conn_end_on_b.clone())
                    .with_channel(
                        packet.destination_port.clone(),
                        packet.destination_channel.clone(),
                        chan_end_on_b.clone(),
                    )
                    .with_send_sequence(
                        packet.destination_port.clone(),
                        packet.destination_channel.clone(),
                        1.into(),
                    )
                    .with_height(host_height)
                    // This `with_recv_sequence` is required for ordered channels
                    .with_recv_sequence(
                        packet.destination_port.clone(),
                        packet.destination_channel.clone(),
                        packet.sequence,
                    ),
                msg,
                want_pass: true,
            },
            Test {
                name: "Packet timeout expired".to_string(),
                ctx: context
                    .with_client(&ClientId::default(), client_height)
                    .with_connection(ConnectionId::default(), conn_end_on_b)
                    .with_channel(PortId::default(), ChannelId::default(), chan_end_on_b)
                    .with_send_sequence(PortId::default(), ChannelId::default(), 1.into())
                    .with_height(host_height),
                msg: msg_packet_old,
                want_pass: false,
            },
        ]
        .into_iter()
        .collect();

        for test in tests {
            let res = process(&test.ctx, &test.msg);
            // Additionally check the events and the output objects in the result.
            match res {
                Ok(proto_output) => {
                    assert!(
                            test.want_pass,
                            "recv_packet: test passed but was supposed to fail for test: {}, \nparams \n msg={:?}\nctx:{:?}",
                            test.name,
                            test.msg.clone(),
                            test.ctx.clone()
                        );

                    assert!(!proto_output.events.is_empty()); // Some events must exist.

                    for e in proto_output.events.iter() {
                        assert!(matches!(e, &IbcEvent::ReceivePacket(_)));
                    }
                }
                Err(e) => {
                    assert!(
                            !test.want_pass,
                            "recv_packet: did not pass test: {}, \nparams \nmsg={:?}\nctx={:?}\nerror={:?}",
                            test.name,
                            test.msg.clone(),
                            test.ctx.clone(),
                            e,
                        );
                }
            }
        }
    }
}
