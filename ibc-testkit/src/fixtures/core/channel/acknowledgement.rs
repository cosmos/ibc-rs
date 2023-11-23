use ibc::core::channel::types::proto::v1::{
    MsgAcknowledgement as RawMsgAcknowledgement, Packet as RawPacket,
};
use ibc::core::client::types::proto::v1::Height as RawHeight;

use super::{dummy_proof, dummy_raw_packet};
use crate::fixtures::core::signer::dummy_bech32_account;

/// Returns a dummy `RawMsgAcknowledgement`, for testing purposes only!
/// The `height` parametrizes both the proof height as well as the timeout height.
pub fn dummy_raw_msg_acknowledgement(height: u64) -> RawMsgAcknowledgement {
    dummy_raw_msg_ack_with_packet(dummy_raw_packet(height, 1), height)
}

pub fn dummy_raw_msg_ack_with_packet(packet: RawPacket, height: u64) -> RawMsgAcknowledgement {
    RawMsgAcknowledgement {
        packet: Some(packet),
        acknowledgement: dummy_proof(),
        proof_acked: dummy_proof(),
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
    use ibc::core::channel::types::msgs::MsgAcknowledgement;
    use ibc::primitives::prelude::*;

    use super::*;

    #[test]
    fn msg_acknowledgment_try_from_raw() {
        struct Test {
            name: String,
            raw: RawMsgAcknowledgement,
            want_pass: bool,
        }

        let height = 50;
        let default_raw_msg = dummy_raw_msg_acknowledgement(height);

        let tests: Vec<Test> = vec![
            Test {
                name: "Good parameters".to_string(),
                raw: default_raw_msg.clone(),
                want_pass: true,
            },
            Test {
                name: "Missing packet".to_string(),
                raw: RawMsgAcknowledgement {
                    packet: None,
                    ..default_raw_msg.clone()
                },
                want_pass: false,
            },
            Test {
                name: "Missing proof height".to_string(),
                raw: RawMsgAcknowledgement {
                    proof_height: None,
                    ..default_raw_msg.clone()
                },
                want_pass: false,
            },
            Test {
                name: "Empty signer".to_string(),
                raw: RawMsgAcknowledgement {
                    signer: dummy_bech32_account(),
                    ..default_raw_msg.clone()
                },
                want_pass: true,
            },
            Test {
                name: "Empty proof acked".to_string(),
                raw: RawMsgAcknowledgement {
                    proof_acked: Vec::new(),
                    ..default_raw_msg
                },
                want_pass: false,
            },
        ];

        for test in tests {
            let res_msg: Result<MsgAcknowledgement, PacketError> = test.raw.clone().try_into();

            assert_eq!(
                res_msg.is_ok(),
                test.want_pass,
                "MsgAcknowledgement::try_from failed for test {} \nraw message: {:?} with error: {:?}",
                test.name,
                test.raw,
                res_msg.err()
            );
        }
    }
}
