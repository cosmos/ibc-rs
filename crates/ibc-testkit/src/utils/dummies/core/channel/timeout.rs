use ibc::proto::core::channel::v1::MsgTimeout as RawMsgTimeout;
use ibc::proto::core::client::v1::Height as RawHeight;

use super::{dummy_proof, dummy_raw_packet};
use crate::utils::dummies::core::signer::dummy_bech32_account;

/// Returns a dummy `RawMsgTimeout`, for testing purposes only!
/// The `height` parametrizes both the proof height as well as the timeout height.
pub fn dummy_raw_msg_timeout(
    proof_height: u64,
    timeout_height: u64,
    timeout_timestamp: u64,
) -> RawMsgTimeout {
    RawMsgTimeout {
        packet: Some(dummy_raw_packet(timeout_height, timeout_timestamp)),
        proof_unreceived: dummy_proof(),
        proof_height: Some(RawHeight {
            revision_number: 0,
            revision_height: proof_height,
        }),
        next_sequence_recv: 1,
        signer: dummy_bech32_account(),
    }
}
