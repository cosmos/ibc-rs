use ibc_eureka_core_channel_types::commitment::{
    compute_ack_commitment, compute_packet_commitment,
};
use ibc_eureka_core_channel_types::error::ChannelError;
use ibc_eureka_core_channel_types::events::{ReceivePacket, WriteAcknowledgement};
use ibc_eureka_core_channel_types::msgs::MsgRecvPacket;
use ibc_eureka_core_client::context::prelude::*;
use ibc_eureka_core_handler_types::events::{IbcEvent, MessageEvent};
use ibc_eureka_core_host::types::path::{
    AckPathV2 as AckPath, ClientConsensusStatePath, CommitmentPathV2 as CommitmentPath, Path,
    ReceiptPathV2 as ReceiptPath, SeqRecvPathV2 as SeqRecvPath,
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

    let (source_prefix, _source_port) = &payload.header.source_port;
    let channel_source_client_on_target = &packet.header.source_client_on_target;
    let (target_prefix, _target_port) = &payload.header.target_port;
    let channel_target_client_on_source = &packet.header.target_client_on_source;
    let seq_on_a = &packet.header.seq_on_a;

    // Check if another relayer already relayed the packet.
    // We don't want to fail the transaction in this case.
    {
        let packet_already_received = {
            let receipt_path_on_b = ReceiptPath::new(
                channel_source_client_on_target.as_ref(),
                &format!("{source_prefix:?}"),
                channel_target_client_on_source.as_ref(),
                &format!("{target_prefix:?}"),
                seq_on_a,
            );
            ctx_b.get_packet_receipt(&receipt_path_on_b)?.is_ok()
        };

        if packet_already_received {
            return Ok(());
        }
    }

    let (extras, acknowledgement) = module.on_recv_packet_execute(&msg.packet, &msg.signer);

    // state changes
    {
        // `recvPacket` core handler state changes
        {
            let seq_recv_path_on_b = SeqRecvPath::new(
                channel_source_client_on_target.as_ref(),
                &format!("{source_prefix:?}"),
                channel_target_client_on_source.as_ref(),
                &format!("{target_prefix:?}"),
            );
            let next_seq_recv = ctx_b.get_next_sequence_recv(&seq_recv_path_on_b)?;
            ctx_b.store_next_sequence_recv(&seq_recv_path_on_b, next_seq_recv.increment())?;
        }
        let ack_path_on_b = AckPath::new(
            channel_source_client_on_target.as_ref(),
            &format!("{source_prefix:?}"),
            channel_target_client_on_source.as_ref(),
            &format!("{target_prefix:?}"),
            seq_on_a,
        );
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

        let event = IbcEvent::ReceivePacket(ReceivePacket::new(msg.packet.clone()));
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

    let (source_prefix, _source_port) = &payload.header.source_port;
    let channel_target_client_on_source = &packet.header.target_client_on_source;
    let (target_prefix, _target_port) = &payload.header.target_port;
    let channel_source_client_on_target = &packet.header.source_client_on_target;
    let seq_on_a = &packet.header.seq_on_a;
    let data = &payload.data;

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
        let id_source_client_on_target = channel_source_client_on_target.as_ref();
        let client_val_ctx_b = ctx_b.get_client_validation_context();
        let source_client_on_target = client_val_ctx_b.client_state(id_source_client_on_target)?;

        source_client_on_target
            .status(
                ctx_b.get_client_validation_context(),
                id_source_client_on_target,
            )?
            .verify_is_active()?;

        source_client_on_target.validate_proof_height(msg.proof_height_on_a)?;

        let client_cons_state_path_on_b = ClientConsensusStatePath::new(
            id_source_client_on_target.clone(),
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
        let commitment_path_on_a = CommitmentPath::new(
            channel_source_client_on_target.as_ref(),
            &format!("{source_prefix:?}"),
            channel_target_client_on_source.as_ref(),
            &format!("{target_prefix:?}"),
            seq_on_a,
        );

        // Verify the proof for the packet against the chain store.
        source_client_on_target.verify_membership(
            source_prefix,
            &msg.proof_commitment_on_a,
            consensus_state_of_a_on_b.root(),
            Path::CommitmentV2(commitment_path_on_a),
            expected_commitment_on_a.into_vec(),
        )?;
    }

    {
        // Note: We don't check for the packet receipt here because another
        // relayer may have already relayed the packet. If that's the case,
        // we want to avoid failing the transaction and consuming
        // unnecessary fees.

        // Case where the recvPacket is successful and an
        // acknowledgement will be written (not a no-op)
        validate_write_acknowledgement(ctx_b, msg)?;
    }

    Ok(())
}

fn validate_write_acknowledgement<Ctx>(ctx_b: &Ctx, msg: &MsgRecvPacket) -> Result<(), ChannelError>
where
    Ctx: ValidationContext,
{
    let packet = &msg.packet;
    let payload = &packet.payloads[0];

    let channel_source_client_on_target = &packet.header.source_client_on_target;
    let (target_prefix, _target_port) = &payload.header.target_port;
    let channel_target_client_on_source = &packet.header.target_client_on_source;
    let (source_prefix, _source_port) = &payload.header.source_port;
    let seq_on_a = &packet.header.seq_on_a;

    let ack_path_on_b = AckPath::new(
        channel_source_client_on_target.as_ref(),
        &format!("{source_prefix:?}"),
        channel_target_client_on_source.as_ref(),
        &format!("{target_prefix:?}"),
        seq_on_a,
    );
    if ctx_b.get_packet_acknowledgement(&ack_path_on_b).is_ok() {
        return Err(ChannelError::DuplicateAcknowledgment(*seq_on_a));
    }

    Ok(())
}
