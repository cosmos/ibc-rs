use ibc_core_client_types::Height;
use ibc_core_commitment_types::commitment::CommitmentProofBytes;
use ibc_core_host_types::identifiers::{ChannelId, PortId};
use ibc_primitives::prelude::*;
use ibc_primitives::Signer;
use ibc_proto::ibc::core::channel::v1::MsgChannelOpenConfirm as RawMsgChannelOpenConfirm;
use ibc_proto::Protobuf;

use crate::error::ChannelError;

pub const CHAN_OPEN_CONFIRM_TYPE_URL: &str = "/ibc.core.channel.v1.MsgChannelOpenConfirm";

///
/// Message definition for the fourth step in the channel open handshake (`ChanOpenConfirm`
/// datagram).
/// Per our convention, this message is sent to chain B.
///
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MsgChannelOpenConfirm {
    pub port_id_on_b: PortId,
    pub chan_id_on_b: ChannelId,
    pub proof_chan_end_on_a: CommitmentProofBytes,
    pub proof_height_on_a: Height,
    pub signer: Signer,
}

impl Protobuf<RawMsgChannelOpenConfirm> for MsgChannelOpenConfirm {}

impl TryFrom<RawMsgChannelOpenConfirm> for MsgChannelOpenConfirm {
    type Error = ChannelError;

    fn try_from(raw_msg: RawMsgChannelOpenConfirm) -> Result<Self, Self::Error> {
        Ok(MsgChannelOpenConfirm {
            port_id_on_b: raw_msg.port_id.parse()?,
            chan_id_on_b: raw_msg.channel_id.parse()?,
            proof_chan_end_on_a: raw_msg
                .proof_ack
                .try_into()
                .map_err(|_| ChannelError::InvalidProof)?,
            proof_height_on_a: raw_msg
                .proof_height
                .and_then(|raw_height| raw_height.try_into().ok())
                .ok_or(ChannelError::MissingHeight)?,
            signer: raw_msg.signer.into(),
        })
    }
}

impl From<MsgChannelOpenConfirm> for RawMsgChannelOpenConfirm {
    fn from(domain_msg: MsgChannelOpenConfirm) -> Self {
        RawMsgChannelOpenConfirm {
            port_id: domain_msg.port_id_on_b.to_string(),
            channel_id: domain_msg.chan_id_on_b.to_string(),
            proof_ack: domain_msg.proof_chan_end_on_a.into(),
            proof_height: Some(domain_msg.proof_height_on_a.into()),
            signer: domain_msg.signer.to_string(),
        }
    }
}
