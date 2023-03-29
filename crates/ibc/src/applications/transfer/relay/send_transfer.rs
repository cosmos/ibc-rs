use crate::applications::transfer::context::{
    TokenTransferExecutionContext, TokenTransferValidationContext,
};
use crate::applications::transfer::error::TokenTransferError;
use crate::applications::transfer::events::TransferEvent;
use crate::applications::transfer::is_sender_chain_source;
use crate::applications::transfer::msgs::transfer::MsgTransfer;
use crate::core::ics04_channel::handler::send_packet::{send_packet_execute, send_packet_validate};
use crate::core::ics04_channel::packet::Packet;
use crate::core::ics24_host::path::{ChannelEndPath, SeqSendPath};
use crate::events::ModuleEvent;
use crate::prelude::*;

/// This function handles the transfer sending logic.
/// If this method returns an error, the runtime is expected to rollback all state modifications to
/// the `Ctx` caused by all messages from the transaction that this `msg` is a part of.
pub fn send_transfer<Ctx>(ctx_a: &mut Ctx, msg: MsgTransfer) -> Result<(), TokenTransferError>
where
    Ctx: TokenTransferExecutionContext,
{
    send_transfer_validate(ctx_a, msg.clone())?;
    send_transfer_execute(ctx_a, msg)
}

pub fn send_transfer_validate<Ctx>(ctx_a: &Ctx, msg: MsgTransfer) -> Result<(), TokenTransferError>
where
    Ctx: TokenTransferValidationContext,
{
    ctx_a.can_send_coins()?;

    let chan_end_path_on_a = ChannelEndPath::new(&msg.port_id_on_a, &msg.chan_id_on_a);
    let chan_end_on_a = ctx_a
        .channel_end(&chan_end_path_on_a)
        .map_err(TokenTransferError::ContextError)?;

    let port_id_on_b = chan_end_on_a.counterparty().port_id().clone();
    let chan_id_on_b = chan_end_on_a
        .counterparty()
        .channel_id()
        .ok_or_else(|| TokenTransferError::DestinationChannelNotFound {
            port_id: msg.port_id_on_a.clone(),
            channel_id: msg.chan_id_on_a.clone(),
        })?
        .clone();

    let seq_send_path_on_a = SeqSendPath::new(&msg.port_id_on_a, &msg.chan_id_on_a);
    let sequence = ctx_a
        .get_next_sequence_send(&seq_send_path_on_a)
        .map_err(TokenTransferError::ContextError)?;

    let token = &msg.packet_data.token;

    let sender: Ctx::AccountId = msg
        .packet_data
        .sender
        .clone()
        .try_into()
        .map_err(|_| TokenTransferError::ParseAccountFailure)?;

    if is_sender_chain_source(
        msg.port_id_on_a.clone(),
        msg.chan_id_on_a.clone(),
        &token.denom,
    ) {
        let escrow_address = ctx_a.get_escrow_account(&msg.port_id_on_a, &msg.chan_id_on_a)?;
        ctx_a.send_coins_validate(&sender, &escrow_address, token)?;
    } else {
        ctx_a.burn_coins_validate(&sender, token)?;
    }

    let packet = {
        let data = serde_json::to_vec(&msg.packet_data)
            .expect("PacketData's infallible Serialize impl failed");

        Packet {
            seq_on_a: sequence,
            port_id_on_a: msg.port_id_on_a,
            chan_id_on_a: msg.chan_id_on_a,
            port_id_on_b,
            chan_id_on_b,
            data,
            timeout_height_on_b: msg.timeout_height_on_b,
            timeout_timestamp_on_b: msg.timeout_timestamp_on_b,
        }
    };

    send_packet_validate(ctx_a, &packet).map_err(TokenTransferError::ContextError)?;

    Ok(())
}

pub fn send_transfer_execute<Ctx>(
    ctx_a: &mut Ctx,
    msg: MsgTransfer,
) -> Result<(), TokenTransferError>
where
    Ctx: TokenTransferExecutionContext,
{
    let chan_end_path_on_a = ChannelEndPath::new(&msg.port_id_on_a, &msg.chan_id_on_a);
    let chan_end_on_a = ctx_a
        .channel_end(&chan_end_path_on_a)
        .map_err(TokenTransferError::ContextError)?;

    let port_on_b = chan_end_on_a.counterparty().port_id().clone();
    let chan_on_b = chan_end_on_a
        .counterparty()
        .channel_id()
        .ok_or_else(|| TokenTransferError::DestinationChannelNotFound {
            port_id: msg.port_id_on_a.clone(),
            channel_id: msg.chan_id_on_a.clone(),
        })?
        .clone();

    // get the next sequence
    let seq_send_path_on_a = SeqSendPath::new(&msg.port_id_on_a, &msg.chan_id_on_a);
    let sequence = ctx_a
        .get_next_sequence_send(&seq_send_path_on_a)
        .map_err(TokenTransferError::ContextError)?;

    let token = &msg.packet_data.token;

    let sender = msg
        .packet_data
        .sender
        .clone()
        .try_into()
        .map_err(|_| TokenTransferError::ParseAccountFailure)?;

    if is_sender_chain_source(
        msg.port_id_on_a.clone(),
        msg.chan_id_on_a.clone(),
        &token.denom,
    ) {
        let escrow_address = ctx_a.get_escrow_account(&msg.port_id_on_a, &msg.chan_id_on_a)?;
        ctx_a.send_coins_execute(&sender, &escrow_address, token)?;
    } else {
        ctx_a.burn_coins_execute(&sender, token)?;
    }

    let packet = {
        let data = {
            serde_json::to_vec(&msg.packet_data)
                .expect("PacketData's infallible Serialize impl failed")
        };

        Packet {
            seq_on_a: sequence,
            port_id_on_a: msg.port_id_on_a,
            chan_id_on_a: msg.chan_id_on_a,
            port_id_on_b: port_on_b,
            chan_id_on_b: chan_on_b,
            data,
            timeout_height_on_b: msg.timeout_height_on_b,
            timeout_timestamp_on_b: msg.timeout_timestamp_on_b,
        }
    };

    send_packet_execute(ctx_a, packet).map_err(TokenTransferError::ContextError)?;

    {
        ctx_a.log_message(format!(
            "IBC fungible token transfer: {} --({})--> {}",
            msg.packet_data.sender, token, msg.packet_data.receiver
        ));

        let transfer_event = TransferEvent {
            sender: msg.packet_data.sender,
            receiver: msg.packet_data.receiver,
        };
        ctx_a.emit_ibc_event(ModuleEvent::from(transfer_event).into());
    }

    Ok(())
}
