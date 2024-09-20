use ibc_core_client_types::Height;
use ibc_core_commitment_types::commitment::CommitmentProofBytes;
use ibc_core_host_types::error::DecodingError;
use ibc_primitives::prelude::*;
use ibc_primitives::Signer;
use ibc_proto::ibc::core::channel::v1::MsgAcknowledgement as RawMsgAcknowledgement;
use ibc_proto::Protobuf;

use crate::acknowledgement::Acknowledgement;
use crate::packet::Packet;

pub const ACKNOWLEDGEMENT_TYPE_URL: &str = "/ibc.core.channel.v1.MsgAcknowledgement";

///
/// Message definition for packet acknowledgements.
///
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MsgAcknowledgement {
    pub packet: Packet,
    pub acknowledgement: Acknowledgement,
    /// Proof of packet acknowledgement on the receiving chain
    pub proof_acked_on_b: CommitmentProofBytes,
    /// Height at which the commitment proof in this message was taken
    pub proof_height_on_b: Height,
    pub signer: Signer,
}

impl Protobuf<RawMsgAcknowledgement> for MsgAcknowledgement {}

impl TryFrom<RawMsgAcknowledgement> for MsgAcknowledgement {
    type Error = DecodingError;

    fn try_from(raw_msg: RawMsgAcknowledgement) -> Result<Self, Self::Error> {
        Ok(MsgAcknowledgement {
            packet: raw_msg
                .packet
                .ok_or(DecodingError::missing_raw_data(
                    "msg acknowledgement packet data",
                ))?
                .try_into()?,
            acknowledgement: raw_msg.acknowledgement.try_into()?,
            proof_acked_on_b: raw_msg.proof_acked.try_into()?,
            proof_height_on_b: raw_msg
                .proof_height
                .and_then(|raw_height| raw_height.try_into().ok())
                .ok_or(DecodingError::invalid_raw_data(
                    "msg acknowledgement proof height",
                ))?,
            signer: raw_msg.signer.into(),
        })
    }
}

impl From<MsgAcknowledgement> for RawMsgAcknowledgement {
    fn from(domain_msg: MsgAcknowledgement) -> Self {
        RawMsgAcknowledgement {
            packet: Some(domain_msg.packet.into()),
            acknowledgement: domain_msg.acknowledgement.into(),
            signer: domain_msg.signer.to_string(),
            proof_height: Some(domain_msg.proof_height_on_b.into()),
            proof_acked: domain_msg.proof_acked_on_b.into(),
        }
    }
}
