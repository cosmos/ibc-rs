#![allow(dead_code)]

#[cfg(test)]
mod test {
    use core::time::Duration;

    use ibc::core::ics02_client::client_state::ClientState;
    use ibc::core::ics02_client::client_state::{
        ClientStateExecution, ClientStateInitializer, ClientStateValidation, UpdateKind,
    };
    use ibc::core::ics02_client::client_type::ClientType;
    use ibc::core::ics02_client::consensus_state::ConsensusState;
    use ibc::core::ics02_client::error::ClientError;
    use ibc::core::ics23_commitment::commitment::{
        CommitmentPrefix, CommitmentProofBytes, CommitmentRoot,
    };
    use ibc::core::ics23_commitment::merkle::MerkleProof;
    use ibc::core::ics24_host::identifier::ClientId;
    use ibc::core::ics24_host::path::Path;
    use ibc::Height;
    use ibc::{core::ics02_client::client_state::ClientStateBase, Any};

    enum ClientValidationContext {
        First(FirstClientValidationContext),
    }

    enum ClientExecutionContext {
        First(FirstClientExecutionContext),
    }

    #[derive(ConsensusState)]
    enum HostConsensusState {
        First(FirstConsensusState),
    }
    struct FirstConsensusState;

    impl ConsensusState for FirstConsensusState {
        fn root(&self) -> &CommitmentRoot {
            todo!()
        }

        fn timestamp(&self) -> ibc::core::timestamp::Timestamp {
            todo!()
        }

        fn encode_vec(&self) -> Result<Vec<u8>, tendermint_proto::Error> {
            todo!()
        }
    }

    #[derive(Debug, PartialEq, Clone, ClientState)]
    #[generics(AnyConsensusState = HostConsensusState,
               ClientValidationContext = ClientValidationContext,
               ClientExecutionContext = ClientExecutionContext)]
    enum HostClientState {
        First(FirstClientState),
    }

    #[derive(Debug, Clone, PartialEq)]
    struct FirstClientState;
    struct FirstClientValidationContext;
    struct FirstClientExecutionContext;

    impl ClientStateBase for FirstClientState {
        fn client_type(&self) -> ClientType {
            todo!()
        }

        fn latest_height(&self) -> Height {
            todo!()
        }

        fn validate_proof_height(&self, _proof_height: Height) -> Result<(), ClientError> {
            todo!()
        }

        fn confirm_not_frozen(&self) -> Result<(), ClientError> {
            todo!()
        }

        fn expired(&self, _elapsed: Duration) -> bool {
            todo!()
        }

        fn verify_upgrade_client(
            &self,
            _upgraded_client_state: Any,
            _upgraded_consensus_state: Any,
            _proof_upgrade_client: MerkleProof,
            _proof_upgrade_consensus_state: MerkleProof,
            _root: &CommitmentRoot,
        ) -> Result<(), ClientError> {
            todo!()
        }

        fn verify_membership(
            &self,
            _prefix: &CommitmentPrefix,
            _proof: &CommitmentProofBytes,
            _root: &CommitmentRoot,
            _path: Path,
            _value: Vec<u8>,
        ) -> Result<(), ClientError> {
            todo!()
        }

        fn verify_non_membership(
            &self,
            _prefix: &CommitmentPrefix,
            _proof: &CommitmentProofBytes,
            _root: &CommitmentRoot,
            _path: Path,
        ) -> Result<(), ClientError> {
            todo!()
        }
    }

    impl ClientStateInitializer<HostConsensusState> for FirstClientState {
        fn initialise(&self, _consensus_state: Any) -> Result<HostConsensusState, ClientError> {
            todo!()
        }
    }

    impl ClientStateValidation<ClientValidationContext> for FirstClientState {
        fn verify_client_message(
            &self,
            _ctx: &ClientValidationContext,
            _client_id: &ClientId,
            _client_message: Any,
            _update_kind: &UpdateKind,
        ) -> Result<(), ClientError> {
            todo!()
        }

        fn check_for_misbehaviour(
            &self,
            _ctx: &ClientValidationContext,
            _client_id: &ClientId,
            _client_message: Any,
            _update_kind: &UpdateKind,
        ) -> Result<bool, ClientError> {
            todo!()
        }
    }

    impl ClientStateExecution<ClientExecutionContext> for FirstClientState {
        fn update_state(
            &self,
            _ctx: &mut ClientExecutionContext,
            _client_id: &ClientId,
            _header: Any,
        ) -> Result<Vec<Height>, ClientError> {
            todo!()
        }

        fn update_state_on_misbehaviour(
            &self,
            _ctx: &mut ClientExecutionContext,
            _client_id: &ClientId,
            _client_message: Any,
            _update_kind: &UpdateKind,
        ) -> Result<(), ClientError> {
            todo!()
        }

        fn update_state_with_upgrade_client(
            &self,
            _ctx: &mut ClientExecutionContext,
            _client_id: &ClientId,
            _upgraded_client_state: Any,
            _upgraded_consensus_state: Any,
        ) -> Result<Height, ClientError> {
            todo!()
        }
    }
}
