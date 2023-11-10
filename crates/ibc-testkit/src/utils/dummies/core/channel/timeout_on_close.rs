use ibc::proto::core::channel::v1::MsgTimeoutOnClose as RawMsgTimeoutOnClose;
use ibc::proto::core::client::v1::Height as RawHeight;

use super::dummy_raw_packet;
use crate::utils::dummies::core::signer::{dummy_bech32_account, dummy_proof};

/// Returns a dummy `RawMsgTimeoutOnClose`, for testing only!
/// The `height` parametrizes both the proof height as well as the timeout height.
pub fn dummy_raw_msg_timeout_on_close(height: u64, timeout_timestamp: u64) -> RawMsgTimeoutOnClose {
    RawMsgTimeoutOnClose {
        packet: Some(dummy_raw_packet(height, timeout_timestamp)),
        proof_unreceived: dummy_proof(),
        proof_close: dummy_proof(),
        proof_height: Some(RawHeight {
            revision_number: 0,
            revision_height: height,
        }),
        next_sequence_recv: 1,
        signer: dummy_bech32_account(),
    }
}
