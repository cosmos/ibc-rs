use core::ops::Add;
use core::time::Duration;

use ibc::core::channel::types::msgs::MsgRecvPacket;
use ibc::core::channel::types::packet::Packet;
use ibc::core::channel::types::proto::v1::MsgRecvPacket as RawMsgRecvPacket;
use ibc::core::client::types::proto::v1::Height as RawHeight;
use ibc::core::client::types::Height;
use ibc::core::commitment_types::commitment::CommitmentProofBytes;
use ibc::core::primitives::{Signer, Timestamp};

use super::{dummy_proof, dummy_raw_packet};
use crate::fixtures::core::signer::dummy_bech32_account;

pub fn dummy_msg_recv_packet(
    packet: Packet,
    proof_commitment_on_a: CommitmentProofBytes,
    proof_height_on_a: Height,
    signer: Signer,
) -> MsgRecvPacket {
    MsgRecvPacket {
        packet,
        proof_commitment_on_a,
        proof_height_on_a,
        signer,
    }
}

/// Returns a dummy `RawMsgRecvPacket`, for testing purposes only! The `height`
/// parametrizes both the proof height as well as the timeout height.
pub fn dummy_raw_msg_recv_packet(height: u64) -> RawMsgRecvPacket {
    let timestamp = Timestamp::now().add(Duration::from_secs(9));
    RawMsgRecvPacket {
        packet: Some(dummy_raw_packet(
            height,
            timestamp.expect("timestamp").nanoseconds(),
        )),
        proof_commitment: dummy_proof(),
        proof_height: Some(RawHeight {
            revision_number: 0,
            revision_height: height,
        }),
        signer: dummy_bech32_account(),
    }
}

#[cfg(test)]
mod test {
    use ibc::core::channel::types::error::PacketError;
    use ibc::primitives::prelude::*;

    use super::*;

    #[test]
    fn msg_recv_packet_try_from_raw() {
        struct Test {
            name: String,
            raw: RawMsgRecvPacket,
            want_pass: bool,
        }

        let height = 20;
        let default_raw_msg = dummy_raw_msg_recv_packet(height);
        let tests: Vec<Test> = vec![
            Test {
                name: "Good parameters".to_string(),
                raw: default_raw_msg.clone(),
                want_pass: true,
            },
            Test {
                name: "Missing proof".to_string(),
                raw: RawMsgRecvPacket {
                    proof_commitment: Vec::new(),
                    ..default_raw_msg.clone()
                },
                want_pass: false,
            },
            Test {
                name: "Missing proof height".to_string(),
                raw: RawMsgRecvPacket {
                    proof_height: None,
                    ..default_raw_msg.clone()
                },
                want_pass: false,
            },
            Test {
                name: "Empty signer".to_string(),
                raw: RawMsgRecvPacket {
                    signer: dummy_bech32_account(),
                    ..default_raw_msg
                },
                want_pass: true,
            },
        ];

        for test in tests {
            let res_msg: Result<MsgRecvPacket, PacketError> = test.raw.clone().try_into();

            assert_eq!(
                res_msg.is_ok(),
                test.want_pass,
                "MsgRecvPacket::try_from failed for test {} \nraw message: {:?} with error: {:?}",
                test.name,
                test.raw,
                res_msg.err()
            );
        }
    }

    #[test]
    fn to_and_from() {
        let raw = dummy_raw_msg_recv_packet(15);
        let msg = MsgRecvPacket::try_from(raw.clone()).unwrap();
        let raw_back = RawMsgRecvPacket::from(msg.clone());
        let msg_back = MsgRecvPacket::try_from(raw_back.clone()).unwrap();
        assert_eq!(raw, raw_back);
        assert_eq!(msg, msg_back);
    }
}
