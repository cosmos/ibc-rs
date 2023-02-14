//! This module implements the processing logic for ICS20 (token transfer) message.
use crate::applications::transfer::context::TokenTransferContext;
use crate::applications::transfer::error::TokenTransferError;
use crate::applications::transfer::is_sender_chain_source;
use crate::applications::transfer::packet::PacketData;
use crate::core::ics04_channel::packet::Packet;
use crate::prelude::*;

pub mod on_recv_packet;
pub mod send_transfer;

pub fn refund_packet_token(
    ctx: &mut impl TokenTransferContext,
    packet: &Packet,
    data: &PacketData,
) -> Result<(), TokenTransferError> {
    let sender = data
        .sender
        .clone()
        .try_into()
        .map_err(|_| TokenTransferError::ParseAccountFailure)?;

    if is_sender_chain_source(
        packet.port_on_a.clone(),
        packet.chan_on_a.clone(),
        &data.token.denom,
    ) {
        // unescrow tokens back to sender
        let escrow_address =
            ctx.get_channel_escrow_address(&packet.port_on_a, &packet.chan_on_a)?;

        ctx.send_coins(&escrow_address, &sender, &data.token)
    }
    // mint vouchers back to sender
    else {
        ctx.mint_coins(&sender, &data.token)
    }
}

pub fn refund_packet_token_validate<Ctx: TokenTransferContext>(
    data: &PacketData,
) -> Result<(), TokenTransferError> {
    let _sender: <Ctx as TokenTransferContext>::AccountId = data
        .sender
        .clone()
        .try_into()
        .map_err(|_| TokenTransferError::ParseAccountFailure)?;

    Ok(())
}
