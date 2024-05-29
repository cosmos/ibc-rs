use ibc_core_host_types::identifiers::{ChannelId, PortId};
use ibc_primitives::prelude::*;
use ibc_primitives::Signer;
use ibc_proto::ibc::core::channel::v1::MsgChannelCloseInit as RawMsgChannelCloseInit;
use ibc_proto::Protobuf;

use crate::error::ChannelError;

pub const CHAN_CLOSE_INIT_TYPE_URL: &str = "/ibc.core.channel.v1.MsgChannelCloseInit";

///
/// Message definition for the first step in the channel close handshake (`ChanCloseInit` datagram).
/// Per our convention, this message is sent to chain A.
///
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MsgChannelCloseInit {
    pub port_id_on_a: PortId,
    pub chan_id_on_a: ChannelId,
    pub signer: Signer,
}

impl Protobuf<RawMsgChannelCloseInit> for MsgChannelCloseInit {}

impl TryFrom<RawMsgChannelCloseInit> for MsgChannelCloseInit {
    type Error = ChannelError;

    fn try_from(raw_msg: RawMsgChannelCloseInit) -> Result<Self, Self::Error> {
        Ok(MsgChannelCloseInit {
            port_id_on_a: raw_msg.port_id.parse()?,
            chan_id_on_a: raw_msg.channel_id.parse()?,
            signer: raw_msg.signer.into(),
        })
    }
}

impl From<MsgChannelCloseInit> for RawMsgChannelCloseInit {
    fn from(domain_msg: MsgChannelCloseInit) -> Self {
        RawMsgChannelCloseInit {
            port_id: domain_msg.port_id_on_a.to_string(),
            channel_id: domain_msg.chan_id_on_a.to_string(),
            signer: domain_msg.signer.to_string(),
        }
    }
}
