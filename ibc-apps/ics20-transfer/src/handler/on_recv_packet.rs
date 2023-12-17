use ibc_app_transfer_types::error::TokenTransferError;
use ibc_app_transfer_types::events::DenomTraceEvent;
use ibc_app_transfer_types::packet::PacketData;
use ibc_app_transfer_types::{is_receiver_chain_source, TracePrefix};
use ibc_core::channel::types::packet::Packet;
use ibc_core::primitives::prelude::*;
use ibc_core::router::types::module::ModuleExtras;

use crate::context::TokenTransferExecutionContext;

/// This function handles the transfer receiving logic.
///
/// Note that `send/mint_coins_validate` steps are performed on the host chain
/// to validate accounts and token info. But the result is then used for
/// execution on the IBC side, including storing acknowledgements and emitting
/// events.
pub fn process_recv_packet_execute<Ctx: TokenTransferExecutionContext>(
    ctx_b: &mut Ctx,
    packet: &Packet,
    data: PacketData,
) -> Result<ModuleExtras, (ModuleExtras, TokenTransferError)> {
    ctx_b
        .can_receive_coins()
        .map_err(|err| (ModuleExtras::empty(), err))?;

    let receiver_account = data.receiver.clone().try_into().map_err(|_| {
        (
            ModuleExtras::empty(),
            TokenTransferError::ParseAccountFailure,
        )
    })?;

    let extras = if is_receiver_chain_source(
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

        // Note: it is correct to do the validation here because `recv_packet()`
        // works slightly differently. We do not have a
        // `on_recv_packet_validate()` callback because regardless of whether or
        // not the app succeeds to receive the packet, we want to run the
        // `execute()` phase. And this is because the app failing to receive
        // does not constitute a failure of the message processing.
        // Specifically, when the app fails to receive, we need to return
        // a `TokenTransferAcknowledgement::Error` acknowledgement, which
        // gets relayed back to the sender so that the escrowed tokens
        // can be refunded.
        ctx_b
            .unescrow_coins_validate(
                &receiver_account,
                &packet.port_id_on_b,
                &packet.chan_id_on_b,
                &coin,
            )
            .map_err(|token_err| (ModuleExtras::empty(), token_err))?;
        ctx_b
            .unescrow_coins_execute(
                &receiver_account,
                &packet.port_id_on_b,
                &packet.chan_id_on_b,
                &coin,
            )
            .map_err(|token_err| (ModuleExtras::empty(), token_err))?;

        ModuleExtras::empty()
    } else {
        // sender chain is the source, mint vouchers
        let prefix = TracePrefix::new(packet.port_id_on_b.clone(), packet.chan_id_on_b.clone());
        let coin = {
            let mut c = data.token;
            c.denom.add_trace_prefix(prefix);
            c
        };

        let extras = {
            let denom_trace_event = DenomTraceEvent {
                trace_hash: ctx_b.denom_hash_string(&coin.denom),
                denom: coin.denom.clone(),
            };
            ModuleExtras {
                events: vec![denom_trace_event.into()],
                log: Vec::new(),
            }
        };

        // Note: it is correct to do the validation here because `recv_packet()`
        // works slightly differently. We do not have a
        // `on_recv_packet_validate()` callback because regardless of whether or
        // not the app succeeds to receive the packet, we want to run the
        // `execute()` phase. And this is because the app failing to receive
        // does not constitute a failure of the message processing.
        // Specifically, when the app fails to receive, we need to return
        // a `TokenTransferAcknowledgement::Error` acknowledgement, which
        // gets relayed back to the sender so that the escrowed tokens
        // can be refunded.
        ctx_b
            .mint_coins_validate(&receiver_account, &coin)
            .map_err(|token_err| (extras.clone(), token_err))?;

        ctx_b
            .mint_coins_execute(&receiver_account, &coin)
            .map_err(|token_err| (extras.clone(), token_err))?;

        extras
    };

    Ok(extras)
}
