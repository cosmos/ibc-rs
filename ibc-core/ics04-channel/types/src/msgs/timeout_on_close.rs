use ibc_core_client_types::Height;
use ibc_core_commitment_types::commitment::CommitmentProofBytes;
use ibc_core_host_types::identifiers::Sequence;
use ibc_primitives::prelude::*;
use ibc_primitives::{Msg, Signer};
use ibc_proto::ibc::core::channel::v1::MsgTimeoutOnClose as RawMsgTimeoutOnClose;
use ibc_proto::Protobuf;

use crate::error::PacketError;
use crate::packet::Packet;

pub const TIMEOUT_ON_CLOSE_TYPE_URL: &str = "/ibc.core.channel.v1.MsgTimeoutOnClose";

///
/// Message definition for packet timeout domain type.
///
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MsgTimeoutOnClose {
    pub packet: Packet,
    pub next_seq_recv_on_b: Sequence,
    pub proof_unreceived_on_b: CommitmentProofBytes,
    pub proof_close_on_b: CommitmentProofBytes,
    pub proof_height_on_b: Height,
    pub signer: Signer,
}

impl Msg for MsgTimeoutOnClose {
    type Raw = RawMsgTimeoutOnClose;

    fn type_url(&self) -> String {
        TIMEOUT_ON_CLOSE_TYPE_URL.to_string()
    }
}

impl Protobuf<RawMsgTimeoutOnClose> for MsgTimeoutOnClose {}

impl TryFrom<RawMsgTimeoutOnClose> for MsgTimeoutOnClose {
    type Error = PacketError;

    fn try_from(raw_msg: RawMsgTimeoutOnClose) -> Result<Self, Self::Error> {
        if raw_msg.next_sequence_recv == 0 {
            return Err(PacketError::ZeroPacketSequence);
        }

        Ok(MsgTimeoutOnClose {
            packet: raw_msg
                .packet
                .ok_or(PacketError::MissingPacket)?
                .try_into()?,
            next_seq_recv_on_b: Sequence::from(raw_msg.next_sequence_recv),
            proof_unreceived_on_b: raw_msg
                .proof_unreceived
                .try_into()
                .map_err(|_| PacketError::InvalidProof)?,
            proof_close_on_b: raw_msg
                .proof_close
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

impl From<MsgTimeoutOnClose> for RawMsgTimeoutOnClose {
    fn from(domain_msg: MsgTimeoutOnClose) -> Self {
        RawMsgTimeoutOnClose {
            packet: Some(domain_msg.packet.into()),
            proof_unreceived: domain_msg.proof_unreceived_on_b.into(),
            proof_close: domain_msg.proof_close_on_b.into(),
            proof_height: Some(domain_msg.proof_height_on_b.into()),
            next_sequence_recv: domain_msg.next_seq_recv_on_b.into(),
            signer: domain_msg.signer.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use ibc_primitives::prelude::*;
    use ibc_proto::ibc::core::channel::v1::MsgTimeoutOnClose as RawMsgTimeoutOnClose;
    use ibc_testkit::utils::core::channel::dummy_raw_msg_timeout_on_close;

    use crate::msgs::timeout_on_close::MsgTimeoutOnClose;

    #[test]
    fn msg_timeout_on_close_try_from_raw() {
        let height = 50;
        let timeout_timestamp = 5;
        let raw = dummy_raw_msg_timeout_on_close(height, timeout_timestamp);

        let msg = MsgTimeoutOnClose::try_from(raw.clone()).unwrap();
        let raw_back = RawMsgTimeoutOnClose::from(msg);
        assert_eq!(raw, raw_back);
    }

    #[test]
    fn parse_timeout_on_close_msg() {
        struct Test {
            name: String,
            raw: RawMsgTimeoutOnClose,
            want_pass: bool,
        }

        let height = 50;
        let timeout_timestamp = 5;
        let default_raw_msg = dummy_raw_msg_timeout_on_close(height, timeout_timestamp);

        let tests: Vec<Test> = vec![
            Test {
                name: "Good parameters".to_string(),
                raw: default_raw_msg.clone(),
                want_pass: true,
            },
            Test {
                name: "Missing packet".to_string(),
                raw: RawMsgTimeoutOnClose {
                    packet: None,
                    ..default_raw_msg.clone()
                },
                want_pass: false,
            },
            Test {
                name: "Missing proof of unreceived packet".to_string(),
                raw: RawMsgTimeoutOnClose {
                    proof_unreceived: Vec::new(),
                    ..default_raw_msg.clone()
                },
                want_pass: false,
            },
            Test {
                name: "Missing proof of channel".to_string(),
                raw: RawMsgTimeoutOnClose {
                    proof_close: Vec::new(),
                    ..default_raw_msg.clone()
                },
                want_pass: false,
            },
            Test {
                name: "Missing proof height".to_string(),
                raw: RawMsgTimeoutOnClose {
                    proof_height: None,
                    ..default_raw_msg
                },
                want_pass: false,
            },
        ];

        for test in tests {
            let res_msg = MsgTimeoutOnClose::try_from(test.raw.clone());

            assert_eq!(
                test.want_pass,
                res_msg.is_ok(),
                "MsgTimeoutOnClose::try_from raw failed for test {}, \nraw msg {:?} with error {:?}",
                test.name,
                test.raw,
                res_msg.err(),
            );
        }
    }
}
