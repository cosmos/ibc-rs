use crate::applications::transfer::context::TokenTransferContext;
use crate::applications::transfer::error::Error as Ics20Error;
use crate::applications::transfer::packet::PacketData;
use crate::applications::transfer::relay::refund_packet_token;
use crate::core::ics04_channel::packet::Packet;

pub fn process_timeout_packet(
    ctx: &mut impl TokenTransferContext,
    packet: &Packet,
    data: &PacketData,
) -> Result<(), Ics20Error> {
    refund_packet_token(ctx, packet, data)
}
