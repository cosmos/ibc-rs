//! Implements IBC handlers responsible for processing Non-Fungible Token
//! Transfers (ICS-721) messages.
mod on_recv_packet;
mod send_transfer;

pub use on_recv_packet::*;
pub use send_transfer::*;

use ibc_core::channel::types::packet::Packet;
pub use on_recv_packet::*;
pub use send_transfer::*;

use crate::types::error::NftTransferError;
use crate::types::is_sender_chain_source;
use crate::types::packet::PacketData;

use crate::context::{NftTransferExecutionContext, NftTransferValidationContext};

pub fn refund_packet_nft_execute<N, C>(
    ctx_a: &mut impl NftTransferExecutionContext<N, C>,
    packet: &Packet,
    data: &PacketData,
) -> Result<(), NftTransferError> {
    let sender = data
        .sender
        .clone()
        .try_into()
        .map_err(|_| NftTransferError::ParseAccountFailure)?;

    if is_sender_chain_source(
        packet.port_id_on_a.clone(),
        packet.chan_id_on_a.clone(),
        &data.class_id,
    ) {
        data.token_ids.as_ref().iter().try_for_each(|token_id| {
            ctx_a.unescrow_nft_execute(
                &sender,
                &packet.port_id_on_a,
                &packet.chan_id_on_a,
                &data.class_id,
                token_id,
            )
        })
    }
    // mint vouchers back to sender
    else {
        data.token_ids
            .0
            .iter()
            .zip(data.token_uris.iter())
            .zip(data.token_data.iter())
            .try_for_each(|((token_id, token_uri), token_data)| {
                ctx_a.mint_nft_execute(&sender, &data.class_id, token_id, token_uri, token_data)
            })
    }
}

pub fn refund_packet_nft_validate<N, C>(
    ctx_a: &impl NftTransferValidationContext<N, C>,
    packet: &Packet,
    data: &PacketData,
) -> Result<(), NftTransferError> {
    let sender = data
        .sender
        .clone()
        .try_into()
        .map_err(|_| NftTransferError::ParseAccountFailure)?;

    if is_sender_chain_source(
        packet.port_id_on_a.clone(),
        packet.chan_id_on_a.clone(),
        &data.class_id,
    ) {
        data.token_ids.0.iter().try_for_each(|token_id| {
            ctx_a.unescrow_nft_validate(
                &sender,
                &packet.port_id_on_a,
                &packet.chan_id_on_a,
                &data.class_id,
                token_id,
            )
        })
    } else {
        data.token_ids
            .0
            .iter()
            .zip(data.token_uris.iter())
            .zip(data.token_data.iter())
            .try_for_each(|((token_id, token_uri), token_data)| {
                ctx_a.mint_nft_validate(&sender, &data.class_id, token_id, token_uri, token_data)
            })
    }
}
