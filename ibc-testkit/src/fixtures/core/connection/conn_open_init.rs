use ibc::core::connection::types::msgs::MsgConnectionOpenInit;
use ibc::core::connection::types::proto::v1::{
    MsgConnectionOpenInit as RawMsgConnectionOpenInit, Version as RawVersion,
};
use ibc::core::connection::types::version::Version as ConnectionVersion;
use ibc::core::connection::types::Counterparty;
use ibc::core::host::types::identifiers::ClientId;
use ibc::core::primitives::prelude::*;

use super::dummy_raw_counterparty_conn;
use crate::fixtures::core::signer::dummy_bech32_account;

pub fn raw_version_from_identifier(identifier: &str) -> Option<RawVersion> {
    if identifier.is_empty() {
        return None;
    }

    Some(RawVersion {
        identifier: identifier.to_string(),
        features: vec![],
    })
}

/// Returns a dummy `MsgConnectionOpenInit` for testing purposes only!
pub fn dummy_msg_conn_open_init() -> MsgConnectionOpenInit {
    MsgConnectionOpenInit::try_from(dummy_raw_msg_conn_open_init()).expect("Never fails")
}

/// Setter for `client_id`. Amenable to chaining, since it consumes the input message.
pub fn dummy_msg_conn_open_init_with_client_id(
    msg: MsgConnectionOpenInit,
    client_id: ClientId,
) -> MsgConnectionOpenInit {
    MsgConnectionOpenInit {
        client_id_on_a: client_id,
        ..msg
    }
}

/// Setter for `counterparty`. Amenable to chaining, since it consumes the input message.
pub fn msg_conn_open_init_with_counterparty_conn_id(
    msg: MsgConnectionOpenInit,
    counterparty_conn_id: u64,
) -> MsgConnectionOpenInit {
    let counterparty =
        Counterparty::try_from(dummy_raw_counterparty_conn(Some(counterparty_conn_id)))
            .expect("Never fails");
    MsgConnectionOpenInit {
        counterparty,
        ..msg
    }
}

/// Setter for the connection `version`
pub fn msg_conn_open_with_version(
    msg: MsgConnectionOpenInit,
    identifier: Option<&str>,
) -> MsgConnectionOpenInit {
    let version = match identifier {
        Some(v) => ConnectionVersion::try_from(RawVersion {
            identifier: v.to_string(),
            features: vec![],
        })
        .expect("could not create version from identifier")
        .into(),
        None => None,
    };
    MsgConnectionOpenInit { version, ..msg }
}

/// Returns a dummy `RawMsgConnectionOpenInit`, for testing purposes only!
pub fn dummy_raw_msg_conn_open_init() -> RawMsgConnectionOpenInit {
    RawMsgConnectionOpenInit {
        client_id: "07-tendermint-0".into(),
        counterparty: Some(dummy_raw_counterparty_conn(None)),
        version: Some(ConnectionVersion::compatibles()[0].clone().into()),
        delay_period: 0,
        signer: dummy_bech32_account(),
    }
}

#[cfg(test)]
mod tests {
    use ibc::core::connection::types::proto::v1::Counterparty as RawCounterparty;

    use super::*;

    #[test]
    fn parse_connection_open_init_msg() {
        #[derive(Clone, Debug, PartialEq)]
        struct Test {
            name: String,
            raw: RawMsgConnectionOpenInit,
            want_pass: bool,
        }

        let default_init_msg = dummy_raw_msg_conn_open_init();

        let tests: Vec<Test> = vec![
            Test {
                name: "Good parameters".to_string(),
                raw: default_init_msg.clone(),
                want_pass: true,
            },
            Test {
                name: "Bad client id, name too short".to_string(),
                raw: RawMsgConnectionOpenInit {
                    client_id: "client".to_string(),
                    ..default_init_msg.clone()
                },
                want_pass: false,
            },
            Test {
                name: "Bad destination connection id, name too long".to_string(),
                raw: RawMsgConnectionOpenInit {
                    counterparty: Some(RawCounterparty {
                        connection_id:
                            "abcdefghijksdffjssdkflweldflsfladfsfwjkrekcmmsdfsdfjflddmnopqrstu"
                                .to_string(),
                        ..dummy_raw_counterparty_conn(None)
                    }),
                    ..default_init_msg
                },
                want_pass: false,
            },
        ]
        .into_iter()
        .collect();

        for test in tests {
            let msg = MsgConnectionOpenInit::try_from(test.raw.clone());

            assert_eq!(
                test.want_pass,
                msg.is_ok(),
                "MsgConnOpenInit::new failed for test {}, \nmsg {:?} with error {:?}",
                test.name,
                test.raw,
                msg.err(),
            );
        }
    }

    #[test]
    fn to_and_from() {
        let raw = dummy_raw_msg_conn_open_init();
        let msg = MsgConnectionOpenInit::try_from(raw.clone()).unwrap();
        let raw_back = RawMsgConnectionOpenInit::from(msg.clone());
        let msg_back = MsgConnectionOpenInit::try_from(raw_back.clone()).unwrap();
        assert_eq!(raw, raw_back);
        assert_eq!(msg, msg_back);

        // Check if handler sets counterparty connection id to `None`
        // in case relayer passes `MsgConnectionOpenInit` message with it set to `Some(_)`.
        let raw_with_counterparty_conn_id_some = dummy_raw_msg_conn_open_init();
        let msg_with_counterparty_conn_id_some =
            MsgConnectionOpenInit::try_from(raw_with_counterparty_conn_id_some).unwrap();
        let raw_with_counterparty_conn_id_some_back =
            RawMsgConnectionOpenInit::from(msg_with_counterparty_conn_id_some.clone());
        let msg_with_counterparty_conn_id_some_back =
            MsgConnectionOpenInit::try_from(raw_with_counterparty_conn_id_some_back.clone())
                .unwrap();
        assert_eq!(
            raw_with_counterparty_conn_id_some_back
                .counterparty
                .unwrap()
                .connection_id,
            "".to_string()
        );
        assert_eq!(
            msg_with_counterparty_conn_id_some,
            msg_with_counterparty_conn_id_some_back
        );
    }

    /// Test that borsh serialization/deserialization works well with delay periods up to u64::MAX
    #[cfg(feature = "borsh")]
    #[test]
    fn test_borsh() {
        let mut raw = dummy_raw_msg_conn_open_init();
        raw.delay_period = u64::MAX;
        let msg = MsgConnectionOpenInit::try_from(raw.clone()).unwrap();

        let serialized = borsh::to_vec(&msg).unwrap();

        let msg_deserialized =
            <MsgConnectionOpenInit as borsh::BorshDeserialize>::try_from_slice(&serialized)
                .unwrap();

        assert_eq!(msg, msg_deserialized);
    }
}
