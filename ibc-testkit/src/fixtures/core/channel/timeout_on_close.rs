use ibc::core::channel::types::proto::v1::MsgTimeoutOnClose as RawMsgTimeoutOnClose;
use ibc::core::client::types::proto::v1::Height as RawHeight;

use super::{dummy_proof, dummy_raw_packet};
use crate::fixtures::core::signer::dummy_bech32_account;

/// Returns a dummy `RawMsgTimeoutOnClose`, for testing purposes only!
/// The `height` parametrizes both the proof height as well as the timeout height.
pub fn dummy_raw_msg_timeout_on_close(height: u64, timeout_timestamp: u64) -> RawMsgTimeoutOnClose {
    RawMsgTimeoutOnClose {
        packet: Some(dummy_raw_packet(height, timeout_timestamp)),
        proof_unreceived: dummy_proof(),
        proof_close: dummy_proof(),
        proof_height: Some(RawHeight {
            revision_number: 0,
            revision_height: height,
        }),
        next_sequence_recv: 1,
        signer: dummy_bech32_account(),
        counterparty_upgrade_sequence: 0,
    }
}

#[cfg(test)]
mod tests {
    use ibc::core::channel::types::msgs::MsgTimeoutOnClose;
    use ibc::primitives::prelude::*;

    use super::*;

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
