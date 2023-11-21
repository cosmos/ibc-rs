use core::ops::Add;
use core::time::Duration;

use ibc::core::channel::types::msgs::MsgRecvPacket;
use ibc::core::channel::types::packet::Packet;
use ibc::core::channel::types::proto::v1::MsgRecvPacket as RawMsgRecvPacket;
use ibc::core::client::types::proto::v1::Height as RawHeight;
use ibc::core::client::types::Height;
use ibc::core::commitment_types::commitment::CommitmentProofBytes;
use ibc::core::primitives::{Signer, Timestamp};

use super::{dummy_proof, dummy_raw_packet};
use crate::utils::dummies::core::signer::dummy_bech32_account;

pub fn dummy_msg_recv_packet(
    packet: Packet,
    proof_commitment_on_a: CommitmentProofBytes,
    proof_height_on_a: Height,
    signer: Signer,
) -> MsgRecvPacket {
    MsgRecvPacket {
        packet,
        proof_commitment_on_a,
        proof_height_on_a,
        signer,
    }
}

/// Returns a dummy `RawMsgRecvPacket`, for testing purposes only! The `height`
/// parametrizes both the proof height as well as the timeout height.
pub fn dummy_raw_msg_recv_packet(height: u64) -> RawMsgRecvPacket {
    let timestamp = Timestamp::now().add(Duration::from_secs(9));
    RawMsgRecvPacket {
        packet: Some(dummy_raw_packet(
            height,
            timestamp.expect("timestamp").nanoseconds(),
        )),
        proof_commitment: dummy_proof(),
        proof_height: Some(RawHeight {
            revision_number: 0,
            revision_height: height,
        }),
        signer: dummy_bech32_account(),
    }
}
