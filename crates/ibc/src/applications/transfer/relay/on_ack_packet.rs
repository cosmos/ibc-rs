use crate::applications::transfer::acknowledgement::TokenTransferAcknowledgement;
use crate::applications::transfer::context::TokenTransferContext;
use crate::applications::transfer::error::TokenTransferError;
use crate::applications::transfer::packet::PacketData;
use crate::applications::transfer::relay::refund_packet_token;
use crate::core::ics04_channel::packet::Packet;

pub fn process_ack_packet(
    ctx: &mut impl TokenTransferContext,
    packet: &Packet,
    data: &PacketData,
    ack: &TokenTransferAcknowledgement,
) -> Result<(), TokenTransferError> {
    if matches!(ack, TokenTransferAcknowledgement::Error(_)) {
        refund_packet_token(ctx, packet, data)?;
    }

    Ok(())
}
