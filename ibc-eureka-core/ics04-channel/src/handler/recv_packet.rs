use ibc_eureka_core_channel_types::channel::{Counterparty, Order, State as ChannelState};
use ibc_eureka_core_channel_types::commitment::{
    compute_ack_commitment, compute_packet_commitment,
};
use ibc_eureka_core_channel_types::error::ChannelError;
use ibc_eureka_core_channel_types::events::{ReceivePacket, WriteAcknowledgement};
use ibc_eureka_core_channel_types::msgs::MsgRecvPacket;
use ibc_eureka_core_channel_types::packet::Receipt;
use ibc_eureka_core_client::context::prelude::*;
use ibc_eureka_core_handler_types::events::{IbcEvent, MessageEvent};
use ibc_eureka_core_host::types::path::{
    AckPath, ChannelEndPath, ClientConsensusStatePath, CommitmentPath, Path, ReceiptPath,
    SeqRecvPath,
};
use ibc_eureka_core_host::{ExecutionContext, ValidationContext};
use ibc_eureka_core_router::module::Module;
use ibc_primitives::prelude::*;

pub fn recv_packet_validate<ValCtx>(ctx_b: &ValCtx, msg: MsgRecvPacket) -> Result<(), ChannelError>
where
    ValCtx: ValidationContext,
{
    // Note: this contains the validation for `write_acknowledgement` as well.
    validate(ctx_b, &msg)

    // nothing to validate with the module, since `onRecvPacket` cannot fail.
    // If any error occurs, then an "error acknowledgement" must be returned.
}

pub fn recv_packet_execute<ExecCtx>(
    ctx_b: &mut ExecCtx,
    module: &mut dyn Module,
    msg: MsgRecvPacket,
) -> Result<(), ChannelError>
where
    ExecCtx: ExecutionContext,
{
    let packet = &msg.packet;
    let payload = &packet.payloads[0];

    let port_id_on_b = &payload.header.target_port.1;
    let channel_id_on_b = &packet.header.target_client;
    let seq_on_a = &packet.header.seq_on_a;

    let chan_end_path_on_b = ChannelEndPath::new(port_id_on_b, channel_id_on_b);
    let chan_end_on_b = ctx_b.channel_end(&chan_end_path_on_b)?;

    // Check if another relayer already relayed the packet.
    // We don't want to fail the transaction in this case.
    {
        let packet_already_received = match chan_end_on_b.ordering {
            // Note: ibc-go doesn't make the check for `Order::None` channels
            Order::None => false,
            Order::Unordered => {
                let receipt_path_on_b = ReceiptPath::new(port_id_on_b, channel_id_on_b, *seq_on_a);
                ctx_b.get_packet_receipt(&receipt_path_on_b)?.is_ok()
            }
            Order::Ordered => {
                let seq_recv_path_on_b = SeqRecvPath::new(port_id_on_b, channel_id_on_b);
                let next_seq_recv = ctx_b.get_next_sequence_recv(&seq_recv_path_on_b)?;

                // the sequence number has already been incremented, so
                // another relayer already relayed the packet
                seq_on_a < &next_seq_recv
            }
        };

        if packet_already_received {
            return Ok(());
        }
    }

    let (extras, acknowledgement) = module.on_recv_packet_execute(&msg.packet, &msg.signer);

    // state changes
    {
        // `recvPacket` core handler state changes
        match chan_end_on_b.ordering {
            Order::Unordered => {
                let receipt_path_on_b = ReceiptPath {
                    port_id: port_id_on_b.clone(),
                    channel_id: channel_id_on_b.clone(),
                    sequence: *seq_on_a,
                };

                ctx_b.store_packet_receipt(&receipt_path_on_b, Receipt::Ok)?;
            }
            Order::Ordered => {
                let seq_recv_path_on_b = SeqRecvPath::new(port_id_on_b, channel_id_on_b);
                let next_seq_recv = ctx_b.get_next_sequence_recv(&seq_recv_path_on_b)?;
                ctx_b.store_next_sequence_recv(&seq_recv_path_on_b, next_seq_recv.increment())?;
            }
            _ => {}
        }
        let ack_path_on_b = AckPath::new(port_id_on_b, channel_id_on_b, *seq_on_a);
        // `writeAcknowledgement` handler state changes
        ctx_b.store_packet_acknowledgement(
            &ack_path_on_b,
            compute_ack_commitment(&acknowledgement),
        )?;
    }

    // emit events and logs
    {
        ctx_b.log_message("success: packet receive".to_string())?;
        ctx_b.log_message("success: packet write acknowledgement".to_string())?;

        let event = IbcEvent::ReceivePacket(ReceivePacket::new(
            msg.packet.clone(),
            chan_end_on_b.ordering,
        ));
        ctx_b.emit_ibc_event(IbcEvent::Message(MessageEvent::Channel))?;
        ctx_b.emit_ibc_event(event)?;
        let event =
            IbcEvent::WriteAcknowledgement(WriteAcknowledgement::new(msg.packet, acknowledgement));
        ctx_b.emit_ibc_event(IbcEvent::Message(MessageEvent::Channel))?;
        ctx_b.emit_ibc_event(event)?;

        for module_event in extras.events {
            ctx_b.emit_ibc_event(IbcEvent::Module(module_event))?;
        }

        for log_message in extras.log {
            ctx_b.log_message(log_message)?;
        }
    }

    Ok(())
}

fn validate<Ctx>(ctx_b: &Ctx, msg: &MsgRecvPacket) -> Result<(), ChannelError>
where
    Ctx: ValidationContext,
{
    ctx_b.validate_message_signer(&msg.signer)?;

    let packet = &msg.packet;
    let payload = &packet.payloads[0];

    let (_, port_id_on_a) = &payload.header.source_port;
    let channel_id_on_a = &packet.header.source_client;
    let (prefix_on_b, port_id_on_b) = &payload.header.target_port;
    let channel_id_on_b = &packet.header.target_client;
    let seq_on_a = &packet.header.seq_on_a;
    let data = &payload.data;

    let chan_end_path_on_b = ChannelEndPath::new(port_id_on_b, channel_id_on_b);
    let chan_end_on_b = ctx_b.channel_end(&chan_end_path_on_b)?;

    chan_end_on_b.verify_state_matches(&ChannelState::Open)?;

    let counterparty = Counterparty::new(port_id_on_a.clone(), Some(channel_id_on_a.clone()));

    chan_end_on_b.verify_counterparty_matches(&counterparty)?;

    let latest_height = ctx_b.host_height()?;
    if packet.header.timeout_height_on_b.has_expired(latest_height) {
        return Err(ChannelError::InsufficientPacketHeight {
            chain_height: latest_height,
            timeout_height: packet.header.timeout_height_on_b,
        });
    }

    let latest_timestamp = ctx_b.host_timestamp()?;
    if packet
        .header
        .timeout_timestamp_on_b
        .has_expired(&latest_timestamp)
    {
        return Err(ChannelError::ExpiredPacketTimestamp);
    }

    // Verify proofs
    {
        let client_id_on_b = channel_id_on_a.as_ref();
        let client_val_ctx_b = ctx_b.get_client_validation_context();
        let client_state_of_a_on_b = client_val_ctx_b.client_state(client_id_on_b)?;

        client_state_of_a_on_b
            .status(ctx_b.get_client_validation_context(), client_id_on_b)?
            .verify_is_active()?;

        client_state_of_a_on_b.validate_proof_height(msg.proof_height_on_a)?;

        let client_cons_state_path_on_b = ClientConsensusStatePath::new(
            client_id_on_b.clone(),
            msg.proof_height_on_a.revision_number(),
            msg.proof_height_on_a.revision_height(),
        );

        let consensus_state_of_a_on_b =
            client_val_ctx_b.consensus_state(&client_cons_state_path_on_b)?;

        let expected_commitment_on_a = compute_packet_commitment(
            data,
            &packet.header.timeout_height_on_b,
            &packet.header.timeout_timestamp_on_b,
        );
        let commitment_path_on_a = CommitmentPath::new(port_id_on_a, channel_id_on_a, *seq_on_a);

        // Verify the proof for the packet against the chain store.
        client_state_of_a_on_b.verify_membership(
            prefix_on_b,
            &msg.proof_commitment_on_a,
            consensus_state_of_a_on_b.root(),
            Path::Commitment(commitment_path_on_a),
            expected_commitment_on_a.into_vec(),
        )?;
    }

    match chan_end_on_b.ordering {
        Order::Ordered => {
            let seq_recv_path_on_b = SeqRecvPath::new(port_id_on_b, channel_id_on_b);
            let next_seq_recv = ctx_b.get_next_sequence_recv(&seq_recv_path_on_b)?;
            if seq_on_a > &next_seq_recv {
                return Err(ChannelError::MismatchedPacketSequence {
                    actual: *seq_on_a,
                    expected: next_seq_recv,
                });
            }

            if seq_on_a == &next_seq_recv {
                // Case where the recvPacket is successful and an
                // acknowledgement will be written (not a no-op)
                validate_write_acknowledgement(ctx_b, msg)?;
            }
        }
        Order::Unordered => {
            // Note: We don't check for the packet receipt here because another
            // relayer may have already relayed the packet. If that's the case,
            // we want to avoid failing the transaction and consuming
            // unnecessary fees.

            // Case where the recvPacket is successful and an
            // acknowledgement will be written (not a no-op)
            validate_write_acknowledgement(ctx_b, msg)?;
        }
        Order::None => {
            return Err(ChannelError::InvalidState {
                expected: "Channel ordering to not be None".to_string(),
                actual: chan_end_on_b.ordering.to_string(),
            })
        }
    }

    Ok(())
}

fn validate_write_acknowledgement<Ctx>(ctx_b: &Ctx, msg: &MsgRecvPacket) -> Result<(), ChannelError>
where
    Ctx: ValidationContext,
{
    let packet = &msg.packet;
    let payload = &packet.payloads[0];

    let port_id_on_b = &payload.header.target_port.1;
    let channel_id_on_b = &packet.header.target_client;
    let seq_on_a = &packet.header.seq_on_a;

    let ack_path_on_b = AckPath::new(port_id_on_b, channel_id_on_b, *seq_on_a);
    if ctx_b.get_packet_acknowledgement(&ack_path_on_b).is_ok() {
        return Err(ChannelError::DuplicateAcknowledgment(*seq_on_a));
    }

    Ok(())
}
