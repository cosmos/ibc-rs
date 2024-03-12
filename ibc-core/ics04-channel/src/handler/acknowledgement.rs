use ibc_core_channel_types::channel::{Counterparty, Order, State as ChannelState};
use ibc_core_channel_types::commitment::{compute_ack_commitment, compute_packet_commitment};
use ibc_core_channel_types::error::{ChannelError, PacketError};
use ibc_core_channel_types::events::AcknowledgePacket;
use ibc_core_channel_types::msgs::MsgAcknowledgement;
use ibc_core_client::context::prelude::*;
use ibc_core_connection::delay::verify_conn_delay_passed;
use ibc_core_connection::types::State as ConnectionState;
use ibc_core_handler_types::error::ContextError;
use ibc_core_handler_types::events::{IbcEvent, MessageEvent};
use ibc_core_host::types::path::{
    AckPath, ChannelEndPath, ClientConsensusStatePath, CommitmentPath, Path, SeqAckPath,
};
use ibc_core_host::{ExecutionContext, ValidationContext};
use ibc_core_router::module::Module;
use ibc_primitives::prelude::*;

pub fn acknowledgement_packet_validate<ValCtx>(
    ctx_a: &ValCtx,
    module: &dyn Module,
    msg: MsgAcknowledgement,
) -> Result<(), ContextError>
where
    ValCtx: ValidationContext,
{
    validate(ctx_a, &msg)?;

    module
        .on_acknowledgement_packet_validate(&msg.packet, &msg.acknowledgement, &msg.signer)
        .map_err(ContextError::PacketError)
}

pub fn acknowledgement_packet_execute<ExecCtx>(
    ctx_a: &mut ExecCtx,
    module: &mut dyn Module,
    msg: MsgAcknowledgement,
) -> Result<(), ContextError>
where
    ExecCtx: ExecutionContext,
{
    let chan_end_path_on_a =
        ChannelEndPath::new(&msg.packet.port_id_on_a, &msg.packet.chan_id_on_a);
    let chan_end_on_a = ctx_a.channel_end(&chan_end_path_on_a)?;
    let conn_id_on_a = &chan_end_on_a.connection_hops()[0];

    // In all cases, this event is emitted
    let event = IbcEvent::AcknowledgePacket(AcknowledgePacket::new(
        msg.packet.clone(),
        chan_end_on_a.ordering,
        conn_id_on_a.clone(),
    ));
    ctx_a.emit_ibc_event(IbcEvent::Message(MessageEvent::Channel))?;
    ctx_a.emit_ibc_event(event)?;

    let commitment_path_on_a = CommitmentPath::new(
        &msg.packet.port_id_on_a,
        &msg.packet.chan_id_on_a,
        msg.packet.seq_on_a,
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
        module.on_acknowledgement_packet_execute(&msg.packet, &msg.acknowledgement, &msg.signer);

    cb_result?;

    // apply state changes
    {
        ctx_a.delete_packet_commitment(&commitment_path_on_a)?;

        if let Order::Ordered = chan_end_on_a.ordering {
            // Note: in validation, we verified that `msg.packet.sequence == nextSeqRecv`
            // (where `nextSeqRecv` is the value in the store)
            let seq_ack_path_on_a =
                SeqAckPath::new(&msg.packet.port_id_on_a, &msg.packet.chan_id_on_a);
            ctx_a.store_next_sequence_ack(&seq_ack_path_on_a, msg.packet.seq_on_a.increment())?;
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

fn validate<Ctx>(ctx_a: &Ctx, msg: &MsgAcknowledgement) -> Result<(), ContextError>
where
    Ctx: ValidationContext,
{
    ctx_a.validate_message_signer(&msg.signer)?;

    let packet = &msg.packet;
    let chan_end_path_on_a = ChannelEndPath::new(&packet.port_id_on_a, &packet.chan_id_on_a);
    let chan_end_on_a = ctx_a.channel_end(&chan_end_path_on_a)?;

    chan_end_on_a.verify_state_matches(&ChannelState::Open)?;

    let counterparty = Counterparty::new(
        packet.port_id_on_b.clone(),
        Some(packet.chan_id_on_b.clone()),
    );

    chan_end_on_a.verify_counterparty_matches(&counterparty)?;

    let conn_id_on_a = &chan_end_on_a.connection_hops()[0];
    let conn_end_on_a = ctx_a.connection_end(conn_id_on_a)?;

    conn_end_on_a.verify_state_matches(&ConnectionState::Open)?;

    let commitment_path_on_a =
        CommitmentPath::new(&packet.port_id_on_a, &packet.chan_id_on_a, packet.seq_on_a);

    // Verify packet commitment
    let Ok(commitment_on_a) = ctx_a.get_packet_commitment(&commitment_path_on_a) else {
        // This error indicates that the timeout has already been relayed
        // or there is a misconfigured relayer attempting to prove a timeout
        // for a packet never sent. Core IBC will treat this error as a no-op in order to
        // prevent an entire relay transaction from failing and consuming unnecessary fees.
        return Ok(());
    };

    if commitment_on_a
        != compute_packet_commitment(
            &packet.data,
            &packet.timeout_height_on_b,
            &packet.timeout_timestamp_on_b,
        )
    {
        return Err(PacketError::IncorrectPacketCommitment {
            sequence: packet.seq_on_a,
        }
        .into());
    }

    if let Order::Ordered = chan_end_on_a.ordering {
        let seq_ack_path_on_a = SeqAckPath::new(&packet.port_id_on_a, &packet.chan_id_on_a);
        let next_seq_ack = ctx_a.get_next_sequence_ack(&seq_ack_path_on_a)?;
        if packet.seq_on_a != next_seq_ack {
            return Err(PacketError::InvalidPacketSequence {
                given_sequence: packet.seq_on_a,
                next_sequence: next_seq_ack,
            }
            .into());
        }
    }

    // Verify proofs
    {
        let client_id_on_a = conn_end_on_a.client_id();

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
        let ack_path_on_b =
            AckPath::new(&packet.port_id_on_b, &packet.chan_id_on_b, packet.seq_on_a);

        verify_conn_delay_passed(ctx_a, msg.proof_height_on_b, &conn_end_on_a)?;

        // Verify the proof for the packet against the chain store.
        client_state_of_b_on_a
            .verify_membership(
                conn_end_on_a.counterparty().prefix(),
                &msg.proof_acked_on_b,
                consensus_state_of_b_on_a.root(),
                Path::Ack(ack_path_on_b),
                ack_commitment.into_vec(),
            )
            .map_err(|e| ChannelError::PacketVerificationFailed {
                sequence: packet.seq_on_a,
                client_error: e,
            })
            .map_err(PacketError::Channel)?;
    }

    Ok(())
}
