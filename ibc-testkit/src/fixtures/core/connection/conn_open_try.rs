use ibc::core::client::types::proto::v1::Height as RawHeight;
use ibc::core::client::types::Height;
use ibc::core::connection::types::msgs::MsgConnectionOpenTry;
use ibc::core::connection::types::proto::v1::MsgConnectionOpenTry as RawMsgConnectionOpenTry;
use ibc::core::connection::types::version::Version as ConnectionVersion;
use ibc::core::host::types::identifiers::{ClientId, ConnectionId};
use ibc::core::primitives::prelude::*;

use super::dummy_raw_counterparty_conn;
use crate::fixtures::core::channel::dummy_proof;
use crate::fixtures::core::signer::dummy_bech32_account;
use crate::testapp::ibc::clients::mock::client_state::MockClientState;
use crate::testapp::ibc::clients::mock::header::MockHeader;

/// Returns a dummy `MsgConnectionOpenTry` for testing purposes only!
pub fn dummy_msg_conn_open_try(proof_height: u64, consensus_height: u64) -> MsgConnectionOpenTry {
    MsgConnectionOpenTry::try_from(dummy_raw_msg_conn_open_try(proof_height, consensus_height))
        .expect("Never fails")
}
/// Setter for the `client_id`
pub fn msg_conn_open_try_with_client_id(
    msg: MsgConnectionOpenTry,
    client_id: ClientId,
) -> MsgConnectionOpenTry {
    MsgConnectionOpenTry {
        client_id_on_b: client_id,
        ..msg
    }
}

/// Returns a dummy `RawMsgConnectionOpenTry` with parametrized heights. The parameter
/// `proof_height` represents the height, on the source chain, at which this chain produced the
/// proof. Parameter `consensus_height` represents the height of destination chain which a
/// client on the source chain stores.
pub fn dummy_raw_msg_conn_open_try(
    proof_height: u64,
    consensus_height: u64,
) -> RawMsgConnectionOpenTry {
    let client_state_height = Height::new(0, consensus_height).expect("could not create height");

    #[allow(deprecated)]
    RawMsgConnectionOpenTry {
        client_id: "07-tendermint-0".into(),
        previous_connection_id: ConnectionId::zero().to_string(),
        client_state: Some(MockClientState::new(MockHeader::new(client_state_height)).into()),
        counterparty: Some(dummy_raw_counterparty_conn(Some(0))),
        delay_period: 0,
        counterparty_versions: ConnectionVersion::compatibles()
            .iter()
            .map(|v| v.clone().into())
            .collect(),
        proof_init: dummy_proof(),
        proof_height: Some(RawHeight {
            revision_number: 0,
            revision_height: proof_height,
        }),
        proof_consensus: dummy_proof(),
        consensus_height: Some(RawHeight {
            revision_number: 0,
            revision_height: consensus_height,
        }),
        proof_client: dummy_proof(),
        signer: dummy_bech32_account(),
        host_consensus_state_proof: vec![],
    }
}

#[cfg(test)]
mod tests {
    use ibc::core::connection::types::proto::v1::Counterparty as RawCounterparty;

    use super::*;

    #[test]
    fn parse_connection_open_try_msg() {
        #[derive(Clone, Debug, PartialEq)]
        struct Test {
            name: String,
            raw: RawMsgConnectionOpenTry,
            want_pass: bool,
        }

        let default_try_msg = dummy_raw_msg_conn_open_try(10, 34);

        let tests: Vec<Test> =
            vec![
                Test {
                    name: "Good parameters".to_string(),
                    raw: default_try_msg.clone(),
                    want_pass: true,
                },
                Test {
                    name: "Bad client id, name too short".to_string(),
                    raw: RawMsgConnectionOpenTry {
                        client_id: "client".to_string(),
                        ..default_try_msg.clone()
                    },
                    want_pass: false,
                },
                Test {
                    name: "Bad destination connection id, name too long".to_string(),
                    raw: RawMsgConnectionOpenTry {
                        counterparty: Some(RawCounterparty {
                            connection_id:
                            "abcdasdfasdfsdfasfdwefwfsdfsfsfasfwewvxcvdvwgadvaadsefghijklmnopqrstu"
                                .to_string(),
                            ..dummy_raw_counterparty_conn(Some(0))
                        }),
                        ..default_try_msg.clone()
                    },
                    want_pass: false,
                },
                Test {
                    name: "Correct destination client id with lower/upper case and special chars"
                        .to_string(),
                    raw: RawMsgConnectionOpenTry {
                        counterparty: Some(RawCounterparty {
                            client_id: "ClientId_".to_string(),
                            ..dummy_raw_counterparty_conn(Some(0))
                        }),
                        ..default_try_msg.clone()
                    },
                    want_pass: true,
                },
                Test {
                    name: "Bad counterparty versions, empty versions vec".to_string(),
                    raw: RawMsgConnectionOpenTry {
                        counterparty_versions: Vec::new(),
                        ..default_try_msg.clone()
                    },
                    want_pass: false,
                },
                Test {
                    name: "Bad counterparty versions, empty version string".to_string(),
                    raw: RawMsgConnectionOpenTry {
                        counterparty_versions: Vec::new(),
                        ..default_try_msg.clone()
                    },
                    want_pass: false,
                },
                Test {
                    name: "Bad proof height, height is 0".to_string(),
                    raw: RawMsgConnectionOpenTry {
                        proof_height: Some(RawHeight { revision_number: 1, revision_height: 0 }),
                        ..default_try_msg.clone()
                    },
                    want_pass: false,
                },
                Test {
                    name: "Bad consensus height, height is 0".to_string(),
                    raw: RawMsgConnectionOpenTry {
                        proof_height: Some(RawHeight { revision_number: 1, revision_height: 0 }),
                        ..default_try_msg.clone()
                    },
                    want_pass: false,
                },
                Test {
                    name: "Empty proof".to_string(),
                    raw: RawMsgConnectionOpenTry {
                        proof_init: b"".to_vec(),
                        ..default_try_msg
                    },
                    want_pass: false,
                }
            ]
            .into_iter()
            .collect();

        for test in tests {
            let msg = MsgConnectionOpenTry::try_from(test.raw.clone());

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
        let raw = dummy_raw_msg_conn_open_try(10, 34);
        let msg = MsgConnectionOpenTry::try_from(raw.clone()).unwrap();
        let raw_back = RawMsgConnectionOpenTry::from(msg.clone());
        let msg_back = MsgConnectionOpenTry::try_from(raw_back.clone()).unwrap();
        assert_eq!(raw, raw_back);
        assert_eq!(msg, msg_back);
    }

    /// Test that borsh serialization/deserialization works well with delay periods up to u64::MAX
    #[cfg(feature = "borsh")]
    #[test]
    fn test_borsh() {
        let mut raw = dummy_raw_msg_conn_open_try(10, 34);
        raw.delay_period = u64::MAX;
        let msg = MsgConnectionOpenTry::try_from(raw.clone()).unwrap();

        let serialized = borsh::to_vec(&msg).unwrap();

        let msg_deserialized =
            <MsgConnectionOpenTry as borsh::BorshDeserialize>::try_from_slice(&serialized).unwrap();

        assert_eq!(msg, msg_deserialized);
    }
}
