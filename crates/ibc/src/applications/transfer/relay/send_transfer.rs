use crate::applications::transfer::context::TokenTransferContext;
use crate::applications::transfer::error::TokenTransferError;
use crate::applications::transfer::events::TransferEvent;
use crate::applications::transfer::msgs::transfer::MsgTransfer;
use crate::applications::transfer::packet::PacketData;
use crate::applications::transfer::{is_sender_chain_source, Coin, PrefixedCoin};
use crate::core::ics04_channel::handler::send_packet::send_packet;
use crate::core::ics04_channel::packet::Packet;
use crate::core::ics24_host::path::{ChannelEndPath, SeqSendPath};
use crate::events::ModuleEvent;
use crate::handler::{HandlerOutput, HandlerOutputBuilder};
use crate::prelude::*;

/// This function handles the transfer sending logic.
/// If this method returns an error, the runtime is expected to rollback all state modifications to
/// the `Ctx` caused by all messages from the transaction that this `msg` is a part of.
pub fn send_transfer<Ctx, C>(
    ctx: &mut Ctx,
    output: &mut HandlerOutputBuilder<()>,
    msg: MsgTransfer<C>,
) -> Result<(), TokenTransferError>
where
    Ctx: TokenTransferContext,
    C: TryInto<PrefixedCoin>,
{
    if !ctx.is_send_enabled() {
        return Err(TokenTransferError::SendDisabled);
    }
    let chan_end_path_on_a = ChannelEndPath::new(&msg.port_on_a, &msg.chan_on_a);
    let chan_end_on_a = ctx
        .channel_end(&chan_end_path_on_a)
        .map_err(TokenTransferError::ContextError)?;

    let port_on_b = chan_end_on_a.counterparty().port_id().clone();
    let chan_on_b = chan_end_on_a
        .counterparty()
        .channel_id()
        .ok_or_else(|| TokenTransferError::DestinationChannelNotFound {
            port_id: msg.port_on_a.clone(),
            channel_id: msg.chan_on_a.clone(),
        })?
        .clone();

    // get the next sequence
    let seq_send_path_on_a = SeqSendPath::new(&msg.port_on_a, &msg.chan_on_a);
    let sequence = ctx
        .get_next_sequence_send(&seq_send_path_on_a)
        .map_err(TokenTransferError::ContextError)?;

    let token = msg
        .token
        .try_into()
        .map_err(|_| TokenTransferError::InvalidToken)?;
    let denom = token.denom.clone();
    let coin = Coin {
        denom: denom.clone(),
        amount: token.amount,
    };

    let sender = msg
        .sender
        .clone()
        .try_into()
        .map_err(|_| TokenTransferError::ParseAccountFailure)?;

    if is_sender_chain_source(msg.port_on_a.clone(), msg.chan_on_a.clone(), &denom) {
        let escrow_address = ctx.get_channel_escrow_address(&msg.port_on_a, &msg.chan_on_a)?;
        ctx.send_coins(&sender, &escrow_address, &coin)?;
    } else {
        ctx.burn_coins(&sender, &coin)?;
    }

    let data = {
        let data = PacketData {
            token: coin,
            sender: msg.sender.clone(),
            receiver: msg.receiver.clone(),
        };
        serde_json::to_vec(&data).expect("PacketData's infallible Serialize impl failed")
    };

    let packet = Packet {
        sequence,
        port_on_a: msg.port_on_a,
        chan_on_a: msg.chan_on_a,
        port_on_b,
        chan_on_b,
        data,
        timeout_height_on_b: msg.timeout_height_on_b,
        timeout_timestamp_on_b: msg.timeout_timestamp_on_b,
    };

    let HandlerOutput {
        result,
        log,
        events,
    } = send_packet(ctx, packet).map_err(TokenTransferError::ContextError)?;

    ctx.store_send_packet_result(result)
        .map_err(TokenTransferError::ContextError)?;

    output.merge_output(
        HandlerOutput::builder()
            .with_log(log)
            .with_events(events)
            .with_result(()),
    );

    output.log(format!(
        "IBC fungible token transfer: {} --({})--> {}",
        msg.sender, token, msg.receiver
    ));

    let transfer_event = TransferEvent {
        sender: msg.sender,
        receiver: msg.receiver,
    };
    output.emit(ModuleEvent::from(transfer_event).into());

    Ok(())
}
