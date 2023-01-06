use crate::core::ics03_connection::connection::State as ConnectionState;
use crate::core::ics04_channel::channel::State;
use crate::core::ics04_channel::channel::{Counterparty, Order};
use crate::core::ics04_channel::error::ChannelError;
use crate::core::ics04_channel::events::AcknowledgePacket;
use crate::core::ics04_channel::msgs::acknowledgement::MsgAcknowledgement;
use crate::core::ics04_channel::packet::{PacketResult, Sequence};
use crate::core::ics04_channel::{context::ChannelReader, error::PacketError};
use crate::core::ics24_host::identifier::{ChannelId, PortId};
use crate::events::IbcEvent;
use crate::handler::{HandlerOutput, HandlerResult};
use crate::prelude::*;

#[derive(Clone, Debug)]
pub struct AckPacketResult {
    pub port_id: PortId,
    pub channel_id: ChannelId,
    pub seq: Sequence,
    pub seq_number: Option<Sequence>,
}

pub fn process<Ctx: ChannelReader>(
    ctx_a: &Ctx,
    msg: &MsgAcknowledgement,
) -> HandlerResult<PacketResult, PacketError> {
    let mut output = HandlerOutput::builder();

    let packet = &msg.packet;

    let chan_end_on_a = ctx_a
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

    let conn_id_on_a = &chan_end_on_a.connection_hops()[0];
    let conn_end_on_a = ctx_a
        .connection_end(conn_id_on_a)
        .map_err(PacketError::Channel)?;

    if !conn_end_on_a.state_matches(&ConnectionState::Open) {
        return Err(PacketError::ConnectionNotOpen {
            connection_id: chan_end_on_a.connection_hops()[0].clone(),
        });
    }

    // Verify packet commitment
    let packet_commitment = ctx_a.get_packet_commitment(
        &packet.source_port,
        &packet.source_channel,
        &packet.sequence,
    )?;

    if packet_commitment
        != ctx_a.packet_commitment(
            &packet.data,
            &packet.timeout_height,
            &packet.timeout_timestamp,
        )
    {
        return Err(PacketError::IncorrectPacketCommitment {
            sequence: packet.sequence,
        });
    }

    // Verify proofs
    {
        let client_id_on_a = conn_end_on_a.client_id();
        let client_state_on_a = ctx_a
            .client_state(client_id_on_a)
            .map_err(PacketError::Channel)?;

        // The client must not be frozen.
        if client_state_on_a.is_frozen() {
            return Err(PacketError::FrozenClient {
                client_id: client_id_on_a.clone(),
            });
        }

        let consensus_state = ctx_a
            .client_consensus_state(client_id_on_a, &msg.proof_height_on_b)
            .map_err(PacketError::Channel)?;

        let ack_commitment = ctx_a.ack_commitment(&msg.acknowledgement);

        // Verify the proof for the packet against the chain store.
        client_state_on_a
            .verify_packet_acknowledgement(
                ctx_a,
                msg.proof_height_on_b,
                &conn_end_on_a,
                &msg.proof_acked_on_b,
                consensus_state.root(),
                &packet.destination_port,
                &packet.destination_channel,
                packet.sequence,
                ack_commitment,
            )
            .map_err(|e| ChannelError::PacketVerificationFailed {
                sequence: packet.sequence,
                client_error: e,
            })
            .map_err(PacketError::Channel)?;
    }

    let result = if chan_end_on_a.order_matches(&Order::Ordered) {
        let next_seq_ack =
            ctx_a.get_next_sequence_ack(&packet.source_port, &packet.source_channel)?;

        if packet.sequence != next_seq_ack {
            return Err(PacketError::InvalidPacketSequence {
                given_sequence: packet.sequence,
                next_sequence: next_seq_ack,
            });
        }

        PacketResult::Ack(AckPacketResult {
            port_id: packet.source_port.clone(),
            channel_id: packet.source_channel.clone(),
            seq: packet.sequence,
            seq_number: Some(next_seq_ack.increment()),
        })
    } else {
        PacketResult::Ack(AckPacketResult {
            port_id: packet.source_port.clone(),
            channel_id: packet.source_channel.clone(),
            seq: packet.sequence,
            seq_number: None,
        })
    };

    output.log("success: packet ack");

    output.emit(IbcEvent::AcknowledgePacket(AcknowledgePacket::new(
        packet.clone(),
        chan_end_on_a.ordering,
        conn_id_on_a.clone(),
    )));

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
    use crate::core::ics04_channel::handler::acknowledgement::process;
    use crate::core::ics04_channel::msgs::acknowledgement::test_util::get_dummy_raw_msg_acknowledgement;
    use crate::core::ics04_channel::msgs::acknowledgement::MsgAcknowledgement;
    use crate::core::ics04_channel::Version;
    use crate::core::ics24_host::identifier::{ClientId, ConnectionId};
    use crate::events::IbcEvent;
    use crate::mock::context::MockContext;
    use crate::prelude::*;
    use crate::timestamp::ZERO_DURATION;

    #[test]
    fn ack_packet_processing() {
        struct Test {
            name: String,
            ctx: MockContext,
            msg: MsgAcknowledgement,
            want_pass: bool,
        }

        let context = MockContext::default();

        let client_height = Height::new(0, 2).unwrap();

        let msg = MsgAcknowledgement::try_from(get_dummy_raw_msg_acknowledgement(
            client_height.revision_height(),
        ))
        .unwrap();
        let packet = msg.packet.clone();

        let data = context.packet_commitment(
            &packet.data,
            &packet.timeout_height,
            &packet.timeout_timestamp,
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
                name: "Good parameters".to_string(),
                ctx: context
                    .with_client(&ClientId::default(), client_height)
                    .with_connection(ConnectionId::default(), conn_end_on_a)
                    .with_channel(
                        packet.source_port.clone(),
                        packet.source_channel.clone(),
                        chan_end_on_a,
                    )
                    .with_packet_commitment(
                        packet.source_port,
                        packet.source_channel,
                        packet.sequence,
                        data,
                    ) //with_ack_sequence required for ordered channels
                    .with_ack_sequence(
                        packet.destination_port,
                        packet.destination_channel,
                        1.into(),
                    ),
                msg,
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
                        "ack_packet: test passed but was supposed to fail for test: {}, \nparams {:?} {:?}",
                        test.name,
                        test.msg.clone(),
                        test.ctx.clone()
                    );

                    assert!(!proto_output.events.is_empty()); // Some events must exist.

                    for e in proto_output.events.iter() {
                        assert!(matches!(e, &IbcEvent::AcknowledgePacket(_)));
                    }
                }
                Err(e) => {
                    assert!(
                        !test.want_pass,
                        "ack_packet: did not pass test: {}, \nparams {:?} {:?} error: {:?}",
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
