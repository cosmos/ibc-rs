use ibc::core::ics24_host::identifier::{ChannelId, PortId};
use ibc::prelude::*;
use ibc::proto::core::channel::v1::MsgChannelOpenAck as RawMsgChannelOpenAck;
use ibc::proto::core::client::v1::Height;

use crate::utils::dummies::core::signer::{dummy_bech32_account, dummy_proof};

/// Returns a dummy `RawMsgChannelOpenAck`, for testing only!
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
