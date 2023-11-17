use ibc::core::channel::types::proto::v1::MsgChannelOpenConfirm as RawMsgChannelOpenConfirm;
use ibc::core::client::types::proto::v1::Height;
use ibc::core::host::identifiers::{ChannelId, PortId};
use ibc::core::primitives::prelude::*;

use super::dummy_proof;
use crate::utils::dummies::core::signer::dummy_bech32_account;

/// Returns a dummy `RawMsgChannelOpenConfirm`, for testing purposes only!
pub fn dummy_raw_msg_chan_open_confirm(proof_height: u64) -> RawMsgChannelOpenConfirm {
    RawMsgChannelOpenConfirm {
        port_id: PortId::transfer().to_string(),
        channel_id: ChannelId::default().to_string(),
        proof_ack: dummy_proof(),
        proof_height: Some(Height {
            revision_number: 0,
            revision_height: proof_height,
        }),
        signer: dummy_bech32_account(),
    }
}
