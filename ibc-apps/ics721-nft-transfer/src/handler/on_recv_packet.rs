use ibc_core::channel::types::packet::Packet;
use ibc_core::primitives::prelude::*;
use ibc_core::router::types::module::ModuleExtras;

use crate::context::NftTransferExecutionContext;
use crate::types::error::NftTransferError;
use crate::types::events::ClassTraceEvent;
use crate::types::packet::PacketData;
use crate::types::{is_receiver_chain_source, TracePrefix};

/// This function handles the transfer receiving logic.
///
/// Note that `send/mint_nft_validate` steps are performed on the host chain
/// to validate accounts and NFT info. But the result is then used for execution
/// on the IBC side, including storing acknowledgements and emitting events.
pub fn process_recv_packet_execute<Ctx, N, C>(
    ctx_b: &mut Ctx,
    packet: &Packet,
    data: PacketData,
) -> Result<ModuleExtras, Box<(ModuleExtras, NftTransferError)>>
where
    Ctx: NftTransferExecutionContext<N, C>,
{
    ctx_b
        .can_receive_nft()
        .map_err(|err| (ModuleExtras::empty(), err))?;

    let receiver_account = data
        .receiver
        .clone()
        .try_into()
        .map_err(|_| (ModuleExtras::empty(), NftTransferError::ParseAccountFailure))?;

    let extras = if is_receiver_chain_source(
        packet.port_id_on_a.clone(),
        packet.chan_id_on_a.clone(),
        &data.class_id,
    ) {
        // sender chain is not the source, unescrow the NFT
        let prefix = TracePrefix::new(packet.port_id_on_a.clone(), packet.chan_id_on_a.clone());
        let class_id = {
            let mut c = data.class_id;
            c.remove_trace_prefix(&prefix);
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
        data.token_ids
            .as_ref()
            .iter()
            .try_for_each(|token_id| {
                ctx_b.unescrow_nft_validate(
                    &receiver_account,
                    &packet.port_id_on_b,
                    &packet.chan_id_on_b,
                    &class_id,
                    token_id,
                )
            })
            .map_err(|nft_error| (ModuleExtras::empty(), nft_error))?;
        data.token_ids
            .as_ref()
            .iter()
            .try_for_each(|token_id| {
                ctx_b.unescrow_nft_execute(
                    &receiver_account,
                    &packet.port_id_on_b,
                    &packet.chan_id_on_b,
                    &class_id,
                    token_id,
                )
            })
            .map_err(|nft_error| (ModuleExtras::empty(), nft_error))?;

        ModuleExtras::empty()
    } else {
        // sender chain is the source, mint vouchers
        let prefix = TracePrefix::new(packet.port_id_on_b.clone(), packet.chan_id_on_b.clone());
        let class_id = {
            let mut c = data.class_id;
            c.add_trace_prefix(prefix);
            c
        };

        let extras = {
            let class_trace_event = ClassTraceEvent {
                trace_hash: ctx_b.class_hash_string(&class_id),
                class: class_id.clone(),
            };
            ModuleExtras {
                events: vec![class_trace_event.into()],
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
        data.token_ids
            .0
            .iter()
            .zip(data.token_uris.iter())
            .zip(data.token_data.iter())
            .try_for_each(|((token_id, token_uri), token_data)| {
                ctx_b
                    .mint_nft_validate(
                        &receiver_account,
                        &class_id,
                        token_id,
                        token_uri,
                        token_data,
                    )
                    .and(ctx_b.mint_nft_execute(
                        &receiver_account,
                        &class_id,
                        token_id,
                        token_uri,
                        token_data,
                    ))
            })
            .map_err(|nft_error| (extras.clone(), nft_error))?;

        extras
    };

    Ok(extras)
}
