use ibc::core::ics24_host::identifier::PortId;
use ibc::prelude::*;
use ibc::proto::core::channel::v1::MsgChannelOpenInit as RawMsgChannelOpenInit;

use super::dummy_raw_channel_end;
use crate::utils::dummies::core::signer::dummy_bech32_account;

/// Returns a dummy `RawMsgChannelOpenInit`, for testing only!
pub fn dummy_raw_msg_chan_open_init(counterparty_channel_id: Option<u64>) -> RawMsgChannelOpenInit {
    RawMsgChannelOpenInit {
        port_id: PortId::transfer().to_string(),
        channel: Some(dummy_raw_channel_end(1, counterparty_channel_id)),
        signer: dummy_bech32_account(),
    }
}
