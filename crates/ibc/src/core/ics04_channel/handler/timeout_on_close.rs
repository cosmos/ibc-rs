use crate::core::ics04_channel::channel::State;
use crate::core::ics04_channel::channel::{ChannelEnd, Counterparty, Order};
use crate::core::ics04_channel::error::ChannelError;
use crate::core::ics04_channel::events::{ChannelClosed, TimeoutPacket};
use crate::core::ics04_channel::msgs::timeout_on_close::MsgTimeoutOnClose;
use crate::core::ics04_channel::packet::PacketResult;
use crate::core::ics04_channel::{
    context::ChannelReader, error::PacketError, handler::timeout::TimeoutPacketResult,
};
use crate::events::IbcEvent;
use crate::handler::{HandlerOutput, HandlerResult};
use crate::prelude::*;

pub fn process<Ctx: ChannelReader>(
    ctx_a: &Ctx,
    msg: &MsgTimeoutOnClose,
) -> HandlerResult<PacketResult, PacketError> {
    let mut output = HandlerOutput::builder();

    let packet = &msg.packet;

    let chan_end_on_a = ctx_a
        .channel_end(&packet.port_on_a, &packet.chan_on_a)
        .map_err(PacketError::Channel)?;

    let counterparty = Counterparty::new(packet.port_on_b.clone(), Some(packet.chan_on_b.clone()));

    if !chan_end_on_a.counterparty_matches(&counterparty) {
        return Err(PacketError::InvalidPacketCounterparty {
            port_id: packet.port_on_b.clone(),
            channel_id: packet.chan_on_b.clone(),
        });
    }

    //verify the packet was sent, check the store
    let commitment_on_a =
        ctx_a.get_packet_commitment(&packet.port_on_a, &packet.chan_on_a, &packet.sequence)?;

    let expected_commitment_on_a = ctx_a.packet_commitment(
        &packet.data,
        &packet.timeout_height_on_b,
        &packet.timeout_timestamp_on_b,
    );
    if commitment_on_a != expected_commitment_on_a {
        return Err(PacketError::IncorrectPacketCommitment {
            sequence: packet.sequence,
        });
    }

    let conn_id_on_a = chan_end_on_a.connection_hops()[0].clone();
    let conn_end_on_a = ctx_a
        .connection_end(&conn_id_on_a)
        .map_err(PacketError::Channel)?;

    // Verify proofs
    {
        let client_id_on_a = conn_end_on_a.client_id().clone();
        let client_state_of_b_on_a = ctx_a
            .client_state(&client_id_on_a)
            .map_err(PacketError::Channel)?;

        // The client must not be frozen.
        if client_state_of_b_on_a.is_frozen() {
            return Err(PacketError::FrozenClient {
                client_id: client_id_on_a,
            });
        }

        let consensus_state_of_b_on_a = ctx_a
            .client_consensus_state(&client_id_on_a, &msg.proof_height_on_b)
            .map_err(PacketError::Channel)?;
        let prefix_on_b = conn_end_on_a.counterparty().prefix();
        let port_id_on_b = &chan_end_on_a.counterparty().port_id;
        let chan_id_on_b =
            chan_end_on_a
                .counterparty()
                .channel_id()
                .ok_or(PacketError::Channel(
                    ChannelError::InvalidCounterpartyChannelId,
                ))?;
        let conn_id_on_b = conn_end_on_a.counterparty().connection_id().ok_or(
            PacketError::UndefinedConnectionCounterparty {
                connection_id: chan_end_on_a.connection_hops()[0].clone(),
            },
        )?;
        let expected_conn_hops_on_b = vec![conn_id_on_b.clone()];
        let expected_counterparty =
            Counterparty::new(packet.port_on_a.clone(), Some(packet.chan_on_a.clone()));
        let expected_chan_end_on_b = ChannelEnd::new(
            State::Closed,
            *chan_end_on_a.ordering(),
            expected_counterparty,
            expected_conn_hops_on_b,
            chan_end_on_a.version().clone(),
        );

        // Verify the proof for the channel state against the expected channel end.
        // A counterparty channel id of None in not possible, and is checked by validate_basic in msg.
        client_state_of_b_on_a
            .verify_channel_state(
                msg.proof_height_on_b,
                prefix_on_b,
                &msg.proof_unreceived_on_b,
                consensus_state_of_b_on_a.root(),
                port_id_on_b,
                chan_id_on_b,
                &expected_chan_end_on_b,
            )
            .map_err(ChannelError::VerifyChannelFailed)
            .map_err(PacketError::Channel)?;

        let next_seq_recv_verification_result = if chan_end_on_a.order_matches(&Order::Ordered) {
            if packet.sequence < msg.next_seq_recv_on_b {
                return Err(PacketError::InvalidPacketSequence {
                    given_sequence: packet.sequence,
                    next_sequence: msg.next_seq_recv_on_b,
                });
            }
            client_state_of_b_on_a.verify_next_sequence_recv(
                ctx_a,
                msg.proof_height_on_b,
                &conn_end_on_a,
                &msg.proof_unreceived_on_b,
                consensus_state_of_b_on_a.root(),
                &packet.port_on_b,
                &packet.chan_on_b,
                packet.sequence,
            )
        } else {
            client_state_of_b_on_a.verify_packet_receipt_absence(
                ctx_a,
                msg.proof_height_on_b,
                &conn_end_on_a,
                &msg.proof_unreceived_on_b,
                consensus_state_of_b_on_a.root(),
                &packet.port_on_b,
                &packet.chan_on_b,
                packet.sequence,
            )
        };
        next_seq_recv_verification_result
            .map_err(|e| ChannelError::PacketVerificationFailed {
                sequence: msg.next_seq_recv_on_b,
                client_error: e,
            })
            .map_err(PacketError::Channel)?;
    };

    output.log("success: packet timeout");

    output.emit(IbcEvent::TimeoutPacket(TimeoutPacket::new(
        packet.clone(),
        chan_end_on_a.ordering,
    )));

    let packet_res_chan = if chan_end_on_a.order_matches(&Order::Ordered) {
        output.emit(IbcEvent::ChannelClosed(ChannelClosed::new(
            msg.packet.port_on_a.clone(),
            msg.packet.chan_on_a.clone(),
            chan_end_on_a.counterparty().port_id.clone(),
            chan_end_on_a.counterparty().channel_id.clone(),
            conn_id_on_a,
            chan_end_on_a.ordering,
        )));
        Some(chan_end_on_a)
    } else {
        None
    };

    let result = PacketResult::Timeout(TimeoutPacketResult {
        port_id: packet.port_on_a.clone(),
        channel_id: packet.chan_on_a.clone(),
        seq: packet.sequence,
        channel: packet_res_chan,
    });

    Ok(output.with_result(result))
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use test_log::test;

    use crate::core::ics02_client::height::Height;
    use crate::core::ics03_connection::connection::ConnectionEnd;
    use crate::core::ics03_connection::connection::Counterparty as ConnectionCounterparty;
    use crate::core::ics03_connection::connection::State as ConnectionState;
    use crate::core::ics03_connection::version::get_compatible_versions;
    use crate::core::ics04_channel::channel::{ChannelEnd, Counterparty, Order, State};
    use crate::core::ics04_channel::context::ChannelReader;
    use crate::core::ics04_channel::handler::timeout_on_close::process;
    use crate::core::ics04_channel::msgs::timeout_on_close::test_util::get_dummy_raw_msg_timeout_on_close;
    use crate::core::ics04_channel::msgs::timeout_on_close::MsgTimeoutOnClose;
    use crate::core::ics04_channel::Version;
    use crate::core::ics24_host::identifier::{ChannelId, ClientId, ConnectionId, PortId};
    use crate::events::IbcEvent;
    use crate::mock::context::MockContext;
    use crate::timestamp::ZERO_DURATION;

    #[test]
    fn timeout_on_close_packet_processing() {
        struct Test {
            name: String,
            ctx: MockContext,
            msg: MsgTimeoutOnClose,
            want_pass: bool,
        }

        let context = MockContext::default();

        let height = 2;
        let timeout_timestamp = 5;

        let client_height = Height::new(0, 2).unwrap();

        let msg = MsgTimeoutOnClose::try_from(get_dummy_raw_msg_timeout_on_close(
            height,
            timeout_timestamp,
        ))
        .unwrap();
        let packet = msg.packet.clone();

        let data = context.packet_commitment(
            &msg.packet.data,
            &msg.packet.timeout_height_on_b,
            &msg.packet.timeout_timestamp_on_b,
        );

        let chan_end_on_a = ChannelEnd::new(
            State::Open,
            Order::Ordered,
            Counterparty::new(packet.port_on_b.clone(), Some(packet.chan_on_b)),
            vec![ConnectionId::default()],
            Version::ics20(),
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

        let tests: Vec<Test> = vec![
            Test {
                name: "Processing fails because no channel exists in the context".to_string(),
                ctx: context.clone(),
                msg: msg.clone(),
                want_pass: false,
            },
            Test {
                name: "Processing fails no packet commitment is found".to_string(),
                ctx: context
                    .clone()
                    .with_channel(
                        PortId::default(),
                        ChannelId::default(),
                        chan_end_on_a.clone(),
                    )
                    .with_connection(ConnectionId::default(), conn_end_on_a.clone()),
                msg: msg.clone(),
                want_pass: false,
            },
            Test {
                name: "Good parameters".to_string(),
                ctx: context
                    .with_client(&ClientId::default(), client_height)
                    .with_connection(ConnectionId::default(), conn_end_on_a)
                    .with_channel(packet.port_on_a, packet.chan_on_a, chan_end_on_a)
                    .with_packet_commitment(
                        msg.packet.port_on_a.clone(),
                        msg.packet.chan_on_a.clone(),
                        msg.packet.sequence,
                        data,
                    ),
                msg: msg.clone(),
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
                        "TO_on_close_packet: test passed but was supposed to fail for test: {}, \nparams {:?} {:?}",
                        test.name,
                        test.msg.clone(),
                        test.ctx.clone()
                    );

                    let events = proto_output.events;
                    let src_channel_end = test
                        .ctx
                        .channel_end(&msg.packet.port_on_a, &msg.packet.chan_on_a)
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
