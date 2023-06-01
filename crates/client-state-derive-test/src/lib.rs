#![allow(dead_code)]

#[cfg(test)]
mod test {
    use ibc::core::ics02_client::client_state::ClientStateInitializer;
    use ibc::core::ics02_client::error::ClientError;
    use ibc::core::ics02_client::ClientState;
    use ibc::core::ics23_commitment::merkle::MerkleProof;
    use ibc::{core::ics02_client::client_state::ClientStateBase, Any};

    enum HostConsensusState {
        First(FirstConsensusState),
    }

    #[derive(Debug, PartialEq, Clone, ClientState)]
    #[host(consensus_state = HostConsensusState)]
    enum HostClientState {
        First(FirstClientState),
    }

    #[derive(Debug, Clone, PartialEq)]
    struct FirstClientState;
    struct FirstConsensusState;

    impl ClientStateBase for FirstClientState {
        fn client_type(&self) -> ibc::core::ics02_client::client_type::ClientType {
            todo!()
        }

        fn latest_height(&self) -> ibc::Height {
            todo!()
        }

        fn validate_proof_height(&self, _proof_height: ibc::Height) -> Result<(), ClientError> {
            todo!()
        }

        fn confirm_not_frozen(&self) -> Result<(), ClientError> {
            todo!()
        }

        fn expired(&self, _elapsed: core::time::Duration) -> bool {
            todo!()
        }

        fn verify_upgrade_client(
            &self,
            _upgraded_client_state: Any,
            _upgraded_consensus_state: Any,
            _proof_upgrade_client: MerkleProof,
            _proof_upgrade_consensus_state: MerkleProof,
            _root: &ibc::core::ics23_commitment::commitment::CommitmentRoot,
        ) -> Result<(), ClientError> {
            todo!()
        }

        fn verify_membership(
            &self,
            _prefix: &ibc::core::ics23_commitment::commitment::CommitmentPrefix,
            _proof: &ibc::core::ics23_commitment::commitment::CommitmentProofBytes,
            _root: &ibc::core::ics23_commitment::commitment::CommitmentRoot,
            _path: ibc::core::ics24_host::path::Path,
            _value: Vec<u8>,
        ) -> Result<(), ClientError> {
            todo!()
        }

        fn verify_non_membership(
            &self,
            _prefix: &ibc::core::ics23_commitment::commitment::CommitmentPrefix,
            _proof: &ibc::core::ics23_commitment::commitment::CommitmentProofBytes,
            _root: &ibc::core::ics23_commitment::commitment::CommitmentRoot,
            _path: ibc::core::ics24_host::path::Path,
        ) -> Result<(), ClientError> {
            todo!()
        }
    }

    impl ClientStateInitializer<HostConsensusState> for FirstClientState {
        fn initialise(&self, _consensus_state: Any) -> Result<HostConsensusState, ClientError> {
            todo!()
        }
    }
}
