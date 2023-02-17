use crate::core::ics04_channel::channel::State;
use crate::core::ics04_channel::commitment::AcknowledgementCommitment;
use crate::core::ics04_channel::events::WriteAcknowledgement;
use crate::core::ics04_channel::msgs::acknowledgement::Acknowledgement;
use crate::core::ics04_channel::packet::{Packet, PacketResult, Sequence};
use crate::core::ics04_channel::{context::ChannelReader, error::PacketError};
use crate::core::ics24_host::identifier::{ChannelId, PortId};
use crate::core::ics24_host::path::{AckPath, ChannelEndPath};
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
    let chan_end_path_on_b = ChannelEndPath::new(&packet.port_on_b, &packet.chan_on_b);
    let chan_end_on_b = ctx_b
        .channel_end(&chan_end_path_on_b)
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
    let ack_path_on_b = AckPath::new(&packet.port_on_b, &packet.chan_on_b, packet.sequence);
    match ctx_b.get_packet_acknowledgement(&ack_path_on_b) {
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

