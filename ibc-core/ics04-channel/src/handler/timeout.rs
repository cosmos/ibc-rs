use ibc_core_channel_types::channel::{Counterparty, Order, State};
use ibc_core_channel_types::commitment::compute_packet_commitment;
use ibc_core_channel_types::error::{ChannelError, PacketError};
use ibc_core_channel_types::events::{ChannelClosed, TimeoutPacket};
use ibc_core_channel_types::msgs::{MsgTimeout, MsgTimeoutOnClose};
use ibc_core_client::context::prelude::*;
use ibc_core_connection::delay::verify_conn_delay_passed;
use ibc_core_handler_types::error::ContextError;
use ibc_core_handler_types::events::{IbcEvent, MessageEvent};
use ibc_core_host::types::path::{
    ChannelEndPath, ClientConsensusStatePath, CommitmentPath, Path, ReceiptPath, SeqRecvPath,
};
use ibc_core_host::{ExecutionContext, ValidationContext};
use ibc_core_router::module::Module;
use ibc_primitives::prelude::*;

use super::timeout_on_close;

pub enum TimeoutMsgType {
    Timeout(MsgTimeout),
    TimeoutOnClose(MsgTimeoutOnClose),
}

pub fn timeout_packet_validate<ValCtx>(
    ctx_a: &ValCtx,
    module: &dyn Module,
    timeout_msg_type: TimeoutMsgType,
) -> Result<(), ContextError>
where
    ValCtx: ValidationContext,
{
    match &timeout_msg_type {
        TimeoutMsgType::Timeout(msg) => validate(ctx_a, msg),
        TimeoutMsgType::TimeoutOnClose(msg) => timeout_on_close::validate(ctx_a, msg),
    }?;

    let (packet, signer) = match timeout_msg_type {
        TimeoutMsgType::Timeout(msg) => (msg.packet, msg.signer),
        TimeoutMsgType::TimeoutOnClose(msg) => (msg.packet, msg.signer),
    };

    module
        .on_timeout_packet_validate(&packet, &signer)
        .map_err(ContextError::PacketError)
}

pub fn timeout_packet_execute<ExecCtx>(
    ctx_a: &mut ExecCtx,
    module: &mut dyn Module,
    timeout_msg_type: TimeoutMsgType,
) -> Result<(), ContextError>
where
    ExecCtx: ExecutionContext,
{
    let (packet, signer) = match timeout_msg_type {
        TimeoutMsgType::Timeout(msg) => (msg.packet, msg.signer),
        TimeoutMsgType::TimeoutOnClose(msg) => (msg.packet, msg.signer),
    };
    let chan_end_path_on_a = ChannelEndPath::new(&packet.port_id_on_a, &packet.chan_id_on_a);
    let chan_end_on_a = ctx_a.channel_end(&chan_end_path_on_a)?;

    // In all cases, this event is emitted
    let event = IbcEvent::TimeoutPacket(TimeoutPacket::new(packet.clone(), chan_end_on_a.ordering));
    ctx_a.emit_ibc_event(IbcEvent::Message(MessageEvent::Channel))?;
    ctx_a.emit_ibc_event(event)?;

    let commitment_path_on_a =
        CommitmentPath::new(&packet.port_id_on_a, &packet.chan_id_on_a, packet.seq_on_a);

    // check if we're in the NO-OP case
    if ctx_a.get_packet_commitment(&commitment_path_on_a).is_err() {
        // This error indicates that the timeout has already been relayed
        // or there is a misconfigured relayer attempting to prove a timeout
        // for a packet never sent. Core IBC will treat this error as a no-op in order to
        // prevent an entire relay transaction from failing and consuming unnecessary fees.
        return Ok(());
    };

    let (extras, cb_result) = module.on_timeout_packet_execute(&packet, &signer);

    cb_result?;

    // apply state changes
    let chan_end_on_a = {
        ctx_a.delete_packet_commitment(&commitment_path_on_a)?;

        if let Order::Ordered = chan_end_on_a.ordering {
            let mut chan_end_on_a = chan_end_on_a;
            chan_end_on_a.state = State::Closed;
            ctx_a.store_channel(&chan_end_path_on_a, chan_end_on_a.clone())?;

            chan_end_on_a
        } else {
            chan_end_on_a
        }
    };

    // emit events and logs
    {
        ctx_a.log_message("success: packet timeout".to_string())?;

        if let Order::Ordered = chan_end_on_a.ordering {
            let conn_id_on_a = chan_end_on_a.connection_hops()[0].clone();

            let event = IbcEvent::ChannelClosed(ChannelClosed::new(
                packet.port_id_on_a.clone(),
                packet.chan_id_on_a.clone(),
                chan_end_on_a.counterparty().port_id.clone(),
                chan_end_on_a.counterparty().channel_id.clone(),
                conn_id_on_a,
                chan_end_on_a.ordering,
            ));
            ctx_a.emit_ibc_event(IbcEvent::Message(MessageEvent::Channel))?;
            ctx_a.emit_ibc_event(event)?;
        }

        for module_event in extras.events {
            ctx_a.emit_ibc_event(IbcEvent::Module(module_event))?;
        }

        for log_message in extras.log {
            ctx_a.log_message(log_message)?;
        }
    }

    Ok(())
}

fn validate<Ctx>(ctx_a: &Ctx, msg: &MsgTimeout) -> Result<(), ContextError>
where
    Ctx: ValidationContext,
{
    ctx_a.validate_message_signer(&msg.signer)?;

    let chan_end_on_a = ctx_a.channel_end(&ChannelEndPath::new(
        &msg.packet.port_id_on_a,
        &msg.packet.chan_id_on_a,
    ))?;

    chan_end_on_a.verify_state_matches(&State::Open)?;

    let counterparty = Counterparty::new(
        msg.packet.port_id_on_b.clone(),
        Some(msg.packet.chan_id_on_b.clone()),
    );

    chan_end_on_a.verify_counterparty_matches(&counterparty)?;

    let conn_id_on_a = chan_end_on_a.connection_hops()[0].clone();
    let conn_end_on_a = ctx_a.connection_end(&conn_id_on_a)?;

    //verify packet commitment
    let commitment_path_on_a = CommitmentPath::new(
        &msg.packet.port_id_on_a,
        &msg.packet.chan_id_on_a,
        msg.packet.seq_on_a,
    );
    let Ok(commitment_on_a) = ctx_a.get_packet_commitment(&commitment_path_on_a) else {
        // This error indicates that the timeout has already been relayed
        // or there is a misconfigured relayer attempting to prove a timeout
        // for a packet never sent. Core IBC will treat this error as a no-op in order to
        // prevent an entire relay transaction from failing and consuming unnecessary fees.
        return Ok(());
    };

    let expected_commitment_on_a = compute_packet_commitment(
        &msg.packet.data,
        &msg.packet.timeout_height_on_b,
        &msg.packet.timeout_timestamp_on_b,
    );
    if commitment_on_a != expected_commitment_on_a {
        return Err(PacketError::IncorrectPacketCommitment {
            sequence: msg.packet.seq_on_a,
        }
        .into());
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

        // check that timeout height or timeout timestamp has passed on the other end
        let client_cons_state_path_on_a = ClientConsensusStatePath::new(
            client_id_on_a.clone(),
            msg.proof_height_on_b.revision_number(),
            msg.proof_height_on_b.revision_height(),
        );
        let consensus_state_of_b_on_a =
            client_val_ctx_a.consensus_state(&client_cons_state_path_on_a)?;
        let timestamp_of_b = consensus_state_of_b_on_a.timestamp();

        if !msg.packet.timed_out(&timestamp_of_b, msg.proof_height_on_b) {
            return Err(PacketError::PacketTimeoutNotReached {
                timeout_height: msg.packet.timeout_height_on_b,
                chain_height: msg.proof_height_on_b,
                timeout_timestamp: msg.packet.timeout_timestamp_on_b,
                chain_timestamp: timestamp_of_b,
            }
            .into());
        }

        verify_conn_delay_passed(ctx_a, msg.proof_height_on_b, &conn_end_on_a)?;

        let next_seq_recv_verification_result = match chan_end_on_a.ordering {
            Order::Ordered => {
                if msg.packet.seq_on_a < msg.next_seq_recv_on_b {
                    return Err(PacketError::InvalidPacketSequence {
                        given_sequence: msg.packet.seq_on_a,
                        next_sequence: msg.next_seq_recv_on_b,
                    }
                    .into());
                }
                let seq_recv_path_on_b =
                    SeqRecvPath::new(&msg.packet.port_id_on_b, &msg.packet.chan_id_on_b);

                client_state_of_b_on_a.verify_membership(
                    conn_end_on_a.counterparty().prefix(),
                    &msg.proof_unreceived_on_b,
                    consensus_state_of_b_on_a.root(),
                    Path::SeqRecv(seq_recv_path_on_b),
                    msg.packet.seq_on_a.to_vec(),
                )
            }
            Order::Unordered => {
                let receipt_path_on_b = ReceiptPath::new(
                    &msg.packet.port_id_on_b,
                    &msg.packet.chan_id_on_b,
                    msg.packet.seq_on_a,
                );

                client_state_of_b_on_a.verify_non_membership(
                    conn_end_on_a.counterparty().prefix(),
                    &msg.proof_unreceived_on_b,
                    consensus_state_of_b_on_a.root(),
                    Path::Receipt(receipt_path_on_b),
                )
            }
            Order::None => {
                return Err(ContextError::ChannelError(ChannelError::InvalidOrderType {
                    expected: "Channel ordering cannot be None".to_string(),
                    actual: chan_end_on_a.ordering.to_string(),
                }))
            }
        };

        next_seq_recv_verification_result
            .map_err(|e| ChannelError::PacketVerificationFailed {
                sequence: msg.next_seq_recv_on_b,
                client_error: e,
            })
            .map_err(PacketError::Channel)?;
    }

    Ok(())
}
