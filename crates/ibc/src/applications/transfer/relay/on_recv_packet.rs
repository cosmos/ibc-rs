use crate::applications::transfer::context::TokenTransferContext;
use crate::applications::transfer::error::TokenTransferError;
use crate::applications::transfer::events::DenomTraceEvent;
use crate::applications::transfer::packet::PacketData;
use crate::applications::transfer::{is_receiver_chain_source, TracePrefix};
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

#[cfg(feature = "val_exec_ctx")]
pub use val_exec_ctx::*;
#[cfg(feature = "val_exec_ctx")]
mod val_exec_ctx {
    pub use super::*;
    use crate::core::ics04_channel::handler::ModuleExtras;

    ////////////////////////////////////////////////////////////
    // FIXME BEFORE MERGE: events need to be emitted even when there's an error
    ////////////////////////////////////////////////////////////
    pub fn process_recv_packet_execute<Ctx: TokenTransferContext>(
        ctx: &mut Ctx,
        packet: &Packet,
        data: PacketData,
    ) -> Result<ModuleExtras, TokenTransferError> {
        if !ctx.is_receive_enabled() {
            return Err(TokenTransferError::ReceiveDisabled);
        }

        let receiver_account = data
            .receiver
            .clone()
            .try_into()
            .map_err(|_| TokenTransferError::ParseAccountFailure)?;

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

            let escrow_address =
                ctx.get_channel_escrow_address(&packet.port_on_b, &packet.chan_on_b)?;

            ctx.send_coins(&escrow_address, &receiver_account, &coin)?;

            ModuleExtras::empty()
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

            ctx.mint_coins(&receiver_account, &coin)?;

            ModuleExtras {
                events: vec![denom_trace_event.into()],
                log: Vec::new(),
            }
        };

        Ok(extras)
    }
}
