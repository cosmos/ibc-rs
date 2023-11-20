use ibc::core::channel::types::proto::v1::MsgChannelOpenAck as RawMsgChannelOpenAck;
use ibc::core::client::types::proto::v1::Height;
use ibc::core::host::types::identifiers::{ChannelId, PortId};
use ibc::core::primitives::prelude::*;

use super::dummy_proof;
use crate::utils::dummies::core::signer::dummy_bech32_account;

/// Returns a dummy `RawMsgChannelOpenAck`, for testing purposes only!
pub fn dummy_raw_msg_chan_open_ack(proof_height: u64) -> RawMsgChannelOpenAck {
    RawMsgChannelOpenAck {
        port_id: PortId::transfer().to_string(),
        channel_id: ChannelId::default().to_string(),
        counterparty_channel_id: ChannelId::default().to_string(),
        counterparty_version: "".to_string(),
        proof_try: dummy_proof(),
        proof_height: Some(Height {
            revision_number: 0,
            revision_height: proof_height,
        }),
        signer: dummy_bech32_account(),
    }
}
