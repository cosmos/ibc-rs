use ibc_core_client_types::Height;
use ibc_core_commitment_types::commitment::CommitmentProofBytes;
use ibc_core_host_types::error::DecodingError;
use ibc_core_host_types::identifiers::Sequence;
use ibc_primitives::prelude::*;
use ibc_primitives::Signer;
use ibc_proto::ibc::core::channel::v1::MsgTimeout as RawMsgTimeout;
use ibc_proto::Protobuf;

use crate::packet::Packet;

pub const TIMEOUT_TYPE_URL: &str = "/ibc.core.channel.v1.MsgTimeout";

///
/// Message definition for packet timeout domain type,
/// which is sent on chain A and needs to prove that a previously sent packet was not received on chain B
///
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MsgTimeout {
    pub packet: Packet,
    pub next_seq_recv_on_b: Sequence,
    pub proof_unreceived_on_b: CommitmentProofBytes,
    pub proof_height_on_b: Height,
    pub signer: Signer,
}

impl Protobuf<RawMsgTimeout> for MsgTimeout {}

impl TryFrom<RawMsgTimeout> for MsgTimeout {
    type Error = DecodingError;

    fn try_from(raw_msg: RawMsgTimeout) -> Result<Self, Self::Error> {
        if raw_msg.next_sequence_recv == 0 {
            return Err(DecodingError::invalid_raw_data(
                "msg timeout packet sequence cannot be 0",
            ));
        }
        Ok(MsgTimeout {
            packet: raw_msg
                .packet
                .ok_or(DecodingError::missing_raw_data("msg timeout packet data"))?
                .try_into()?,
            next_seq_recv_on_b: Sequence::from(raw_msg.next_sequence_recv),
            proof_unreceived_on_b: raw_msg.proof_unreceived.try_into()?,
            proof_height_on_b: raw_msg
                .proof_height
                .and_then(|raw_height| raw_height.try_into().ok())
                .ok_or(DecodingError::missing_raw_data("msg timeout proof height"))?,
            signer: raw_msg.signer.into(),
        })
    }
}

impl From<MsgTimeout> for RawMsgTimeout {
    fn from(domain_msg: MsgTimeout) -> Self {
        RawMsgTimeout {
            packet: Some(domain_msg.packet.into()),
            proof_unreceived: domain_msg.proof_unreceived_on_b.into(),
            proof_height: Some(domain_msg.proof_height_on_b.into()),
            next_sequence_recv: domain_msg.next_seq_recv_on_b.into(),
            signer: domain_msg.signer.to_string(),
        }
    }
}
