use crate::core::ics04_channel::channel::State;
use crate::core::ics04_channel::commitment::AcknowledgementCommitment;
use crate::core::ics04_channel::events::WriteAcknowledgement;
use crate::core::ics04_channel::msgs::acknowledgement::Acknowledgement;
use crate::core::ics04_channel::packet::{Packet, PacketResult, Sequence};
use crate::core::ics04_channel::{context::ChannelReader, error::PacketError};
use crate::core::ics24_host::identifier::{ChannelId, PortId};
use crate::prelude::*;
use crate::{
    events::IbcEvent,
    handler::{HandlerOutput, HandlerResult},
};

#[derive(Clone, Debug)]
pub struct WriteAckPacketResult {
    pub port_id: PortId,
    pub channel_id: ChannelId,
    pub seq: Sequence,
    pub ack_commitment: AcknowledgementCommitment,
}

/// Per our convention, this message is processed on chain B.
pub fn process<Ctx: ChannelReader>(
    ctx_b: &Ctx,
    packet: Packet,
    ack: Acknowledgement,
) -> HandlerResult<PacketResult, PacketError> {
    let mut output = HandlerOutput::builder();

    let chan_end_on_b = ctx_b
        .channel_end(&packet.port_on_b, &packet.chan_on_b)
        .map_err(PacketError::Channel)?;

    if !chan_end_on_b.state_matches(&State::Open) {
        return Err(PacketError::InvalidChannelState {
            channel_id: packet.chan_on_a,
            state: chan_end_on_b.state,
        });
    }

    // NOTE: IBC app modules might have written the acknowledgement synchronously on
    // the OnRecvPacket callback so we need to check if the acknowledgement is already
    // set on the store and return an error if so.
    match ctx_b.get_packet_acknowledgement(&packet.port_on_b, &packet.chan_on_b, &packet.sequence) {
        Ok(_) => {
            return Err(PacketError::AcknowledgementExists {
                sequence: packet.sequence,
            })
        }
        Err(e)
            if e.to_string()
                == PacketError::PacketAcknowledgementNotFound {
                    sequence: packet.sequence,
                }
                .to_string() => {}
        Err(e) => return Err(e),
    }

    let result = PacketResult::WriteAck(WriteAckPacketResult {
        port_id: packet.port_on_b.clone(),
        channel_id: packet.chan_on_b.clone(),
        seq: packet.sequence,
        ack_commitment: ctx_b.ack_commitment(&ack),
    });

    output.log("success: packet write acknowledgement");

    {
        let conn_id_on_b = chan_end_on_b.connection_hops()[0].clone();

        output.emit(IbcEvent::WriteAcknowledgement(WriteAcknowledgement::new(
            packet,
            ack,
            conn_id_on_b,
        )));
    }

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
    use crate::core::ics04_channel::handler::write_acknowledgement::process;
    use crate::core::ics04_channel::packet::test_utils::get_dummy_raw_packet;
    use crate::core::ics04_channel::Version;
    use crate::core::ics24_host::identifier::{ChannelId, ClientId, ConnectionId, PortId};
    use crate::mock::context::MockContext;
    use crate::timestamp::ZERO_DURATION;
    use crate::{core::ics04_channel::packet::Packet, events::IbcEvent};

    #[test]
    fn write_ack_packet_processing() {
        struct Test {
            name: String,
            ctx: MockContext,
            packet: Packet,
            ack: Vec<u8>,
            want_pass: bool,
        }

        let context = MockContext::default();

        let client_height = Height::new(0, 1).unwrap();

        let mut packet: Packet = get_dummy_raw_packet(1, 6).try_into().unwrap();
        packet.sequence = 1.into();
        packet.data = vec![0];

        let ack = vec![0];
        let ack_null = Vec::new();

        let dest_channel_end = ChannelEnd::new(
            State::Open,
            Order::default(),
            Counterparty::new(packet.port_on_a.clone(), Some(packet.chan_on_a.clone())),
            vec![ConnectionId::default()],
            Version::new("ics20-1".to_string()),
        );

        let connection_end = ConnectionEnd::new(
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
                packet: packet.clone(),
                ack: ack.clone(),
                want_pass: false,
            },
            Test {
                name: "Good parameters".to_string(),
                ctx: context
                    .clone()
                    .with_client(&ClientId::default(), client_height)
                    .with_connection(ConnectionId::default(), connection_end.clone())
                    .with_channel(
                        packet.port_on_b.clone(),
                        packet.chan_on_b.clone(),
                        dest_channel_end.clone(),
                    ),
                packet: packet.clone(),
                ack,
                want_pass: true,
            },
            Test {
                name: "Zero ack".to_string(),
                ctx: context
                    .with_client(&ClientId::default(), Height::new(0, 1).unwrap())
                    .with_connection(ConnectionId::default(), connection_end)
                    .with_channel(PortId::default(), ChannelId::default(), dest_channel_end),
                packet,
                ack: ack_null,
                want_pass: false,
            },
        ]
        .into_iter()
        .collect();

        for test in tests {
            let res = process(&test.ctx, test.packet.clone(), test.ack.try_into().unwrap());
            // Additionally check the events and the output objects in the result.
            match res {
                Ok(proto_output) => {
                    assert!(
                        test.want_pass,
                        "write_ack: test passed but was supposed to fail for test: {}, \nparams {:?} {:?}",
                        test.name,
                        test.packet.clone(),
                        test.ctx.clone()
                    );

                    assert!(!proto_output.events.is_empty()); // Some events must exist.

                    for e in proto_output.events.iter() {
                        assert!(matches!(e, &IbcEvent::WriteAcknowledgement(_)));
                    }
                }
                Err(e) => {
                    assert!(
                        !test.want_pass,
                        "write_ack: did not pass test: {}, \nparams {:?} {:?} error: {:?}",
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
