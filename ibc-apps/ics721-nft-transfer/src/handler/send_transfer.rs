use ibc_core::channel::context::{SendPacketExecutionContext, SendPacketValidationContext};
use ibc_core::channel::handler::{send_packet_execute, send_packet_validate};
use ibc_core::channel::types::packet::Packet;
use ibc_core::handler::types::events::MessageEvent;
use ibc_core::host::types::path::{ChannelEndPath, SeqSendPath};
use ibc_core::primitives::prelude::*;
use ibc_core::router::types::event::ModuleEvent;

use crate::context::{NftTransferExecutionContext, NftTransferValidationContext};
use crate::types::error::NftTransferError;
use crate::types::events::TransferEvent;
use crate::types::msgs::transfer::MsgTransfer;
use crate::types::{is_sender_chain_source, MODULE_ID_STR};

/// Initiate a token transfer. Equivalent to calling [`send_nft_transfer_validate`], followed by [`send_nft_transfer_execute`].
pub fn send_nft_transfer<SendPacketCtx, NftCtx, N, C>(
    send_packet_ctx_a: &mut SendPacketCtx,
    nft_ctx: &mut NftCtx,
    msg: MsgTransfer,
) -> Result<(), NftTransferError>
where
    SendPacketCtx: SendPacketExecutionContext,
    NftCtx: NftTransferExecutionContext<N, C>,
{
    send_nft_transfer_validate(send_packet_ctx_a, nft_ctx, msg.clone())?;
    send_nft_transfer_execute(send_packet_ctx_a, nft_ctx, msg)
}

/// Validates the token transfer
pub fn send_nft_transfer_validate<SendPacketCtx, NftCtx, N, C>(
    send_packet_ctx_a: &SendPacketCtx,
    nft_ctx: &NftCtx,
    msg: MsgTransfer,
) -> Result<(), NftTransferError>
where
    SendPacketCtx: SendPacketValidationContext,
    NftCtx: NftTransferValidationContext<N, C>,
{
    nft_ctx.can_send_nft()?;

    let chan_end_path_on_a = ChannelEndPath::new(&msg.port_id_on_a, &msg.chan_id_on_a);
    let chan_end_on_a = send_packet_ctx_a.channel_end(&chan_end_path_on_a)?;

    let port_id_on_b = chan_end_on_a.counterparty().port_id().clone();
    let chan_id_on_b = chan_end_on_a
        .counterparty()
        .channel_id()
        .ok_or_else(|| NftTransferError::DestinationChannelNotFound {
            port_id: msg.port_id_on_a.clone(),
            channel_id: msg.chan_id_on_a.clone(),
        })?
        .clone();

    let seq_send_path_on_a = SeqSendPath::new(&msg.port_id_on_a, &msg.chan_id_on_a);
    let sequence = send_packet_ctx_a.get_next_sequence_send(&seq_send_path_on_a)?;

    let class_id = &msg.packet_data.class_id;
    let token_ids = &msg.packet_data.token_ids;

    let sender: NftCtx::AccountId = msg
        .packet_data
        .sender
        .clone()
        .try_into()
        .map_err(|_| NftTransferError::ParseAccountFailure)?;

    if is_sender_chain_source(msg.port_id_on_a.clone(), msg.chan_id_on_a.clone(), class_id) {
        token_ids.as_ref().iter().try_for_each(|token_id| {
            nft_ctx.escrow_nft_validate(
                &sender,
                &msg.port_id_on_a,
                &msg.chan_id_on_a,
                class_id,
                token_id,
                &msg.packet_data.memo,
            )
        })?;
    } else {
        token_ids.as_ref().iter().try_for_each(|token_id| {
            nft_ctx.burn_nft_validate(&sender, class_id, token_id, &msg.packet_data.memo)
        })?;
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
pub fn send_nft_transfer_execute<SendPacketCtx, NftCtx, N, C>(
    send_packet_ctx_a: &mut SendPacketCtx,
    nft_ctx: &mut NftCtx,
    msg: MsgTransfer,
) -> Result<(), NftTransferError>
where
    SendPacketCtx: SendPacketExecutionContext,
    NftCtx: NftTransferExecutionContext<N, C>,
{
    let chan_end_path_on_a = ChannelEndPath::new(&msg.port_id_on_a, &msg.chan_id_on_a);
    let chan_end_on_a = send_packet_ctx_a.channel_end(&chan_end_path_on_a)?;

    let port_on_b = chan_end_on_a.counterparty().port_id().clone();
    let chan_on_b = chan_end_on_a
        .counterparty()
        .channel_id()
        .ok_or_else(|| NftTransferError::DestinationChannelNotFound {
            port_id: msg.port_id_on_a.clone(),
            channel_id: msg.chan_id_on_a.clone(),
        })?
        .clone();

    // get the next sequence
    let seq_send_path_on_a = SeqSendPath::new(&msg.port_id_on_a, &msg.chan_id_on_a);
    let sequence = send_packet_ctx_a.get_next_sequence_send(&seq_send_path_on_a)?;

    let class_id = &msg.packet_data.class_id;
    let token_ids = &msg.packet_data.token_ids;

    let sender = msg
        .packet_data
        .sender
        .clone()
        .try_into()
        .map_err(|_| NftTransferError::ParseAccountFailure)?;

    if is_sender_chain_source(msg.port_id_on_a.clone(), msg.chan_id_on_a.clone(), class_id) {
        token_ids.as_ref().iter().try_for_each(|token_id| {
            nft_ctx.escrow_nft_execute(
                &sender,
                &msg.port_id_on_a,
                &msg.chan_id_on_a,
                class_id,
                token_id,
                &msg.packet_data.memo,
            )
        })?;
    } else {
        token_ids.as_ref().iter().try_for_each(|token_id| {
            nft_ctx.burn_nft_execute(&sender, class_id, token_id, &msg.packet_data.memo)
        })?;
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
            "IBC NFT transfer: {} --({}, ({}))--> {}",
            msg.packet_data.sender, class_id, token_ids, msg.packet_data.receiver
        ))?;

        let transfer_event = TransferEvent {
            sender: msg.packet_data.sender,
            receiver: msg.packet_data.receiver,
            class: msg.packet_data.class_id,
            tokens: msg.packet_data.token_ids,
            memo: msg.packet_data.memo,
        };
        send_packet_ctx_a.emit_ibc_event(ModuleEvent::from(transfer_event).into())?;

        send_packet_ctx_a.emit_ibc_event(MessageEvent::Module(MODULE_ID_STR.to_string()).into())?;
    }

    Ok(())
}
