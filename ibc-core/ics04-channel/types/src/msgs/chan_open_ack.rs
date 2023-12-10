use ibc_core_client_types::Height;
use ibc_core_commitment_types::commitment::CommitmentProofBytes;
use ibc_core_host_types::identifiers::{ChannelId, PortId};
use ibc_primitives::prelude::*;
use ibc_primitives::Signer;
use ibc_proto::ibc::core::channel::v1::MsgChannelOpenAck as RawMsgChannelOpenAck;
use ibc_proto::Protobuf;

use crate::error::ChannelError;
use crate::Version;

pub const CHAN_OPEN_ACK_TYPE_URL: &str = "/ibc.core.channel.v1.MsgChannelOpenAck";

/// Message definition for the third step in the channel open handshake (`ChanOpenAck` datagram).
///
/// Per our convention, this message is sent to chain A.
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MsgChannelOpenAck {
    pub port_id_on_a: PortId,
    pub chan_id_on_a: ChannelId,
    pub chan_id_on_b: ChannelId,
    pub version_on_b: Version,
    pub proof_chan_end_on_b: CommitmentProofBytes,
    pub proof_height_on_b: Height,
    pub signer: Signer,
}

impl Protobuf<RawMsgChannelOpenAck> for MsgChannelOpenAck {}

impl TryFrom<RawMsgChannelOpenAck> for MsgChannelOpenAck {
    type Error = ChannelError;

    fn try_from(raw_msg: RawMsgChannelOpenAck) -> Result<Self, Self::Error> {
        Ok(MsgChannelOpenAck {
            port_id_on_a: raw_msg.port_id.parse()?,
            chan_id_on_a: raw_msg.channel_id.parse()?,
            chan_id_on_b: raw_msg.counterparty_channel_id.parse()?,
            version_on_b: raw_msg.counterparty_version.into(),
            proof_chan_end_on_b: raw_msg
                .proof_try
                .try_into()
                .map_err(|_| ChannelError::InvalidProof)?,
            proof_height_on_b: raw_msg
                .proof_height
                .and_then(|raw_height| raw_height.try_into().ok())
                .ok_or(ChannelError::MissingHeight)?,
            signer: raw_msg.signer.into(),
        })
    }
}

impl From<MsgChannelOpenAck> for RawMsgChannelOpenAck {
    fn from(domain_msg: MsgChannelOpenAck) -> Self {
        RawMsgChannelOpenAck {
            port_id: domain_msg.port_id_on_a.to_string(),
            channel_id: domain_msg.chan_id_on_a.to_string(),
            counterparty_channel_id: domain_msg.chan_id_on_b.to_string(),
            counterparty_version: domain_msg.version_on_b.to_string(),
            proof_try: domain_msg.proof_chan_end_on_b.into(),
            proof_height: Some(domain_msg.proof_height_on_b.into()),
            signer: domain_msg.signer.to_string(),
        }
    }
}
