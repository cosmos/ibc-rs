use ibc::proto::core::channel::v1::{
    MsgAcknowledgement as RawMsgAcknowledgement, Packet as RawPacket,
};
use ibc::proto::core::client::v1::Height as RawHeight;

use super::{dummy_proof, dummy_raw_packet};
use crate::utils::dummies::core::signer::dummy_bech32_account;

/// Returns a dummy `RawMsgAcknowledgement`, for testing purposes only!
/// The `height` parametrizes both the proof height as well as the timeout height.
pub fn dummy_raw_msg_acknowledgement(height: u64) -> RawMsgAcknowledgement {
    dummy_raw_msg_ack_with_packet(dummy_raw_packet(height, 1), height)
}

pub fn dummy_raw_msg_ack_with_packet(packet: RawPacket, height: u64) -> RawMsgAcknowledgement {
    RawMsgAcknowledgement {
        packet: Some(packet),
        acknowledgement: dummy_proof(),
        proof_acked: dummy_proof(),
        proof_height: Some(RawHeight {
            revision_number: 0,
            revision_height: height,
        }),
        signer: dummy_bech32_account(),
    }
}
