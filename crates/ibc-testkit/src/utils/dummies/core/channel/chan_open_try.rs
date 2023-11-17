use ibc::core::channel::types::proto::v1::MsgChannelOpenTry as RawMsgChannelOpenTry;
use ibc::core::client::types::proto::v1::Height;
use ibc::core::host::identifiers::PortId;
use ibc::core::primitives::prelude::*;

use super::{dummy_proof, dummy_raw_channel_end};
use crate::utils::dummies::core::signer::dummy_bech32_account;

/// Returns a dummy `RawMsgChannelOpenTry`, for testing purposes only!
pub fn dummy_raw_msg_chan_open_try(proof_height: u64) -> RawMsgChannelOpenTry {
    #[allow(deprecated)]
    RawMsgChannelOpenTry {
        port_id: PortId::transfer().to_string(),
        previous_channel_id: "".to_string(),
        channel: Some(dummy_raw_channel_end(2, Some(0))),
        counterparty_version: "".to_string(),
        proof_init: dummy_proof(),
        proof_height: Some(Height {
            revision_number: 0,
            revision_height: proof_height,
        }),
        signer: dummy_bech32_account(),
    }
}
