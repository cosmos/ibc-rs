use ibc::core::ics24_host::identifier::{ChannelId, PortId};
use ibc::prelude::*;
use ibc::proto::core::channel::v1::MsgChannelCloseConfirm as RawMsgChannelCloseConfirm;
use ibc::proto::core::client::v1::Height;

use super::dummy_proof;
use crate::utils::dummies::core::signer::dummy_bech32_account;

/// Returns a dummy `RawMsgChannelCloseConfirm`, for testing purposes only!
pub fn dummy_raw_msg_chan_close_confirm(proof_height: u64) -> RawMsgChannelCloseConfirm {
    RawMsgChannelCloseConfirm {
        port_id: PortId::transfer().to_string(),
        channel_id: ChannelId::default().to_string(),
        proof_init: dummy_proof(),
        proof_height: Some(Height {
            revision_number: 0,
            revision_height: proof_height,
        }),
        signer: dummy_bech32_account(),
    }
}
