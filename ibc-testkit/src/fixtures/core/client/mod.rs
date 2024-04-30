#[cfg(feature = "serde")]
mod msg_create_client;
#[cfg(feature = "serde")]
mod msg_update_client;
mod msg_upgrade_client;

#[cfg(feature = "serde")]
pub use msg_create_client::*;
#[cfg(feature = "serde")]
pub use msg_update_client::*;
pub use msg_upgrade_client::*;

#[cfg(test)]
mod tests {
    use core::str::FromStr;

    use ibc::core::client::types::events::*;
    use ibc::core::client::types::Height;
    use ibc::core::host::types::identifiers::*;
    use ibc::primitives::prelude::*;
    use ibc::primitives::ToVec;
    use ibc_proto::google::protobuf::Any;
    use tendermint::abci::Event as AbciEvent;

    use crate::fixtures::clients::mock::dummy_new_mock_header;

    #[test]
    fn ibc_to_abci_client_events() {
        struct Test {
            event_kind: &'static str,
            event: AbciEvent,
            expected_keys: Vec<&'static str>,
            expected_values: Vec<&'static str>,
        }

        let client_type = ClientType::from_str("07-tendermint")
            .expect("never fails because it's a valid client type");
        let client_id = client_type.build_client_id(0);
        let consensus_height = Height::new(0, 5).unwrap();
        let consensus_heights = vec![Height::new(0, 5).unwrap(), Height::new(0, 7).unwrap()];
        let header: Any = dummy_new_mock_header(5).into();
        let expected_keys = vec![
            "client_id",
            "client_type",
            "consensus_height",
            "consensus_heights",
            "header",
        ];

        let expected_values = vec![
            "07-tendermint-0",
            "07-tendermint",
            "0-5",
            "0-5,0-7",
            "0a102f6962632e6d6f636b2e48656164657212040a021005",
        ];

        let tests: Vec<Test> = vec![
            Test {
                event_kind: CREATE_CLIENT_EVENT,
                event: CreateClient::new(client_id.clone(), client_type.clone(), consensus_height)
                    .into(),
                expected_keys: expected_keys[0..3].to_vec(),
                expected_values: expected_values[0..3].to_vec(),
            },
            Test {
                event_kind: UPDATE_CLIENT_EVENT,
                event: UpdateClient::new(
                    client_id.clone(),
                    client_type.clone(),
                    consensus_height,
                    consensus_heights,
                    header.to_vec(),
                )
                .into(),
                expected_keys: expected_keys.clone(),
                expected_values: expected_values.clone(),
            },
            Test {
                event_kind: UPGRADE_CLIENT_EVENT,
                event: UpgradeClient::new(client_id.clone(), client_type.clone(), consensus_height)
                    .into(),
                expected_keys: expected_keys[0..3].to_vec(),
                expected_values: expected_values[0..3].to_vec(),
            },
            Test {
                event_kind: CLIENT_MISBEHAVIOUR_EVENT,
                event: ClientMisbehaviour::new(client_id, client_type).into(),
                expected_keys: expected_keys[0..2].to_vec(),
                expected_values: expected_values[0..2].to_vec(),
            },
        ];

        for t in tests {
            assert_eq!(t.event.kind, t.event_kind);
            assert_eq!(t.expected_keys.len(), t.event.attributes.len());
            for (i, e) in t.event.attributes.iter().enumerate() {
                assert_eq!(
                    e.key_str().unwrap(),
                    t.expected_keys[i],
                    "key mismatch for {:?}",
                    t.event_kind
                );
            }
            for (i, e) in t.event.attributes.iter().enumerate() {
                assert_eq!(
                    e.value_str().unwrap(),
                    t.expected_values[i],
                    "value mismatch for {:?}",
                    t.event_kind
                );
            }
        }
    }
}
