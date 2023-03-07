use crate::applications::transfer::context::TokenTransferExecutionContext;
use crate::applications::transfer::error::TokenTransferError;
use crate::applications::transfer::events::DenomTraceEvent;
use crate::applications::transfer::packet::PacketData;
use crate::applications::transfer::{is_receiver_chain_source, TracePrefix};
use crate::core::ics04_channel::handler::ModuleExtras;
use crate::core::ics04_channel::packet::Packet;
use crate::prelude::*;
use crate::utils::hash::hash;
use core::str::from_utf8;

pub fn process_recv_packet_execute<Ctx: TokenTransferExecutionContext>(
    ctx: &mut Ctx,
    packet: &Packet,
    data: PacketData,
) -> Result<ModuleExtras, TokenTransferError> {
    ctx.is_receive_enabled()?;

    let receiver_account = data
        .receiver
        .clone()
        .try_into()
        .map_err(|_| TokenTransferError::ParseAccountFailure)?;

    if is_receiver_chain_source(
        packet.port_id_on_a.clone(),
        packet.chan_id_on_a.clone(),
        &data.token.denom,
    ) {
        // sender chain is not the source, unescrow tokens
        let prefix = TracePrefix::new(packet.port_id_on_a.clone(), packet.chan_id_on_a.clone());
        let coin = {
            let mut c = data.token;
            c.denom.remove_trace_prefix(&prefix);
            c
        };

        // Check if the receiver account on the source (host) chain is not blocked.
        ctx.is_account_blocked(&receiver_account)?;

        let escrow_account = ctx.get_escrow_account(&packet.port_id_on_b, &packet.chan_id_on_b)?;

        ctx.send_coins(&escrow_account, &receiver_account, &coin)?;

        Ok(ModuleExtras::empty())
    } else {
        // sender chain is the source, mint vouchers
        let prefix = TracePrefix::new(packet.port_id_on_b.clone(), packet.chan_id_on_b.clone());
        let coin = {
            let mut c = data.token;
            c.denom.add_trace_prefix(prefix);
            c
        };

        let denom_hash = hash(coin.denom.to_string().as_bytes());
        if ctx.get_prefixed_denom(denom_hash)?.is_none() {
            ctx.set_prefixed_denom(coin.denom.clone())?;
        }

        let mut extras = {
            let denom_trace_event = DenomTraceEvent {
                trace_hash: from_utf8(&denom_hash)
                    .map_err(TokenTransferError::Utf8Decode)?
                    .to_string(),
                denom: coin.denom.clone(),
            };
            ModuleExtras {
                events: vec![denom_trace_event.into()],
                log: Vec::new(),
            }
        };

        if let Err(e) = ctx.mint_coins(&receiver_account, &coin) {
            extras.log.push(e.to_string());
            return Err(TokenTransferError::CannotMintCoins { extras });
        };

        Ok(extras)
    }
}
