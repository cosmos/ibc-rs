use ibc_proto::ibc::core::channel::v1::MsgTimeout as RawMsgTimeout;
use ibc_proto::Protobuf;

use crate::core::ics04_channel::error::PacketError;
use crate::core::ics04_channel::packet::{Packet, Sequence};
use crate::core::ics23_commitment::commitment::CommitmentProofBytes;
use crate::core::Msg;
use crate::prelude::*;
use crate::signer::Signer;
use crate::Height;

pub(crate) const TYPE_URL: &str = "/ibc.core.channel.v1.MsgTimeout";

///
/// Message definition for packet timeout domain type,
/// which is sent on chain A and needs to prove that a previously sent packet was not received on chain B
///
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

impl Msg for MsgTimeout {
    type Raw = RawMsgTimeout;

    fn type_url(&self) -> String {
        TYPE_URL.to_string()
    }
}

impl Protobuf<RawMsgTimeout> for MsgTimeout {}

impl TryFrom<RawMsgTimeout> for MsgTimeout {
    type Error = PacketError;

    fn try_from(raw_msg: RawMsgTimeout) -> Result<Self, Self::Error> {
        if raw_msg.next_sequence_recv == 0 {
            return Err(PacketError::ZeroPacketSequence);
        }
        Ok(MsgTimeout {
            packet: raw_msg
                .packet
                .ok_or(PacketError::MissingPacket)?
                .try_into()?,
            next_seq_recv_on_b: Sequence::from(raw_msg.next_sequence_recv),
            proof_unreceived_on_b: raw_msg
                .proof_unreceived
                .try_into()
                .map_err(|_| PacketError::InvalidProof)?,
            proof_height_on_b: raw_msg
                .proof_height
                .and_then(|raw_height| raw_height.try_into().ok())
                .ok_or(PacketError::MissingHeight)?,
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

#[cfg(test)]
mod test {
    use ibc_proto::ibc::core::channel::v1::MsgTimeout as RawMsgTimeout;
    use ibc_testkit::utils::dummies::core::channel::dummy_raw_msg_timeout;
    use ibc_testkit::utils::dummies::core::signer::dummy_bech32_account;
    use test_log::test;

    use crate::core::ics04_channel::error::PacketError;
    use crate::core::ics04_channel::msgs::timeout::MsgTimeout;
    use crate::prelude::*;

    #[test]
    fn msg_timeout_try_from_raw() {
        struct Test {
            name: String,
            raw: RawMsgTimeout,
            want_pass: bool,
        }

        let proof_height = 50;
        let timeout_height = proof_height;
        let timeout_timestamp = 0;
        let default_raw_msg =
            dummy_raw_msg_timeout(proof_height, timeout_height, timeout_timestamp);

        let tests: Vec<Test> = vec![
            Test {
                name: "Good parameters".to_string(),
                raw: default_raw_msg.clone(),
                want_pass: true,
            },
            Test {
                name: "Missing packet".to_string(),
                raw: RawMsgTimeout {
                    packet: None,
                    ..default_raw_msg.clone()
                },
                want_pass: false,
            },
            Test {
                name: "Missing proof".to_string(),
                raw: RawMsgTimeout {
                    proof_unreceived: Vec::new(),
                    ..default_raw_msg.clone()
                },
                want_pass: false,
            },
            Test {
                name: "Missing proof height".to_string(),
                raw: RawMsgTimeout {
                    proof_height: None,
                    ..default_raw_msg.clone()
                },
                want_pass: false,
            },
            Test {
                name: "Empty signer".to_string(),
                raw: RawMsgTimeout {
                    signer: dummy_bech32_account(),
                    ..default_raw_msg
                },
                want_pass: true,
            },
        ];

        for test in tests {
            let res_msg: Result<MsgTimeout, PacketError> = test.raw.clone().try_into();

            assert_eq!(
                res_msg.is_ok(),
                test.want_pass,
                "MsgTimeout::try_from failed for test {} \nraw message: {:?} with error: {:?}",
                test.name,
                test.raw,
                res_msg.err()
            );
        }
    }

    #[test]
    fn to_and_from() {
        let dummy_raw_msg_timeout = dummy_raw_msg_timeout(15, 20, 0);
        let raw = dummy_raw_msg_timeout;
        let msg = MsgTimeout::try_from(raw.clone()).unwrap();
        let raw_back = RawMsgTimeout::from(msg.clone());
        let msg_back = MsgTimeout::try_from(raw_back.clone()).unwrap();
        assert_eq!(raw, raw_back);
        assert_eq!(msg, msg_back);
    }
}
