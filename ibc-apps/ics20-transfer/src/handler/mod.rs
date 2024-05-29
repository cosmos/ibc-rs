//! Implements the processing logic for ICS20 (token transfer) message.
mod on_recv_packet;
mod send_transfer;

use ibc_app_transfer_types::error::TokenTransferError;
use ibc_app_transfer_types::is_sender_chain_source;
use ibc_app_transfer_types::packet::PacketData;
use ibc_core::channel::types::packet::Packet;
pub use on_recv_packet::*;
pub use send_transfer::*;

use crate::context::{TokenTransferExecutionContext, TokenTransferValidationContext};

pub fn refund_packet_token_execute(
    ctx_a: &mut impl TokenTransferExecutionContext,
    packet: &Packet,
    data: &PacketData,
) -> Result<(), TokenTransferError> {
    let sender = data
        .sender
        .clone()
        .try_into()
        .map_err(|_| TokenTransferError::ParseAccountFailure)?;

    if is_sender_chain_source(
        packet.port_id_on_a.clone(),
        packet.chan_id_on_a.clone(),
        &data.token.denom,
    ) {
        ctx_a.unescrow_coins_execute(
            &sender,
            &packet.port_id_on_a,
            &packet.chan_id_on_a,
            &data.token,
        )
    }
    // mint vouchers back to sender
    else {
        ctx_a.mint_coins_execute(&sender, &data.token)
    }
}

pub fn refund_packet_token_validate(
    ctx_a: &impl TokenTransferValidationContext,
    packet: &Packet,
    data: &PacketData,
) -> Result<(), TokenTransferError> {
    let sender = data
        .sender
        .clone()
        .try_into()
        .map_err(|_| TokenTransferError::ParseAccountFailure)?;

    if is_sender_chain_source(
        packet.port_id_on_a.clone(),
        packet.chan_id_on_a.clone(),
        &data.token.denom,
    ) {
        ctx_a.unescrow_coins_validate(
            &sender,
            &packet.port_id_on_a,
            &packet.chan_id_on_a,
            &data.token,
        )
    } else {
        ctx_a.mint_coins_validate(&sender, &data.token)
    }
}
