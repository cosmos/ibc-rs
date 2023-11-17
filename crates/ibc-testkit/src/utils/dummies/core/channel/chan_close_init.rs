use ibc::core::channel::types::proto::v1::MsgChannelCloseInit as RawMsgChannelCloseInit;
use ibc::core::host::identifiers::{ChannelId, PortId};
use ibc::core::primitives::prelude::*;

use crate::utils::dummies::core::signer::dummy_bech32_account;

/// Returns a dummy `RawMsgChannelCloseInit`, for testing purposes only!
pub fn dummy_raw_msg_chan_close_init() -> RawMsgChannelCloseInit {
    RawMsgChannelCloseInit {
        port_id: PortId::transfer().to_string(),
        channel_id: ChannelId::default().to_string(),
        signer: dummy_bech32_account(),
    }
}
