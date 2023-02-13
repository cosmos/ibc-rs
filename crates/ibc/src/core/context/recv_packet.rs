use crate::{
    core::{
        ics04_channel::{
            channel::Order,
            error::ChannelError,
            events::{ReceivePacket, WriteAcknowledgement},
            handler::recv_packet,
            msgs::recv_packet::MsgRecvPacket,
            packet::Receipt,
        },
        ics24_host::path::ReceiptsPath,
        ics26_routing::context::ModuleId,
    },
    events::IbcEvent,
    prelude::*,
};

use super::{ContextError, ExecutionContext, ValidationContext};

pub(super) fn recv_packet_validate<ValCtx>(
    ctx_b: &ValCtx,
    msg: MsgRecvPacket,
) -> Result<(), ContextError>
where
    ValCtx: ValidationContext,
{
    // Note: this contains the validation for `write_acknowledgement` as well.
    recv_packet::validate(ctx_b, &msg)

    // nothing to validate with the module, since `onRecvPacket` cannot fail.
}

pub(super) fn recv_packet_execute<ExecCtx>(
    ctx_b: &mut ExecCtx,
    module_id: ModuleId,
    msg: MsgRecvPacket,
) -> Result<(), ContextError>
where
    ExecCtx: ExecutionContext,
{
    let chan_port_id_on_b = (msg.packet.port_on_b.clone(), msg.packet.chan_on_b.clone());
    let chan_end_on_b = ctx_b.channel_end(&chan_port_id_on_b)?;

    // Check if another relayer already relayed the packet.
    // We don't want to fail the transaction in this case.
    {
        let packet_already_received = match chan_end_on_b.ordering {
            // Note: ibc-go doesn't make the check for `Order::None` channels
            Order::None => false,
            Order::Unordered => {
                let packet = msg.packet.clone();

                ctx_b
                    .get_packet_receipt(&(packet.port_on_b, packet.chan_on_b, packet.sequence))
                    .is_ok()
            }
            Order::Ordered => {
                let next_seq_recv = ctx_b.get_next_sequence_recv(&chan_port_id_on_b)?;

                // the sequence number has already been incremented, so
                // another relayer already relayed the packet
                msg.packet.sequence < next_seq_recv
            }
        };

        if packet_already_received {
            return Ok(());
        }
    }

    let module = ctx_b
        .get_route_mut(&module_id)
        .ok_or(ChannelError::RouteNotFound)?;

    let (extras, acknowledgement) = module.on_recv_packet_execute(&msg.packet, &msg.signer);

    // state changes
    {
        // `recvPacket` core handler state changes
        match chan_end_on_b.ordering {
            Order::Unordered => {
                let path = ReceiptsPath {
                    port_id: msg.packet.port_on_b.clone(),
                    channel_id: msg.packet.chan_on_b.clone(),
                    sequence: msg.packet.sequence,
                };

                ctx_b.store_packet_receipt(path, Receipt::Ok)?;
            }
            Order::Ordered => {
                let port_chan_id_on_b =
                    (msg.packet.port_on_b.clone(), msg.packet.chan_on_b.clone());
                let next_seq_recv = ctx_b.get_next_sequence_recv(&port_chan_id_on_b)?;

                ctx_b.store_next_sequence_recv(port_chan_id_on_b, next_seq_recv.increment())?;
            }
            _ => {}
        }

        // `writeAcknowledgement` handler state changes
        ctx_b.store_packet_acknowledgement(
            (
                msg.packet.port_on_b.clone(),
                msg.packet.chan_on_b.clone(),
                msg.packet.sequence,
            ),
            ctx_b.ack_commitment(&acknowledgement),
        )?;
    }

    // emit events and logs
    {
        ctx_b.log_message("success: packet receive".to_string());
        ctx_b.log_message("success: packet write acknowledgement".to_string());

        let conn_id_on_b = &chan_end_on_b.connection_hops()[0];
        ctx_b.emit_ibc_event(IbcEvent::ReceivePacket(ReceivePacket::new(
            msg.packet.clone(),
            chan_end_on_b.ordering,
            conn_id_on_b.clone(),
        )));
        ctx_b.emit_ibc_event(IbcEvent::WriteAcknowledgement(WriteAcknowledgement::new(
            msg.packet,
            acknowledgement,
            conn_id_on_b.clone(),
        )));

        for module_event in extras.events {
            ctx_b.emit_ibc_event(IbcEvent::AppModule(module_event));
        }

        for log_message in extras.log {
            ctx_b.log_message(log_message);
        }
    }

    Ok(())
}
