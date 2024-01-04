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
pub fn process_recv_packet_execute<Ctx>(
    ctx_b: &mut Ctx,
    packet: &Packet,
    data: PacketData,
) -> Result<ModuleExtras, Box<(ModuleExtras, NftTransferError)>>
where
    Ctx: NftTransferExecutionContext,
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

        // Note: the validation is called before the execution.
        // Refer to ICS-20 `process_recv_packet_execute()`.
        for token_id in data.token_ids.as_ref() {
            ctx_b
                .unescrow_nft_validate(
                    &receiver_account,
                    &packet.port_id_on_b,
                    &packet.chan_id_on_b,
                    &class_id,
                    token_id,
                )
                .map_err(|nft_error| (ModuleExtras::empty(), nft_error))?;
            ctx_b
                .unescrow_nft_execute(
                    &receiver_account,
                    &packet.port_id_on_b,
                    &packet.chan_id_on_b,
                    &class_id,
                    token_id,
                )
                .map_err(|nft_error| (ModuleExtras::empty(), nft_error))?;
        }

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

        for ((token_id, token_uri), token_data) in data
            .token_ids
            .as_ref()
            .iter()
            .zip(data.token_uris.iter())
            .zip(data.token_data.iter())
        {
            // Note: the validation is called before the execution.
            // Refer to ICS-20 `process_recv_packet_execute()`.

            let class_uri = data
                .class_uri
                .as_ref()
                .ok_or((ModuleExtras::empty(), NftTransferError::NftClassNotFound))?;
            let class_data = data
                .class_data
                .as_ref()
                .ok_or((ModuleExtras::empty(), NftTransferError::NftClassNotFound))?;
            ctx_b
                .create_or_update_class_validate(&class_id, class_uri, class_data)
                .map_err(|nft_error| (ModuleExtras::empty(), nft_error))?;
            ctx_b
                .create_or_update_class_execute(&class_id, class_uri, class_data)
                .map_err(|nft_error| (ModuleExtras::empty(), nft_error))?;

            ctx_b
                .mint_nft_validate(
                    &receiver_account,
                    &class_id,
                    token_id,
                    token_uri,
                    token_data,
                )
                .map_err(|nft_error| (extras.clone(), nft_error))?;
            ctx_b
                .mint_nft_execute(
                    &receiver_account,
                    &class_id,
                    token_id,
                    token_uri,
                    token_data,
                )
                .map_err(|nft_error| (extras.clone(), nft_error))?;
        }

        extras
    };

    Ok(extras)
}
