use ibc_eureka_core_channel_types::commitment::compute_packet_commitment;
use ibc_eureka_core_channel_types::error::ChannelError;
use ibc_eureka_core_channel_types::events::SendPacket;
use ibc_eureka_core_channel_types::packet::Packet;
use ibc_eureka_core_client::context::prelude::*;
use ibc_eureka_core_handler_types::events::{IbcEvent, MessageEvent};
use ibc_eureka_core_host::types::path::{
    ClientConsensusStatePath, CommitmentPathV2 as CommitmentPath, SeqSendPathV2 as SeqSendPath,
};
use ibc_primitives::prelude::*;

use crate::context::{SendPacketExecutionContext, SendPacketValidationContext};

/// Send the given packet, including all necessary validation.
///
/// Equivalent to calling [`send_packet_validate`], followed by [`send_packet_execute`]
pub fn send_packet(
    ctx_a: &mut impl SendPacketExecutionContext,
    packet: Packet,
) -> Result<(), ChannelError> {
    send_packet_validate(ctx_a, &packet)?;
    send_packet_execute(ctx_a, packet)
}

/// Validate that sending the given packet would succeed.
pub fn send_packet_validate(
    ctx_a: &impl SendPacketValidationContext,
    packet: &Packet,
) -> Result<(), ChannelError> {
    if !packet.header.timeout_height_on_b.is_set() && !packet.header.timeout_timestamp_on_b.is_set()
    {
        return Err(ChannelError::MissingTimeout);
    }

    let payload = &packet.payloads[0];

    let (source_prefix, _source_port) = &payload.header.source_port;
    let channel_target_client_on_source = &packet.header.target_client_on_source;
    let (target_prefix, _target_port) = &payload.header.target_port;
    let channel_source_client_on_target = &packet.header.source_client_on_target;
    let seq_on_a = &packet.header.seq_on_a;

    let id_target_client_on_source = channel_target_client_on_source.as_ref();

    let client_val_ctx_a = ctx_a.get_client_validation_context();

    let target_client_on_source = client_val_ctx_a.client_state(id_target_client_on_source)?;

    target_client_on_source
        .status(
            ctx_a.get_client_validation_context(),
            id_target_client_on_source,
        )?
        .verify_is_active()?;

    let latest_height_on_target = target_client_on_source.latest_height();

    if packet
        .header
        .timeout_height_on_b
        .has_expired(latest_height_on_target)
    {
        return Err(ChannelError::InsufficientPacketHeight {
            chain_height: latest_height_on_target,
            timeout_height: packet.header.timeout_height_on_b,
        });
    }

    let client_cons_state_path_on_a = ClientConsensusStatePath::new(
        id_target_client_on_source.clone(),
        latest_height_on_target.revision_number(),
        latest_height_on_target.revision_height(),
    );
    let target_consensus_state = client_val_ctx_a.consensus_state(&client_cons_state_path_on_a)?;
    let latest_timestamp = target_consensus_state.timestamp()?;
    let packet_timestamp = packet.header.timeout_timestamp_on_b;
    if packet_timestamp.has_expired(&latest_timestamp) {
        return Err(ChannelError::ExpiredPacketTimestamp);
    }

    // TODO(rano): include full channel identifier in the path
    let seq_send_path_on_a = SeqSendPath::new(
        channel_source_client_on_target.as_ref(),
        &format!("{source_prefix:?}"),
        channel_target_client_on_source.as_ref(),
        &format!("{target_prefix:?}"),
    );
    let next_seq_send_on_a = ctx_a.get_next_sequence_send(&seq_send_path_on_a)?;

    if seq_on_a != &next_seq_send_on_a {
        return Err(ChannelError::MismatchedPacketSequence {
            actual: *seq_on_a,
            expected: next_seq_send_on_a,
        });
    }

    Ok(())
}

/// Send the packet without any validation.
///
/// A prior call to [`send_packet_validate`] MUST have succeeded.
pub fn send_packet_execute(
    ctx_a: &mut impl SendPacketExecutionContext,
    packet: Packet,
) -> Result<(), ChannelError> {
    let payload = &packet.payloads[0];

    let (source_prefix, _source_port) = &payload.header.source_port;
    let channel_target_client_on_source = &packet.header.target_client_on_source;
    let (target_prefix, _target_port) = &payload.header.target_port;
    let channel_source_client_on_target = &packet.header.source_client_on_target;
    let seq_on_a = &packet.header.seq_on_a;
    let data = &payload.data;

    {
        let seq_send_path_on_a = SeqSendPath::new(
            channel_source_client_on_target.as_ref(),
            &format!("{source_prefix:?}"),
            channel_target_client_on_source.as_ref(),
            &format!("{target_prefix:?}"),
        );
        let next_seq_send_on_a = ctx_a.get_next_sequence_send(&seq_send_path_on_a)?;

        ctx_a.store_next_sequence_send(&seq_send_path_on_a, next_seq_send_on_a.increment())?;
    }

    ctx_a.store_packet_commitment(
        &CommitmentPath::new(
            channel_source_client_on_target.as_ref(),
            &format!("{source_prefix:?}"),
            channel_target_client_on_source.as_ref(),
            &format!("{target_prefix:?}"),
            seq_on_a,
        ),
        compute_packet_commitment(
            data,
            &packet.header.timeout_height_on_b,
            &packet.header.timeout_timestamp_on_b,
        ),
    )?;

    // emit events and logs
    {
        ctx_a.log_message("success: packet send".to_string())?;
        let event = IbcEvent::SendPacket(SendPacket::new(packet));
        ctx_a.emit_ibc_event(IbcEvent::Message(MessageEvent::Channel))?;
        ctx_a.emit_ibc_event(event)?;
    }

    Ok(())
}
