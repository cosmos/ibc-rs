#[cfg(feature = "serde")]
mod tests {
    use ibc_testkit::fixtures::clients::tendermint::test_serialization_roundtrip;
    use tendermint_rpc::endpoint::abci_query::AbciQuery;

    #[test]
    fn serialization_roundtrip_no_proof() {
        let json_data = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/data/json/consensus_state.json"
        ));
        test_serialization_roundtrip::<AbciQuery>(json_data);
    }

    #[test]
    fn serialization_roundtrip_with_proof() {
        let json_data = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/data/json/consensus_state_proof.json"
        ));
        test_serialization_roundtrip::<AbciQuery>(json_data);
    }
}
