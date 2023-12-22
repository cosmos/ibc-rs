use ibc_app_transfer_types::error::TokenTransferError;
use ibc_app_transfer_types::events::TransferEvent;
use ibc_app_transfer_types::msgs::transfer::MsgTransfer;
use ibc_app_transfer_types::{is_sender_chain_source, MODULE_ID_STR};
use ibc_core::channel::context::{SendPacketExecutionContext, SendPacketValidationContext};
use ibc_core::channel::handler::{send_packet_execute, send_packet_validate};
use ibc_core::channel::types::packet::Packet;
use ibc_core::handler::types::events::MessageEvent;
use ibc_core::host::types::path::{ChannelEndPath, SeqSendPath};
use ibc_core::primitives::prelude::*;
use ibc_core::router::types::event::ModuleEvent;

use crate::context::{TokenTransferExecutionContext, TokenTransferValidationContext};

/// Initiate a token transfer. Equivalent to calling [`send_transfer_validate`], followed by [`send_transfer_execute`].
pub fn send_transfer<SendPacketCtx, TokenCtx>(
    send_packet_ctx_a: &mut SendPacketCtx,
    token_ctx_a: &mut TokenCtx,
    msg: MsgTransfer,
) -> Result<(), TokenTransferError>
where
    SendPacketCtx: SendPacketExecutionContext,
    TokenCtx: TokenTransferExecutionContext,
{
    send_transfer_validate(send_packet_ctx_a, token_ctx_a, msg.clone())?;
    send_transfer_execute(send_packet_ctx_a, token_ctx_a, msg)
}

/// Validates the token transfer. If this succeeds, then it is legal to initiate the transfer with [`send_transfer_execute`].
pub fn send_transfer_validate<SendPacketCtx, TokenCtx>(
    send_packet_ctx_a: &SendPacketCtx,
    token_ctx_a: &TokenCtx,
    msg: MsgTransfer,
) -> Result<(), TokenTransferError>
where
    SendPacketCtx: SendPacketValidationContext,
    TokenCtx: TokenTransferValidationContext,
{
    token_ctx_a.can_send_coins()?;

    let chan_end_path_on_a = ChannelEndPath::new(&msg.port_id_on_a, &msg.chan_id_on_a);
    let chan_end_on_a = send_packet_ctx_a.channel_end(&chan_end_path_on_a)?;

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
    let sequence = send_packet_ctx_a.get_next_sequence_send(&seq_send_path_on_a)?;

    let token = &msg.packet_data.token;

    let sender: TokenCtx::AccountId = msg
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
        token_ctx_a.escrow_coins_validate(
            &sender,
            &msg.port_id_on_a,
            &msg.chan_id_on_a,
            token,
            &msg.packet_data.memo,
        )?;
    } else {
        token_ctx_a.burn_coins_validate(&sender, token, &msg.packet_data.memo)?;
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

    send_packet_validate(send_packet_ctx_a, &packet)?;

    Ok(())
}

/// Executes the token transfer. A prior call to [`send_transfer_validate`] MUST have succeeded.
pub fn send_transfer_execute<SendPacketCtx, TokenCtx>(
    send_packet_ctx_a: &mut SendPacketCtx,
    token_ctx_a: &mut TokenCtx,
    msg: MsgTransfer,
) -> Result<(), TokenTransferError>
where
    SendPacketCtx: SendPacketExecutionContext,
    TokenCtx: TokenTransferExecutionContext,
{
    let chan_end_path_on_a = ChannelEndPath::new(&msg.port_id_on_a, &msg.chan_id_on_a);
    let chan_end_on_a = send_packet_ctx_a.channel_end(&chan_end_path_on_a)?;

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
    let sequence = send_packet_ctx_a.get_next_sequence_send(&seq_send_path_on_a)?;

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
        token_ctx_a.escrow_coins_execute(
            &sender,
            &msg.port_id_on_a,
            &msg.chan_id_on_a,
            token,
            &msg.packet_data.memo,
        )?;
    } else {
        token_ctx_a.burn_coins_execute(&sender, token, &msg.packet_data.memo)?;
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

    send_packet_execute(send_packet_ctx_a, packet)?;

    {
        send_packet_ctx_a.log_message(format!(
            "IBC fungible token transfer: {} --({})--> {}",
            msg.packet_data.sender, token, msg.packet_data.receiver
        ))?;

        let transfer_event = TransferEvent {
            sender: msg.packet_data.sender,
            receiver: msg.packet_data.receiver,
            amount: msg.packet_data.token.amount,
            denom: msg.packet_data.token.denom,
            memo: msg.packet_data.memo,
        };
        send_packet_ctx_a.emit_ibc_event(ModuleEvent::from(transfer_event).into())?;

        send_packet_ctx_a.emit_ibc_event(MessageEvent::Module(MODULE_ID_STR.to_string()).into())?;
    }

    Ok(())
}
