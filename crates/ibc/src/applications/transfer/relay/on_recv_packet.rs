use crate::applications::transfer::context::TokenTransferContext;
use crate::applications::transfer::error::TokenTransferError;
use crate::applications::transfer::events::DenomTraceEvent;
use crate::applications::transfer::packet::PacketData;
use crate::applications::transfer::{is_receiver_chain_source, TracePrefix};
use crate::core::ics04_channel::handler::ModuleExtras;
use crate::core::ics04_channel::packet::Packet;
use crate::core::ics26_routing::context::ModuleOutputBuilder;
use crate::prelude::*;

pub fn process_recv_packet<Ctx: 'static + TokenTransferContext>(
    ctx: &mut Ctx,
    output: &mut ModuleOutputBuilder,
    packet: &Packet,
    data: PacketData,
) -> Result<(), TokenTransferError> {
    if !ctx.is_receive_enabled() {
        return Err(TokenTransferError::ReceiveDisabled);
    }

    let receiver_account = data
        .receiver
        .clone()
        .try_into()
        .map_err(|_| TokenTransferError::ParseAccountFailure)?;

    if is_receiver_chain_source(
        packet.port_on_a.clone(),
        packet.chan_on_a.clone(),
        &data.token.denom,
    ) {
        // sender chain is not the source, unescrow tokens
        let prefix = TracePrefix::new(packet.port_on_a.clone(), packet.chan_on_a.clone());
        let coin = {
            let mut c = data.token;
            c.denom.remove_trace_prefix(&prefix);
            c
        };

        let escrow_address =
            ctx.get_channel_escrow_address(&packet.port_on_b, &packet.chan_on_b)?;

        ctx.send_coins(&escrow_address, &receiver_account, &coin)?;
    } else {
        // sender chain is the source, mint vouchers
        let prefix = TracePrefix::new(packet.port_on_b.clone(), packet.chan_on_b.clone());
        let coin = {
            let mut c = data.token;
            c.denom.add_trace_prefix(prefix);
            c
        };

        let denom_trace_event = DenomTraceEvent {
            trace_hash: ctx.denom_hash_string(&coin.denom),
            denom: coin.denom.clone(),
        };
        output.emit(denom_trace_event.into());

        ctx.mint_coins(&receiver_account, &coin)?;
    }

    Ok(())
}

pub fn process_recv_packet_execute<Ctx: TokenTransferContext>(
    ctx: &mut Ctx,
    packet: &Packet,
    data: PacketData,
) -> Result<ModuleExtras, (ModuleExtras, TokenTransferError)> {
    if !ctx.is_receive_enabled() {
        return Err((ModuleExtras::empty(), TokenTransferError::ReceiveDisabled));
    }

    let receiver_account = data.receiver.clone().try_into().map_err(|_| {
        (
            ModuleExtras::empty(),
            TokenTransferError::ParseAccountFailure,
        )
    })?;

    let extras = if is_receiver_chain_source(
        packet.port_on_a.clone(),
        packet.chan_on_a.clone(),
        &data.token.denom,
    ) {
        // sender chain is not the source, unescrow tokens
        let prefix = TracePrefix::new(packet.port_on_a.clone(), packet.chan_on_a.clone());
        let coin = {
            let mut c = data.token;
            c.denom.remove_trace_prefix(&prefix);
            c
        };

        let escrow_address = ctx
            .get_channel_escrow_address(&packet.port_on_b, &packet.chan_on_b)
            .map_err(|token_err| (ModuleExtras::empty(), token_err))?;

        ctx.send_coins(&escrow_address, &receiver_account, &coin)
            .map_err(|token_err| (ModuleExtras::empty(), token_err))?;

        ModuleExtras::empty()
    } else {
        // sender chain is the source, mint vouchers
        let prefix = TracePrefix::new(packet.port_on_b.clone(), packet.chan_on_b.clone());
        let coin = {
            let mut c = data.token;
            c.denom.add_trace_prefix(prefix);
            c
        };

        let extras = {
            let denom_trace_event = DenomTraceEvent {
                trace_hash: ctx.denom_hash_string(&coin.denom),
                denom: coin.denom.clone(),
            };
            ModuleExtras {
                events: vec![denom_trace_event.into()],
                log: Vec::new(),
            }
        };
        let extras_closure = extras.clone();

        ctx.mint_coins(&receiver_account, &coin)
            .map_err(|token_err| (extras_closure, token_err))?;

        extras
    };

    Ok(extras)
}
