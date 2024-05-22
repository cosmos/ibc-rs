#[cfg(all(test, feature = "serde"))]
mod tests {
    use core::time::Duration;

    use ibc_client_tendermint_types::error::Error;
    use ibc_client_tendermint_types::{AllowUpdate, ClientState, TrustThreshold};
    use ibc_core_client_types::Height;
    use ibc_core_commitment_types::specs::ProofSpecs;
    use ibc_core_host_types::identifiers::ChainId;
    use ibc_primitives::ZERO_DURATION;
    use ibc_testkit::fixtures::clients::tendermint::test_serialization_roundtrip;
    use tendermint_rpc::endpoint::abci_query::AbciQuery;

    #[test]
    fn serialization_roundtrip_no_proof() {
        let json_data = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/data/json/client_state.json"
        ));
        test_serialization_roundtrip::<AbciQuery>(json_data);
    }

    #[test]
    fn serialization_roundtrip_with_proof() {
        let json_data = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/data/json/client_state_proof.json"
        ));
        test_serialization_roundtrip::<AbciQuery>(json_data);
    }
}
