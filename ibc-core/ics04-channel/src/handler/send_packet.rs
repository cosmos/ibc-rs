use ibc_core_channel_types::channel::Counterparty;
use ibc_core_channel_types::commitment::compute_packet_commitment;
use ibc_core_channel_types::error::PacketError;
use ibc_core_channel_types::events::SendPacket;
use ibc_core_channel_types::packet::Packet;
use ibc_core_client::context::prelude::*;
use ibc_core_handler_types::error::ContextError;
use ibc_core_handler_types::events::{IbcEvent, MessageEvent};
use ibc_core_host::types::path::{
    ChannelEndPath, ClientConsensusStatePath, CommitmentPath, SeqSendPath,
};
use ibc_primitives::prelude::*;
use ibc_primitives::Expiry;

use crate::context::{SendPacketExecutionContext, SendPacketValidationContext};

/// Send the given packet, including all necessary validation.
///
/// Equivalent to calling [`send_packet_validate`], followed by [`send_packet_execute`]
pub fn send_packet(
    ctx_a: &mut impl SendPacketExecutionContext,
    packet: Packet,
) -> Result<(), ContextError> {
    send_packet_validate(ctx_a, &packet)?;
    send_packet_execute(ctx_a, packet)
}

/// Validate that sending the given packet would succeed.
pub fn send_packet_validate(
    ctx_a: &impl SendPacketValidationContext,
    packet: &Packet,
) -> Result<(), ContextError> {
    let chan_end_path_on_a = ChannelEndPath::new(&packet.port_id_on_a, &packet.chan_id_on_a);
    let chan_end_on_a = ctx_a.channel_end(&chan_end_path_on_a)?;

    // Checks the channel end not be `Closed`.
    // This allows for optimistic packet processing before a channel opens
    chan_end_on_a.verify_not_closed()?;

    let counterparty = Counterparty::new(
        packet.port_id_on_b.clone(),
        Some(packet.chan_id_on_b.clone()),
    );

    chan_end_on_a.verify_counterparty_matches(&counterparty)?;

    let conn_id_on_a = &chan_end_on_a.connection_hops()[0];

    let conn_end_on_a = ctx_a.connection_end(conn_id_on_a)?;

    let client_id_on_a = conn_end_on_a.client_id();

    let client_val_ctx_a = ctx_a.get_client_validation_context();

    let client_state_of_b_on_a = client_val_ctx_a.client_state(client_id_on_a)?;

    client_state_of_b_on_a
        .status(ctx_a.get_client_validation_context(), client_id_on_a)?
        .verify_is_active()?;

    let latest_height_on_a = client_state_of_b_on_a.latest_height();

    if packet.timeout_height_on_b.has_expired(latest_height_on_a) {
        return Err(PacketError::LowPacketHeight {
            chain_height: latest_height_on_a,
            timeout_height: packet.timeout_height_on_b,
        }
        .into());
    }

    let client_cons_state_path_on_a = ClientConsensusStatePath::new(
        client_id_on_a.clone(),
        latest_height_on_a.revision_number(),
        latest_height_on_a.revision_height(),
    );
    let consensus_state_of_b_on_a =
        client_val_ctx_a.consensus_state(&client_cons_state_path_on_a)?;
    let latest_timestamp = consensus_state_of_b_on_a.timestamp();
    let packet_timestamp = packet.timeout_timestamp_on_b;
    if let Expiry::Expired = latest_timestamp.check_expiry(&packet_timestamp) {
        return Err(PacketError::LowPacketTimestamp.into());
    }

    let seq_send_path_on_a = SeqSendPath::new(&packet.port_id_on_a, &packet.chan_id_on_a);
    let next_seq_send_on_a = ctx_a.get_next_sequence_send(&seq_send_path_on_a)?;

    if packet.seq_on_a != next_seq_send_on_a {
        return Err(PacketError::InvalidPacketSequence {
            given_sequence: packet.seq_on_a,
            next_sequence: next_seq_send_on_a,
        }
        .into());
    }

    Ok(())
}

/// Send the packet without any validation.
///
/// A prior call to [`send_packet_validate`] MUST have succeeded.
pub fn send_packet_execute(
    ctx_a: &mut impl SendPacketExecutionContext,
    packet: Packet,
) -> Result<(), ContextError> {
    {
        let seq_send_path_on_a = SeqSendPath::new(&packet.port_id_on_a, &packet.chan_id_on_a);
        let next_seq_send_on_a = ctx_a.get_next_sequence_send(&seq_send_path_on_a)?;

        ctx_a.store_next_sequence_send(&seq_send_path_on_a, next_seq_send_on_a.increment())?;
    }

    ctx_a.store_packet_commitment(
        &CommitmentPath::new(&packet.port_id_on_a, &packet.chan_id_on_a, packet.seq_on_a),
        compute_packet_commitment(
            &packet.data,
            &packet.timeout_height_on_b,
            &packet.timeout_timestamp_on_b,
        ),
    )?;

    // emit events and logs
    {
        let chan_end_path_on_a = ChannelEndPath::new(&packet.port_id_on_a, &packet.chan_id_on_a);
        let chan_end_on_a = ctx_a.channel_end(&chan_end_path_on_a)?;
        let conn_id_on_a = &chan_end_on_a.connection_hops()[0];

        ctx_a.log_message("success: packet send".to_string())?;
        let event = IbcEvent::SendPacket(SendPacket::new(
            packet,
            chan_end_on_a.ordering,
            conn_id_on_a.clone(),
        ));
        ctx_a.emit_ibc_event(IbcEvent::Message(MessageEvent::Channel))?;
        ctx_a.emit_ibc_event(event)?;
    }

    Ok(())
}
