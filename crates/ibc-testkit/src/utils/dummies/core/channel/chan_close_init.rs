use ibc::core::ics24_host::identifier::{ChannelId, PortId};
use ibc::prelude::*;
use ibc::proto::core::channel::v1::MsgChannelCloseInit as RawMsgChannelCloseInit;

use crate::utils::dummies::core::signer::dummy_bech32_account;

/// Returns a dummy `RawMsgChannelCloseInit`, for testing only!
pub fn dummy_raw_msg_chan_close_init() -> RawMsgChannelCloseInit {
    RawMsgChannelCloseInit {
        port_id: PortId::transfer().to_string(),
        channel_id: ChannelId::default().to_string(),
        signer: dummy_bech32_account(),
    }
}
