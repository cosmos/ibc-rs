use ibc::core::channel::types::proto::v1::MsgTimeout as RawMsgTimeout;
use ibc::core::client::types::proto::v1::Height as RawHeight;

use super::{dummy_proof, dummy_raw_packet};
use crate::fixtures::core::signer::dummy_bech32_account;

/// Returns a dummy `RawMsgTimeout`, for testing purposes only!
/// The `height` parametrizes both the proof height as well as the timeout height.
pub fn dummy_raw_msg_timeout(
    proof_height: u64,
    timeout_height: u64,
    timeout_timestamp: u64,
) -> RawMsgTimeout {
    RawMsgTimeout {
        packet: Some(dummy_raw_packet(timeout_height, timeout_timestamp)),
        proof_unreceived: dummy_proof(),
        proof_height: Some(RawHeight {
            revision_number: 0,
            revision_height: proof_height,
        }),
        next_sequence_recv: 1,
        signer: dummy_bech32_account(),
    }
}

#[cfg(test)]
mod test {
    use ibc::core::channel::types::error::PacketError;
    use ibc::core::channel::types::msgs::MsgTimeout;
    use ibc::primitives::prelude::*;

    use super::*;

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
