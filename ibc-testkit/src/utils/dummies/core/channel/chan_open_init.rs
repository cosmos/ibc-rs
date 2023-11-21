use ibc::core::channel::types::proto::v1::MsgChannelOpenInit as RawMsgChannelOpenInit;
use ibc::core::host::types::identifiers::PortId;
use ibc::core::primitives::prelude::*;

use super::dummy_raw_channel_end;
use crate::utils::dummies::core::signer::dummy_bech32_account;

/// Returns a dummy `RawMsgChannelOpenInit`, for testing purposes only!
pub fn dummy_raw_msg_chan_open_init(counterparty_channel_id: Option<u64>) -> RawMsgChannelOpenInit {
    RawMsgChannelOpenInit {
        port_id: PortId::transfer().to_string(),
        channel: Some(dummy_raw_channel_end(1, counterparty_channel_id)),
        signer: dummy_bech32_account(),
    }
}
