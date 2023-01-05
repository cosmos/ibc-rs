use crate::core::ics04_channel::channel::State;
use crate::core::ics04_channel::channel::{ChannelEnd, Counterparty, Order};
use crate::core::ics04_channel::error::ChannelError;
use crate::core::ics04_channel::events::{ChannelClosed, TimeoutPacket};
use crate::core::ics04_channel::msgs::timeout::MsgTimeout;
use crate::core::ics04_channel::packet::{PacketResult, Sequence};
use crate::core::ics04_channel::{context::ChannelReader, error::PacketError};
use crate::core::ics24_host::identifier::{ChannelId, PortId};
use crate::events::IbcEvent;
use crate::handler::{HandlerOutput, HandlerResult};
use crate::prelude::*;
use crate::timestamp::Expiry;

#[derive(Clone, Debug)]
pub struct TimeoutPacketResult {
    pub port_id: PortId,
    pub channel_id: ChannelId,
    pub seq: Sequence,
    pub channel: Option<ChannelEnd>,
}

/// TimeoutPacket is called by a module which originally attempted to send a
/// packet to a counterparty module, where the timeout height has passed on the
/// counterparty chain without the packet being committed, to prove that the
/// packet can no longer be executed and to allow the calling module to safely
/// perform appropriate state transitions.
pub fn process<Ctx: ChannelReader>(
    ctx_a: &Ctx,
    msg: &MsgTimeout,
) -> HandlerResult<PacketResult, PacketError> {
    let mut output = HandlerOutput::builder();

    let packet = &msg.packet;

    let mut chan_end_on_a = ctx_a
        .channel_end(&packet.source_port, &packet.source_channel)
        .map_err(PacketError::Channel)?;

    if !chan_end_on_a.state_matches(&State::Open) {
        return Err(PacketError::ChannelClosed {
            channel_id: packet.source_channel.clone(),
        });
    }

    let counterparty = Counterparty::new(
        packet.destination_port.clone(),
        Some(packet.destination_channel.clone()),
    );

    if !chan_end_on_a.counterparty_matches(&counterparty) {
        return Err(PacketError::InvalidPacketCounterparty {
            port_id: packet.destination_port.clone(),
            channel_id: packet.destination_channel.clone(),
        });
    }

    let conn_id_on_a = chan_end_on_a.connection_hops()[0].clone();
    let conn_end_on_a = ctx_a
        .connection_end(&conn_id_on_a)
        .map_err(PacketError::Channel)?;

    //verify packet commitment
    let packet_commitment = ctx_a.get_packet_commitment(
        &packet.source_port,
        &packet.source_channel,
        &packet.sequence,
    )?;

    let expected_commitment = ctx_a.packet_commitment(
        &packet.data,
        &packet.timeout_height,
        &packet.timeout_timestamp,
    );
    if packet_commitment != expected_commitment {
        return Err(PacketError::IncorrectPacketCommitment {
            sequence: packet.sequence,
        });
    }

    // Verify proofs
    {
        let client_id_on_a = conn_end_on_a.client_id().clone();
        let client_state_of_b_on_a = ctx_a
            .client_state(&client_id_on_a)
            .map_err(PacketError::Channel)?;

        // check that timeout height or timeout timestamp has passed on the other end
        let proof_height = msg.proof_height_on_b;

        if packet.timeout_height.has_expired(proof_height) {
            return Err(PacketError::PacketTimeoutHeightNotReached {
                timeout_height: packet.timeout_height,
                chain_height: proof_height,
            });
        }

        let consensus_state_of_b_on_a = ctx_a
            .client_consensus_state(&client_id_on_a, &proof_height)
            .map_err(PacketError::Channel)?;
        let proof_timestamp = consensus_state_of_b_on_a.timestamp();

        let packet_timestamp = packet.timeout_timestamp;
        if let Expiry::Expired = packet_timestamp.check_expiry(&proof_timestamp) {
            return Err(PacketError::PacketTimeoutTimestampNotReached {
                timeout_timestamp: packet_timestamp,
                chain_timestamp: proof_timestamp,
            });
        }
        let verify_res = if chan_end_on_a.order_matches(&Order::Ordered) {
            if packet.sequence < msg.next_sequence_recv {
                return Err(PacketError::InvalidPacketSequence {
                    given_sequence: packet.sequence,
                    next_sequence: msg.next_sequence_recv,
                });
            }
            client_state_of_b_on_a.verify_next_sequence_recv(
                ctx_a,
                msg.proof_height_on_b,
                &conn_end_on_a,
                &msg.proof_unreceived_on_b,
                consensus_state_of_b_on_a.root(),
                &packet.destination_port,
                &packet.destination_channel,
                packet.sequence,
            )
        } else {
            client_state_of_b_on_a.verify_packet_receipt_absence(
                ctx_a,
                msg.proof_height_on_b,
                &conn_end_on_a,
                &msg.proof_unreceived_on_b,
                consensus_state_of_b_on_a.root(),
                &packet.destination_port,
                &packet.destination_channel,
                packet.sequence,
            )
        };
        verify_res
            .map_err(|e| ChannelError::PacketVerificationFailed {
                sequence: msg.next_sequence_recv,
                client_error: e,
            })
            .map_err(PacketError::Channel)?;
    }

    output.log("success: packet timeout ");

    output.emit(IbcEvent::TimeoutPacket(TimeoutPacket::new(
        packet.clone(),
        chan_end_on_a.ordering,
    )));

    let packet_res_chan = if chan_end_on_a.order_matches(&Order::Ordered) {
        output.emit(IbcEvent::ChannelClosed(ChannelClosed::new(
            msg.packet.source_port.clone(),
            msg.packet.source_channel.clone(),
            chan_end_on_a.counterparty().port_id.clone(),
            chan_end_on_a.counterparty().channel_id.clone(),
            conn_id_on_a,
            chan_end_on_a.ordering,
        )));
        chan_end_on_a.state = State::Closed;
        Some(chan_end_on_a)
    } else {
        None
    };

    let result = PacketResult::Timeout(TimeoutPacketResult {
        port_id: packet.source_port.clone(),
        channel_id: packet.source_channel.clone(),
        seq: packet.sequence,
        channel: packet_res_chan,
    });

    Ok(output.with_result(result))
}

#[cfg(test)]
mod tests {
    use test_log::test;

    use crate::core::ics02_client::height::Height;
    use crate::core::ics03_connection::connection::ConnectionEnd;
    use crate::core::ics03_connection::connection::Counterparty as ConnectionCounterparty;
    use crate::core::ics03_connection::connection::State as ConnectionState;
    use crate::core::ics03_connection::version::get_compatible_versions;
    use crate::core::ics04_channel::channel::{ChannelEnd, Counterparty, Order, State};
    use crate::core::ics04_channel::context::ChannelReader;
    use crate::core::ics04_channel::handler::timeout::process;
    use crate::core::ics04_channel::msgs::timeout::test_util::get_dummy_raw_msg_timeout;
    use crate::core::ics04_channel::msgs::timeout::MsgTimeout;
    use crate::core::ics04_channel::Version;
    use crate::core::ics24_host::identifier::{ChannelId, ClientId, ConnectionId, PortId};
    use crate::events::IbcEvent;
    use crate::mock::context::MockContext;
    use crate::prelude::*;
    use crate::timestamp::ZERO_DURATION;

    #[test]
    fn timeout_packet_processing() {
        struct Test {
            name: String,
            ctx: MockContext,
            msg: MsgTimeout,
            want_pass: bool,
        }

        let context = MockContext::default();

        let msg_proof_height = 2;
        let msg_timeout_height = 5;
        let timeout_timestamp = 5;

        let client_height = Height::new(0, 2).unwrap();

        let msg = MsgTimeout::try_from(get_dummy_raw_msg_timeout(
            msg_proof_height,
            msg_timeout_height,
            timeout_timestamp,
        ))
        .unwrap();
        let packet = msg.packet.clone();

        let mut msg_ok = msg.clone();
        msg_ok.packet.timeout_timestamp = Default::default();

        let data = context.packet_commitment(
            &msg_ok.packet.data,
            &msg_ok.packet.timeout_height,
            &msg_ok.packet.timeout_timestamp,
        );

        let chan_end_on_a = ChannelEnd::new(
            State::Open,
            Order::default(),
            Counterparty::new(
                packet.destination_port.clone(),
                Some(packet.destination_channel.clone()),
            ),
            vec![ConnectionId::default()],
            Version::ics20(),
        );

        let mut source_ordered_channel_end = chan_end_on_a.clone();
        source_ordered_channel_end.ordering = Order::Ordered;

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

        let tests: Vec<Test> = vec![
            Test {
                name: "Processing fails because no channel exists in the context".to_string(),
                ctx: context.clone(),
                msg: msg.clone(),
                want_pass: false,
            },
            Test {
                name: "Processing fails because the client does not have a consensus state for the required height"
                    .to_string(),
                ctx: context.clone().with_channel(
                    PortId::default(),
                    ChannelId::default(),
                    chan_end_on_a.clone(),
                )
                .with_connection(ConnectionId::default(), conn_end_on_a.clone()),
                msg: msg.clone(),
                want_pass: false,
            },
            Test {
                name: "Processing fails because the proof's timeout has not been reached "
                    .to_string(),
                ctx: context.clone().with_channel(
                    PortId::default(),
                    ChannelId::default(),
                    chan_end_on_a.clone(),
                )
                .with_client(&ClientId::default(), client_height)
                .with_connection(ConnectionId::default(), conn_end_on_a.clone()),
                msg,
                want_pass: false,
            },
            Test {
                name: "Good parameters Unordered channel".to_string(),
                ctx: context.clone()
                    .with_client(&ClientId::default(), client_height)
                    .with_connection(ConnectionId::default(), conn_end_on_a.clone())
                    .with_channel(
                        packet.source_port.clone(),
                        packet.source_channel.clone(),
                        chan_end_on_a,
                    )
                    .with_packet_commitment(
                        msg_ok.packet.source_port.clone(),
                        msg_ok.packet.source_channel.clone(),
                        msg_ok.packet.sequence,
                        data.clone(),
                    ),
                msg: msg_ok.clone(),
                want_pass: true,
            },
            Test {
                name: "Good parameters Ordered Channel".to_string(),
                ctx: context
                    .with_client(&ClientId::default(), client_height)
                    .with_connection(ConnectionId::default(), conn_end_on_a)
                    .with_channel(
                        packet.source_port.clone(),
                        packet.source_channel.clone(),
                        source_ordered_channel_end,
                    )
                    .with_packet_commitment(
                        msg_ok.packet.source_port.clone(),
                        msg_ok.packet.source_channel.clone(),
                        msg_ok.packet.sequence,
                        data,
                    )
                    .with_ack_sequence(
                         packet.destination_port,
                         packet.destination_channel,
                         1.into(),
                     ),
                msg: msg_ok,
                want_pass: true,
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
                        "TO_packet: test passed but was supposed to fail for test: {}, \nparams {:?} {:?}",
                        test.name,
                        test.msg.clone(),
                        test.ctx.clone()
                    );

                    let events = proto_output.events;
                    let src_channel_end = test
                        .ctx
                        .channel_end(&packet.source_port, &packet.source_channel)
                        .unwrap();

                    if src_channel_end.order_matches(&Order::Ordered) {
                        assert_eq!(events.len(), 2);

                        assert!(matches!(events[0], IbcEvent::TimeoutPacket(_)));
                        assert!(matches!(events[1], IbcEvent::ChannelClosed(_)));
                    } else {
                        assert_eq!(events.len(), 1);
                        assert!(matches!(
                            events.first().unwrap(),
                            &IbcEvent::TimeoutPacket(_)
                        ));
                    }
                }
                Err(e) => {
                    assert!(
                        !test.want_pass,
                        "timeout_packet: did not pass test: {}, \nparams {:?} {:?} error: {:?}",
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
