use ibc::core::client::types::proto::v1::Height as RawHeight;
use ibc::core::client::types::Height;
use ibc::core::connection::types::msgs::MsgConnectionOpenAck;
use ibc::core::connection::types::proto::v1::MsgConnectionOpenAck as RawMsgConnectionOpenAck;
use ibc::core::connection::types::version::Version as ConnectionVersion;
use ibc::core::host::types::identifiers::ConnectionId;
use ibc::core::primitives::prelude::*;

use crate::fixtures::core::channel::dummy_proof;
use crate::fixtures::core::signer::dummy_bech32_account;
use crate::testapp::ibc::clients::mock::client_state::MockClientState;
use crate::testapp::ibc::clients::mock::header::MockHeader;

/// Returns a dummy `MsgConnectionOpenAck` with dummy values.
pub fn dummy_msg_conn_open_ack(proof_height: u64, consensus_height: u64) -> MsgConnectionOpenAck {
    MsgConnectionOpenAck::try_from(dummy_raw_msg_conn_open_ack(proof_height, consensus_height))
        .expect("Never fails")
}

/// Returns a dummy `RawMsgConnectionOpenAck`, for testing purposes only!
pub fn dummy_raw_msg_conn_open_ack(
    proof_height: u64,
    consensus_height: u64,
) -> RawMsgConnectionOpenAck {
    let client_state_height = Height::new(0, consensus_height).expect("invalid height");
    RawMsgConnectionOpenAck {
        connection_id: ConnectionId::zero().to_string(),
        counterparty_connection_id: ConnectionId::new(1).to_string(),
        proof_try: dummy_proof(),
        proof_height: Some(RawHeight {
            revision_number: 0,
            revision_height: proof_height,
        }),
        proof_consensus: dummy_proof(),
        consensus_height: Some(RawHeight {
            revision_number: 0,
            revision_height: consensus_height,
        }),
        client_state: Some(MockClientState::new(MockHeader::new(client_state_height)).into()),
        proof_client: dummy_proof(),
        version: Some(ConnectionVersion::compatibles()[0].clone().into()),
        signer: dummy_bech32_account(),
        host_consensus_state_proof: vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_connection_open_ack_msg() {
        #[derive(Clone, Debug, PartialEq)]
        struct Test {
            name: String,
            raw: RawMsgConnectionOpenAck,
            want_pass: bool,
        }

        let default_ack_msg = dummy_raw_msg_conn_open_ack(5, 5);

        let tests: Vec<Test> = vec![
            Test {
                name: "Good parameters".to_string(),
                raw: default_ack_msg.clone(),
                want_pass: true,
            },
            Test {
                name: "Bad connection id, non-alpha".to_string(),
                raw: RawMsgConnectionOpenAck {
                    connection_id: "con007".to_string(),
                    ..default_ack_msg.clone()
                },
                want_pass: false,
            },
            Test {
                name: "Bad version, missing version".to_string(),
                raw: RawMsgConnectionOpenAck {
                    version: None,
                    ..default_ack_msg.clone()
                },
                want_pass: false,
            },
            Test {
                name: "Bad proof height, height is 0".to_string(),
                raw: RawMsgConnectionOpenAck {
                    proof_height: Some(RawHeight {
                        revision_number: 1,
                        revision_height: 0,
                    }),
                    ..default_ack_msg.clone()
                },
                want_pass: false,
            },
            Test {
                name: "Bad consensus height, height is 0".to_string(),
                raw: RawMsgConnectionOpenAck {
                    consensus_height: Some(RawHeight {
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
            let msg = MsgConnectionOpenAck::try_from(test.raw.clone());

            assert_eq!(
                test.want_pass,
                msg.is_ok(),
                "MsgConnOpenAck::new failed for test {}, \nmsg {:?} with error {:?}",
                test.name,
                test.raw,
                msg.err(),
            );
        }
    }

    #[test]
    fn to_and_from() {
        let raw = dummy_raw_msg_conn_open_ack(5, 6);
        let msg = MsgConnectionOpenAck::try_from(raw.clone()).unwrap();
        let raw_back = RawMsgConnectionOpenAck::from(msg.clone());
        let msg_back = MsgConnectionOpenAck::try_from(raw_back.clone()).unwrap();
        assert_eq!(raw, raw_back);
        assert_eq!(msg, msg_back);
    }
}
