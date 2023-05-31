#[cfg(test)]
mod test {
    use ibc::core::ics02_client::ClientState;
    use ibc::core::ics23_commitment::merkle::MerkleProof;
    use ibc::{core::ics02_client::client_state::ClientStateBase, Any};

    #[derive(Debug, PartialEq, Clone, ClientState)]
    enum HostClientState {
        First(FirstClientState),
    }

    #[derive(Debug, Clone, PartialEq)]
    struct FirstClientState;

    impl ClientStateBase for FirstClientState {
        fn client_type(&self) -> ibc::core::ics02_client::client_type::ClientType {
            todo!()
        }

        fn latest_height(&self) -> ibc::Height {
            todo!()
        }

        fn validate_proof_height(
            &self,
            proof_height: ibc::Height,
        ) -> Result<(), ibc::core::ics02_client::error::ClientError> {
            todo!()
        }

        fn confirm_not_frozen(&self) -> Result<(), ibc::core::ics02_client::error::ClientError> {
            todo!()
        }

        fn expired(&self, elapsed: core::time::Duration) -> bool {
            todo!()
        }

        fn verify_upgrade_client(
            &self,
            upgraded_client_state: Any,
            upgraded_consensus_state: Any,
            proof_upgrade_client: MerkleProof,
            proof_upgrade_consensus_state: MerkleProof,
            root: &ibc::core::ics23_commitment::commitment::CommitmentRoot,
        ) -> Result<(), ibc::core::ics02_client::error::ClientError> {
            todo!()
        }

        fn verify_membership(
            &self,
            prefix: &ibc::core::ics23_commitment::commitment::CommitmentPrefix,
            proof: &ibc::core::ics23_commitment::commitment::CommitmentProofBytes,
            root: &ibc::core::ics23_commitment::commitment::CommitmentRoot,
            path: ibc::core::ics24_host::path::Path,
            value: Vec<u8>,
        ) -> Result<(), ibc::core::ics02_client::error::ClientError> {
            todo!()
        }

        fn verify_non_membership(
            &self,
            prefix: &ibc::core::ics23_commitment::commitment::CommitmentPrefix,
            proof: &ibc::core::ics23_commitment::commitment::CommitmentProofBytes,
            root: &ibc::core::ics23_commitment::commitment::CommitmentRoot,
            path: ibc::core::ics24_host::path::Path,
        ) -> Result<(), ibc::core::ics02_client::error::ClientError> {
            todo!()
        }
    }
}
