use ibc_core_channel_types::channel::{Counterparty, Order, State as ChannelState};
use ibc_core_channel_types::commitment::{compute_ack_commitment, compute_packet_commitment};
use ibc_core_channel_types::error::{ChannelError, PacketError};
use ibc_core_channel_types::events::{ReceivePacket, WriteAcknowledgement};
use ibc_core_channel_types::msgs::MsgRecvPacket;
use ibc_core_channel_types::packet::Receipt;
use ibc_core_client::context::prelude::*;
use ibc_core_connection::delay::verify_conn_delay_passed;
use ibc_core_connection::types::State as ConnectionState;
use ibc_core_handler_types::error::ContextError;
use ibc_core_handler_types::events::{IbcEvent, MessageEvent};
use ibc_core_host::types::path::{
    AckPath, ChannelEndPath, ClientConsensusStatePath, CommitmentPath, Path, ReceiptPath,
    SeqRecvPath,
};
use ibc_core_host::{ExecutionContext, ValidationContext};
use ibc_core_router::module::Module;
use ibc_primitives::prelude::*;
use ibc_primitives::Expiry;

pub fn recv_packet_validate<ValCtx>(ctx_b: &ValCtx, msg: MsgRecvPacket) -> Result<(), ContextError>
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
) -> Result<(), ContextError>
where
    ExecCtx: ExecutionContext,
{
    let chan_end_path_on_b =
        ChannelEndPath::new(&msg.packet.port_id_on_b, &msg.packet.chan_id_on_b);
    let chan_end_on_b = ctx_b.channel_end(&chan_end_path_on_b)?;

    // Check if another relayer already relayed the packet.
    // We don't want to fail the transaction in this case.
    {
        let packet_already_received = match chan_end_on_b.ordering {
            // Note: ibc-go doesn't make the check for `Order::None` channels
            Order::None => false,
            Order::Unordered => {
                let packet = &msg.packet;
                let receipt_path_on_b =
                    ReceiptPath::new(&packet.port_id_on_b, &packet.chan_id_on_b, packet.seq_on_a);
                ctx_b.get_packet_receipt(&receipt_path_on_b).is_ok()
            }
            Order::Ordered => {
                let seq_recv_path_on_b =
                    SeqRecvPath::new(&msg.packet.port_id_on_b, &msg.packet.chan_id_on_b);
                let next_seq_recv = ctx_b.get_next_sequence_recv(&seq_recv_path_on_b)?;

                // the sequence number has already been incremented, so
                // another relayer already relayed the packet
                msg.packet.seq_on_a < next_seq_recv
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
                    port_id: msg.packet.port_id_on_b.clone(),
                    channel_id: msg.packet.chan_id_on_b.clone(),
                    sequence: msg.packet.seq_on_a,
                };

                ctx_b.store_packet_receipt(&receipt_path_on_b, Receipt::Ok)?;
            }
            Order::Ordered => {
                let seq_recv_path_on_b =
                    SeqRecvPath::new(&msg.packet.port_id_on_b, &msg.packet.chan_id_on_b);
                let next_seq_recv = ctx_b.get_next_sequence_recv(&seq_recv_path_on_b)?;
                ctx_b.store_next_sequence_recv(&seq_recv_path_on_b, next_seq_recv.increment())?;
            }
            _ => {}
        }
        let ack_path_on_b = AckPath::new(
            &msg.packet.port_id_on_b,
            &msg.packet.chan_id_on_b,
            msg.packet.seq_on_a,
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

        let conn_id_on_b = &chan_end_on_b.connection_hops()[0];
        let event = IbcEvent::ReceivePacket(ReceivePacket::new(
            msg.packet.clone(),
            chan_end_on_b.ordering,
            conn_id_on_b.clone(),
        ));
        ctx_b.emit_ibc_event(IbcEvent::Message(MessageEvent::Channel))?;
        ctx_b.emit_ibc_event(event)?;
        let event = IbcEvent::WriteAcknowledgement(WriteAcknowledgement::new(
            msg.packet,
            acknowledgement,
            conn_id_on_b.clone(),
        ));
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

fn validate<Ctx>(ctx_b: &Ctx, msg: &MsgRecvPacket) -> Result<(), ContextError>
where
    Ctx: ValidationContext,
{
    ctx_b.validate_message_signer(&msg.signer)?;

    let chan_end_path_on_b =
        ChannelEndPath::new(&msg.packet.port_id_on_b, &msg.packet.chan_id_on_b);
    let chan_end_on_b = ctx_b.channel_end(&chan_end_path_on_b)?;

    chan_end_on_b.verify_state_matches(&ChannelState::Open)?;

    let counterparty = Counterparty::new(
        msg.packet.port_id_on_a.clone(),
        Some(msg.packet.chan_id_on_a.clone()),
    );

    chan_end_on_b.verify_counterparty_matches(&counterparty)?;

    let conn_id_on_b = &chan_end_on_b.connection_hops()[0];
    let conn_end_on_b = ctx_b.connection_end(conn_id_on_b)?;

    conn_end_on_b.verify_state_matches(&ConnectionState::Open)?;

    let latest_height = ctx_b.host_height()?;
    if msg.packet.timeout_height_on_b.has_expired(latest_height) {
        return Err(PacketError::LowPacketHeight {
            chain_height: latest_height,
            timeout_height: msg.packet.timeout_height_on_b,
        }
        .into());
    }

    let latest_timestamp = ctx_b.host_timestamp()?;
    if let Expiry::Expired = latest_timestamp.check_expiry(&msg.packet.timeout_timestamp_on_b) {
        return Err(PacketError::LowPacketTimestamp.into());
    }

    // Verify proofs
    {
        let client_id_on_b = conn_end_on_b.client_id();
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
            &msg.packet.data,
            &msg.packet.timeout_height_on_b,
            &msg.packet.timeout_timestamp_on_b,
        );
        let commitment_path_on_a = CommitmentPath::new(
            &msg.packet.port_id_on_a,
            &msg.packet.chan_id_on_a,
            msg.packet.seq_on_a,
        );

        verify_conn_delay_passed(ctx_b, msg.proof_height_on_a, &conn_end_on_b)?;

        // Verify the proof for the packet against the chain store.
        client_state_of_a_on_b
            .verify_membership(
                conn_end_on_b.counterparty().prefix(),
                &msg.proof_commitment_on_a,
                consensus_state_of_a_on_b.root(),
                Path::Commitment(commitment_path_on_a),
                expected_commitment_on_a.into_vec(),
            )
            .map_err(|e| ChannelError::PacketVerificationFailed {
                sequence: msg.packet.seq_on_a,
                client_error: e,
            })
            .map_err(PacketError::Channel)?;
    }

    match chan_end_on_b.ordering {
        Order::Ordered => {
            let seq_recv_path_on_b =
                SeqRecvPath::new(&msg.packet.port_id_on_b, &msg.packet.chan_id_on_b);
            let next_seq_recv = ctx_b.get_next_sequence_recv(&seq_recv_path_on_b)?;
            if msg.packet.seq_on_a > next_seq_recv {
                return Err(PacketError::InvalidPacketSequence {
                    given_sequence: msg.packet.seq_on_a,
                    next_sequence: next_seq_recv,
                }
                .into());
            }

            if msg.packet.seq_on_a == next_seq_recv {
                // Case where the recvPacket is successful and an
                // acknowledgement will be written (not a no-op)
                validate_write_acknowledgement(ctx_b, msg)?;
            }
        }
        Order::Unordered => {
            let receipt_path_on_b = ReceiptPath::new(
                &msg.packet.port_id_on_a,
                &msg.packet.chan_id_on_a,
                msg.packet.seq_on_a,
            );
            let packet_rec = ctx_b.get_packet_receipt(&receipt_path_on_b);
            match packet_rec {
                Ok(_receipt) => {}
                Err(ContextError::PacketError(PacketError::PacketReceiptNotFound { sequence }))
                    if sequence == msg.packet.seq_on_a => {}
                Err(e) => return Err(e),
            }
            // Case where the recvPacket is successful and an
            // acknowledgement will be written (not a no-op)
            validate_write_acknowledgement(ctx_b, msg)?;
        }
        Order::None => {
            return Err(ContextError::ChannelError(ChannelError::InvalidOrderType {
                expected: "Channel ordering cannot be None".to_string(),
                actual: chan_end_on_b.ordering.to_string(),
            }))
        }
    }

    Ok(())
}

fn validate_write_acknowledgement<Ctx>(ctx_b: &Ctx, msg: &MsgRecvPacket) -> Result<(), ContextError>
where
    Ctx: ValidationContext,
{
    let packet = msg.packet.clone();
    let ack_path_on_b = AckPath::new(&packet.port_id_on_b, &packet.chan_id_on_b, packet.seq_on_a);
    if ctx_b.get_packet_acknowledgement(&ack_path_on_b).is_ok() {
        return Err(PacketError::AcknowledgementExists {
            sequence: msg.packet.seq_on_a,
        }
        .into());
    }

    Ok(())
}
