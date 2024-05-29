use ibc::core::client::types::proto::v1::Height;
use ibc::core::connection::types::msgs::MsgConnectionOpenConfirm;
use ibc::core::connection::types::proto::v1::MsgConnectionOpenConfirm as RawMsgConnectionOpenConfirm;
use ibc::primitives::prelude::*;

use crate::fixtures::core::channel::dummy_proof;
use crate::fixtures::core::signer::dummy_bech32_account;

/// Returns a dummy `MsgConnectionOpenConfirm` for testing purposes only!
pub fn dummy_conn_open_confirm() -> MsgConnectionOpenConfirm {
    MsgConnectionOpenConfirm::try_from(dummy_raw_msg_conn_open_confirm()).expect("Never fails")
}

/// Returns a dummy `RawMsgConnectionOpenConfirm` for testing purposes only!
pub fn dummy_raw_msg_conn_open_confirm() -> RawMsgConnectionOpenConfirm {
    RawMsgConnectionOpenConfirm {
        connection_id: "connection-118".to_string(),
        proof_ack: dummy_proof(),
        proof_height: Some(Height {
            revision_number: 0,
            revision_height: 10,
        }),
        signer: dummy_bech32_account(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_connection_open_confirm_msg() {
        #[derive(Clone, Debug, PartialEq)]
        struct Test {
            name: String,
            raw: RawMsgConnectionOpenConfirm,
            want_pass: bool,
        }

        let default_ack_msg = dummy_raw_msg_conn_open_confirm();
        let tests: Vec<Test> = vec![
            Test {
                name: "Good parameters".to_string(),
                raw: default_ack_msg.clone(),
                want_pass: true,
            },
            Test {
                name: "Bad connection id, non-alpha".to_string(),
                raw: RawMsgConnectionOpenConfirm {
                    connection_id: "con007".to_string(),
                    ..default_ack_msg.clone()
                },
                want_pass: false,
            },
            Test {
                name: "Bad proof height, height is 0".to_string(),
                raw: RawMsgConnectionOpenConfirm {
                    proof_height: Some(Height {
                        revision_number: 1,
                        revision_height: 0,
                    }),
                    ..default_ack_msg
                },
                want_pass: false,
            },
        ]
        .into_iter()
        .collect();

        for test in tests {
            let msg = MsgConnectionOpenConfirm::try_from(test.raw.clone());

            assert_eq!(
                test.want_pass,
                msg.is_ok(),
                "MsgConnOpenTry::new failed for test {}, \nmsg {:?} with error {:?}",
                test.name,
                test.raw,
                msg.err(),
            );
        }
    }

    #[test]
    fn to_and_from() {
        let raw = dummy_raw_msg_conn_open_confirm();
        let msg = MsgConnectionOpenConfirm::try_from(raw.clone()).unwrap();
        let raw_back = RawMsgConnectionOpenConfirm::from(msg.clone());
        let msg_back = MsgConnectionOpenConfirm::try_from(raw_back.clone()).unwrap();
        assert_eq!(raw, raw_back);
        assert_eq!(msg, msg_back);
    }
}
