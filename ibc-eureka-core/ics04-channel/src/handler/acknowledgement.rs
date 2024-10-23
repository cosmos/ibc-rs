use ibc_eureka_core_channel_types::channel::{Counterparty, Order, State as ChannelState};
use ibc_eureka_core_channel_types::commitment::{
    compute_ack_commitment, compute_packet_commitment,
};
use ibc_eureka_core_channel_types::error::ChannelError;
use ibc_eureka_core_channel_types::events::AcknowledgePacket;
use ibc_eureka_core_channel_types::msgs::MsgAcknowledgement;
use ibc_eureka_core_client::context::prelude::*;
use ibc_eureka_core_handler_types::events::{IbcEvent, MessageEvent};
use ibc_eureka_core_host::types::path::{
    AckPath, ChannelEndPath, ClientConsensusStatePath, CommitmentPath, Path, SeqAckPath,
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
    let payload = &msg.packet.payloads[0];

    let port_id_on_a = &payload.header.source_port.1;
    let channel_id_on_a = &msg.packet.header.source_client;
    let seq_on_a = &msg.packet.header.seq_on_a;

    let chan_end_path_on_a = ChannelEndPath::new(port_id_on_a, channel_id_on_a);
    let chan_end_on_a = ctx_a.channel_end(&chan_end_path_on_a)?;

    // In all cases, this event is emitted
    let event = IbcEvent::AcknowledgePacket(AcknowledgePacket::new(
        msg.packet.clone(),
        chan_end_on_a.ordering,
    ));
    ctx_a.emit_ibc_event(IbcEvent::Message(MessageEvent::Channel))?;
    ctx_a.emit_ibc_event(event)?;

    let commitment_path_on_a =
        CommitmentPath::new(port_id_on_a, channel_id_on_a, msg.packet.header.seq_on_a);

    // check if we're in the NO-OP case
    if ctx_a.get_packet_commitment(&commitment_path_on_a).is_err() {
        // This error indicates that the timeout has already been relayed
        // or there is a misconfigured relayer attempting to prove a timeout
        // for a packet never sent. Core IBC will treat this error as a no-op in order to
        // prevent an entire relay transaction from failing and consuming unnecessary fees.
        return Ok(());
    };

    let (extras, cb_result) =
        module.on_acknowledgement_packet_execute(&msg.packet, &msg.acknowledgement, &msg.signer);

    cb_result?;

    // apply state changes
    {
        ctx_a.delete_packet_commitment(&commitment_path_on_a)?;

        if let Order::Ordered = chan_end_on_a.ordering {
            // Note: in validation, we verified that `msg.packet.sequence == nextSeqRecv`
            // (where `nextSeqRecv` is the value in the store)
            let seq_ack_path_on_a = SeqAckPath::new(port_id_on_a, channel_id_on_a);
            ctx_a.store_next_sequence_ack(&seq_ack_path_on_a, (*seq_on_a).increment())?;
        }
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

    let (prefix_on_a, port_id_on_a) = &payload.header.source_port;
    let channel_id_on_a = &packet.header.source_client;
    let (_, port_id_on_b) = &payload.header.target_port;
    let channel_id_on_b = &packet.header.target_client;
    let seq_on_a = &packet.header.seq_on_a;
    let data = &payload.data;

    let chan_end_path_on_a = ChannelEndPath::new(port_id_on_a, channel_id_on_a);
    let chan_end_on_a = ctx_a.channel_end(&chan_end_path_on_a)?;

    chan_end_on_a.verify_state_matches(&ChannelState::Open)?;

    let counterparty = Counterparty::new(port_id_on_b.clone(), Some(channel_id_on_b.clone()));

    chan_end_on_a.verify_counterparty_matches(&counterparty)?;

    let commitment_path_on_a = CommitmentPath::new(port_id_on_a, channel_id_on_a, *seq_on_a);

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

    if let Order::Ordered = chan_end_on_a.ordering {
        let seq_ack_path_on_a = SeqAckPath::new(port_id_on_a, channel_id_on_a);
        let next_seq_ack = ctx_a.get_next_sequence_ack(&seq_ack_path_on_a)?;
        if seq_on_a != &next_seq_ack {
            return Err(ChannelError::MismatchedPacketSequence {
                actual: *seq_on_a,
                expected: next_seq_ack,
            });
        }
    }

    // Verify proofs
    {
        // TODO(rano): avoid a vs b confusion
        let client_id_on_a = channel_id_on_b.as_ref();

        let client_val_ctx_a = ctx_a.get_client_validation_context();

        let client_state_of_b_on_a = client_val_ctx_a.client_state(client_id_on_a)?;

        client_state_of_b_on_a
            .status(ctx_a.get_client_validation_context(), client_id_on_a)?
            .verify_is_active()?;

        client_state_of_b_on_a.validate_proof_height(msg.proof_height_on_b)?;

        let client_cons_state_path_on_a = ClientConsensusStatePath::new(
            client_id_on_a.clone(),
            msg.proof_height_on_b.revision_number(),
            msg.proof_height_on_b.revision_height(),
        );
        let consensus_state_of_b_on_a =
            client_val_ctx_a.consensus_state(&client_cons_state_path_on_a)?;
        let ack_commitment = compute_ack_commitment(&msg.acknowledgement);
        let ack_path_on_b = AckPath::new(port_id_on_b, channel_id_on_b, *seq_on_a);

        // Verify the proof for the packet against the chain store.
        client_state_of_b_on_a.verify_membership(
            prefix_on_a,
            &msg.proof_acked_on_b,
            consensus_state_of_b_on_a.root(),
            Path::Ack(ack_path_on_b),
            ack_commitment.into_vec(),
        )?;
    }

    Ok(())
}
