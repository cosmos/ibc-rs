use ibc_eureka_core_channel_types::commitment::{
    compute_ack_commitment, compute_packet_commitment,
};
use ibc_eureka_core_channel_types::error::ChannelError;
use ibc_eureka_core_channel_types::events::AcknowledgePacket;
use ibc_eureka_core_channel_types::msgs::MsgAcknowledgement;
use ibc_eureka_core_client::context::prelude::*;
use ibc_eureka_core_handler_types::events::{IbcEvent, MessageEvent};
use ibc_eureka_core_host::types::identifiers::ClientId;
use ibc_eureka_core_host::types::path::{
    AckPathV2 as AckPath, ClientConsensusStatePath, CommitmentPathV2 as CommitmentPath, Path,
};
use ibc_eureka_core_host::{ExecutionContext, ValidationContext};
use ibc_eureka_core_router::module::Module;
use ibc_primitives::prelude::*;

pub fn acknowledgement_packet_validate<ValCtx>(
    ctx_a: &ValCtx,
    module: &dyn Module,
    msg: MsgAcknowledgement,
) -> Result<(), ChannelError>
where
    ValCtx: ValidationContext,
{
    validate(ctx_a, &msg)?;

    module.on_acknowledgement_packet_validate(&msg.packet, &msg.acknowledgement, &msg.signer)
}

pub fn acknowledgement_packet_execute<ExecCtx>(
    ctx_a: &mut ExecCtx,
    module: &mut dyn Module,
    msg: MsgAcknowledgement,
) -> Result<(), ChannelError>
where
    ExecCtx: ExecutionContext,
{
    let packet = &msg.packet;

    let channel_target_client_on_source = &packet.header.target_client_on_source;
    let channel_source_client_on_target = &packet.header.source_client_on_target;
    let seq_on_a = &packet.header.seq_on_a;

    // In all cases, this event is emitted
    let event = IbcEvent::AcknowledgePacket(AcknowledgePacket::new(packet.clone()));
    ctx_a.emit_ibc_event(IbcEvent::Message(MessageEvent::Channel))?;
    ctx_a.emit_ibc_event(event)?;

    let commitment_path_on_a = CommitmentPath::new(
        channel_source_client_on_target.as_ref(),
        channel_target_client_on_source.as_ref(),
        seq_on_a,
    );

    // check if we're in the NO-OP case
    if ctx_a.get_packet_commitment(&commitment_path_on_a).is_err() {
        // This error indicates that the timeout has already been relayed
        // or there is a misconfigured relayer attempting to prove a timeout
        // for a packet never sent. Core IBC will treat this error as a no-op in order to
        // prevent an entire relay transaction from failing and consuming unnecessary fees.
        return Ok(());
    };

    let (extras, cb_result) =
        module.on_acknowledgement_packet_execute(packet, &msg.acknowledgement, &msg.signer);

    cb_result?;

    // apply state changes
    {
        ctx_a.delete_packet_commitment(&commitment_path_on_a)?;
    }

    // emit events and logs
    {
        ctx_a.log_message("success: packet acknowledgement".to_string())?;

        // Note: Acknowledgement event was emitted at the beginning

        for module_event in extras.events {
            ctx_a.emit_ibc_event(IbcEvent::Module(module_event))?
        }

        for log_message in extras.log {
            ctx_a.log_message(log_message)?;
        }
    }

    Ok(())
}

fn validate<Ctx>(ctx_a: &Ctx, msg: &MsgAcknowledgement) -> Result<(), ChannelError>
where
    Ctx: ValidationContext,
{
    ctx_a.validate_message_signer(&msg.signer)?;

    let packet = &msg.packet;
    let payload = &packet.payloads[0];

    let channel_target_client_on_source = &packet.header.target_client_on_source;
    let channel_source_client_on_target = &packet.header.source_client_on_target;
    let seq_on_a = &packet.header.seq_on_a;
    let data = &payload.data;

    let commitment_path_on_a = CommitmentPath::new(
        channel_source_client_on_target.as_ref(),
        channel_target_client_on_source.as_ref(),
        seq_on_a,
    );

    // Verify packet commitment
    let Ok(commitment_on_a) = ctx_a.get_packet_commitment(&commitment_path_on_a) else {
        // This error indicates that the timeout has already been relayed
        // or there is a misconfigured relayer attempting to prove a timeout
        // for a packet never sent. Core IBC will treat this error as a no-op in order to
        // prevent an entire relay transaction from failing and consuming unnecessary fees.
        return Ok(());
    };

    let expected_commitment_on_a = compute_packet_commitment(
        data,
        &packet.header.timeout_height_on_b,
        &packet.header.timeout_timestamp_on_b,
    );

    if commitment_on_a != expected_commitment_on_a {
        return Err(ChannelError::MismatchedPacketCommitment {
            actual: commitment_on_a,
            expected: expected_commitment_on_a,
        });
    }

    // Verify proofs
    {
        // TODO(rano): avoid a vs b confusion
        let id_target_client_on_source = channel_target_client_on_source.as_ref();
        let id_source_client_on_target: &ClientId = channel_source_client_on_target.as_ref();

        let client_val_ctx_a = ctx_a.get_client_validation_context();

        let (stored_id_source_client_on_target, target_prefix) =
            client_val_ctx_a.counterparty_client(id_target_client_on_source)?;

        if &stored_id_source_client_on_target != id_source_client_on_target {
            return Err(ChannelError::MismatchCounterparty {
                expected: stored_id_source_client_on_target.clone(),
                actual: id_source_client_on_target.clone(),
            });
        }

        let target_client_on_source = client_val_ctx_a.client_state(id_target_client_on_source)?;

        target_client_on_source
            .status(
                ctx_a.get_client_validation_context(),
                id_target_client_on_source,
            )?
            .verify_is_active()?;

        target_client_on_source.validate_proof_height(msg.proof_height_on_b)?;

        let client_cons_state_path_on_a = ClientConsensusStatePath::new(
            id_target_client_on_source.clone(),
            msg.proof_height_on_b.revision_number(),
            msg.proof_height_on_b.revision_height(),
        );
        let consensus_state_of_b_on_a =
            client_val_ctx_a.consensus_state(&client_cons_state_path_on_a)?;
        let ack_commitment = compute_ack_commitment(&msg.acknowledgement);
        let ack_path_on_b = AckPath::new(
            channel_source_client_on_target.as_ref(),
            channel_target_client_on_source.as_ref(),
            seq_on_a,
        );

        // Verify the proof for the packet against the chain store.
        target_client_on_source.verify_membership(
            &target_prefix,
            &msg.proof_acked_on_b,
            consensus_state_of_b_on_a.root(),
            Path::AckV2(ack_path_on_b),
            ack_commitment.into_vec(),
        )?;
    }

    Ok(())
}
