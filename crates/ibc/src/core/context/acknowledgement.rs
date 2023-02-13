use crate::prelude::*;

use crate::{
    core::{
        ics04_channel::{
            channel::Order, error::ChannelError, events::AcknowledgePacket,
            handler::acknowledgement, msgs::acknowledgement::MsgAcknowledgement,
        },
        ics24_host::path::CommitmentsPath,
        ics26_routing::context::ModuleId,
    },
    events::IbcEvent,
};

use super::{ContextError, ExecutionContext, ValidationContext};

pub(super) fn acknowledgement_packet_validate<ValCtx>(
    ctx_a: &ValCtx,
    module_id: ModuleId,
    msg: MsgAcknowledgement,
) -> Result<(), ContextError>
where
    ValCtx: ValidationContext,
{
    acknowledgement::validate(ctx_a, &msg)?;

    let module = ctx_a
        .get_route(&module_id)
        .ok_or(ChannelError::RouteNotFound)?;

    module
        .on_acknowledgement_packet_validate(&msg.packet, &msg.acknowledgement, &msg.signer)
        .map_err(ContextError::PacketError)
}

pub(super) fn acknowledgement_packet_execute<ExecCtx>(
    ctx_a: &mut ExecCtx,
    module_id: ModuleId,
    msg: MsgAcknowledgement,
) -> Result<(), ContextError>
where
    ExecCtx: ExecutionContext,
{
    let port_chan_id_on_a = (msg.packet.port_on_a.clone(), msg.packet.chan_on_a.clone());
    let chan_end_on_a = ctx_a.channel_end(&port_chan_id_on_a)?;
    let conn_id_on_a = &chan_end_on_a.connection_hops()[0];

    // In all cases, this event is emitted
    ctx_a.emit_ibc_event(IbcEvent::AcknowledgePacket(AcknowledgePacket::new(
        msg.packet.clone(),
        chan_end_on_a.ordering,
        conn_id_on_a.clone(),
    )));

    // check if we're in the NO-OP case
    if ctx_a
        .get_packet_commitment(&(
            msg.packet.port_on_a.clone(),
            msg.packet.chan_on_a.clone(),
            msg.packet.sequence,
        ))
        .is_err()
    {
        // This error indicates that the timeout has already been relayed
        // or there is a misconfigured relayer attempting to prove a timeout
        // for a packet never sent. Core IBC will treat this error as a no-op in order to
        // prevent an entire relay transaction from failing and consuming unnecessary fees.
        return Ok(());
    };

    let module = ctx_a
        .get_route_mut(&module_id)
        .ok_or(ChannelError::RouteNotFound)?;

    let (extras, cb_result) =
        module.on_acknowledgement_packet_execute(&msg.packet, &msg.acknowledgement, &msg.signer);

    cb_result?;

    // apply state changes
    {
        let commitment_path = CommitmentsPath {
            port_id: msg.packet.port_on_a.clone(),
            channel_id: msg.packet.chan_on_a.clone(),
            sequence: msg.packet.sequence,
        };
        ctx_a.delete_packet_commitment(commitment_path)?;

        if let Order::Ordered = chan_end_on_a.ordering {
            // Note: in validation, we verified that `msg.packet.sequence == nextSeqRecv`
            // (where `nextSeqRecv` is the value in the store)
            ctx_a.store_next_sequence_ack(port_chan_id_on_a, msg.packet.sequence.increment())?;
        }
    }

    // emit events and logs
    {
        ctx_a.log_message("success: packet acknowledgement".to_string());

        // Note: Acknowledgement event was emitted at the beginning

        for module_event in extras.events {
            ctx_a.emit_ibc_event(IbcEvent::AppModule(module_event));
        }

        for log_message in extras.log {
            ctx_a.log_message(log_message);
        }
    }

    Ok(())
}
