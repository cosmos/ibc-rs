use ibc_client_tendermint_types::{client_type as tm_client_type, ClientState as ClientStateType};
use ibc_core_client::context::client_state::ClientStateCommon;
use ibc_core_client::context::consensus_state::ConsensusState;
use ibc_core_client::types::error::{ClientError, UpgradeClientError};
use ibc_core_client::types::Height;
use ibc_core_commitment_types::commitment::{
    CommitmentPrefix, CommitmentProofBytes, CommitmentRoot,
};
use ibc_core_commitment_types::merkle::{apply_prefix, MerkleProof};
use ibc_core_commitment_types::proto::ics23::{HostFunctionsManager, HostFunctionsProvider};
use ibc_core_commitment_types::specs::ProofSpecs;
use ibc_core_host::types::identifiers::ClientType;
use ibc_core_host::types::path::{Path, UpgradeClientPath};
use ibc_primitives::prelude::*;
use ibc_primitives::proto::Any;
use ibc_primitives::ToVec;

use super::ClientState;
use crate::consensus_state::ConsensusState as TmConsensusState;

impl ClientStateCommon for ClientState {
    fn verify_consensus_state(&self, consensus_state: Any) -> Result<(), ClientError> {
        verify_consensus_state(consensus_state)
    }

    fn client_type(&self) -> ClientType {
        tm_client_type()
    }

    fn latest_height(&self) -> Height {
        self.0.latest_height
    }

    fn validate_proof_height(&self, proof_height: Height) -> Result<(), ClientError> {
        validate_proof_height(self.inner(), proof_height)
    }

    fn verify_upgrade_client(
        &self,
        upgraded_client_state: Any,
        upgraded_consensus_state: Any,
        proof_upgrade_client: CommitmentProofBytes,
        proof_upgrade_consensus_state: CommitmentProofBytes,
        root: &CommitmentRoot,
    ) -> Result<(), ClientError> {
        verify_upgrade_client::<HostFunctionsManager>(
            self.inner(),
            upgraded_client_state,
            upgraded_consensus_state,
            proof_upgrade_client,
            proof_upgrade_consensus_state,
            root,
        )
    }

    fn verify_membership(
        &self,
        prefix: &CommitmentPrefix,
        proof: &CommitmentProofBytes,
        root: &CommitmentRoot,
        path: Path,
        value: Vec<u8>,
    ) -> Result<(), ClientError> {
        verify_membership::<HostFunctionsManager>(
            &self.inner().proof_specs,
            prefix,
            proof,
            root,
            path,
            value,
        )
    }

    fn verify_non_membership(
        &self,
        prefix: &CommitmentPrefix,
        proof: &CommitmentProofBytes,
        root: &CommitmentRoot,
        path: Path,
    ) -> Result<(), ClientError> {
        verify_non_membership::<HostFunctionsManager>(
            &self.inner().proof_specs,
            prefix,
            proof,
            root,
            path,
        )
    }
}

/// Verify an `Any` consensus state by attempting to convert it to a `TmConsensusState`.
/// Also checks whether the converted consensus state's root is present.
///
/// Note that this function is typically implemented as part of the
/// [`ClientStateCommon`] trait, but has been made a standalone function
/// in order to make the ClientState APIs more flexible.
pub fn verify_consensus_state(consensus_state: Any) -> Result<(), ClientError> {
    let tm_consensus_state = TmConsensusState::try_from(consensus_state)?;

    if tm_consensus_state.root().is_empty() {
        return Err(ClientError::Other {
            description: "empty commitment root".into(),
        });
    };

    Ok(())
}

/// Validate the given proof height against the client state's latest height, returning
/// an error if the proof height is greater than the latest height of the client state.
///
/// Note that this function is typically implemented as part of the
/// [`ClientStateCommon`] trait, but has been made a standalone function
/// in order to make the ClientState APIs more flexible.
pub fn validate_proof_height(
    client_state: &ClientStateType,
    proof_height: Height,
) -> Result<(), ClientError> {
    let latest_height = client_state.latest_height;

    if latest_height < proof_height {
        return Err(ClientError::InvalidProofHeight {
            latest_height,
            proof_height,
        });
    }

    Ok(())
}

/// Perform client-specific verifications and check all data in the new
/// client state to be the same across all valid Tendermint clients for the
/// new chain.
///
/// You can learn more about how to upgrade IBC-connected SDK chains in
/// [this](https://ibc.cosmos.network/main/ibc/upgrades/quick-guide.html)
/// guide.
///
/// Note that this function is typically implemented as part of the
/// [`ClientStateCommon`] trait, but has been made a standalone function
/// in order to make the ClientState APIs more flexible.
pub fn verify_upgrade_client<H: HostFunctionsProvider>(
    client_state: &ClientStateType,
    upgraded_client_state: Any,
    upgraded_consensus_state: Any,
    proof_upgrade_client: CommitmentProofBytes,
    proof_upgrade_consensus_state: CommitmentProofBytes,
    root: &CommitmentRoot,
) -> Result<(), ClientError> {
    // Make sure that the client type is of Tendermint type `ClientState`
    let upgraded_tm_client_state = ClientState::try_from(upgraded_client_state.clone())?;

    // Make sure that the consensus type is of Tendermint type `ConsensusState`
    TmConsensusState::try_from(upgraded_consensus_state.clone())?;

    let latest_height = client_state.latest_height;
    let upgraded_tm_client_state_height = upgraded_tm_client_state.latest_height();

    // Make sure the latest height of the current client is not greater then
    // the upgrade height This condition checks both the revision number and
    // the height
    if latest_height >= upgraded_tm_client_state_height {
        Err(UpgradeClientError::LowUpgradeHeight {
            upgraded_height: latest_height,
            client_height: upgraded_tm_client_state_height,
        })?
    }

    // Check to see if the upgrade path is set
    let mut upgrade_path = client_state.upgrade_path.clone();

    if upgrade_path.pop().is_none() {
        return Err(ClientError::ClientSpecific {
            description: "cannot upgrade client as no upgrade path has been set".to_string(),
        });
    };

    let upgrade_path_prefix = CommitmentPrefix::try_from(upgrade_path[0].clone().into_bytes())
        .map_err(ClientError::InvalidCommitmentProof)?;

    let last_height = latest_height.revision_height();

    // Verify the proof of the upgraded client state
    verify_membership::<H>(
        &client_state.proof_specs,
        &upgrade_path_prefix,
        &proof_upgrade_client,
        root,
        Path::UpgradeClient(UpgradeClientPath::UpgradedClientState(last_height)),
        upgraded_client_state.to_vec(),
    )?;

    // Verify the proof of the upgraded consensus state
    verify_membership::<H>(
        &client_state.proof_specs,
        &upgrade_path_prefix,
        &proof_upgrade_consensus_state,
        root,
        Path::UpgradeClient(UpgradeClientPath::UpgradedClientConsensusState(last_height)),
        upgraded_consensus_state.to_vec(),
    )?;

    Ok(())
}

/// Verify membership of the given value against the client's merkle proof.
///
/// Note that this function is typically implemented as part of the
/// [`ClientStateCommon`] trait, but has been made a standalone function
/// in order to make the ClientState APIs more flexible.
pub fn verify_membership<H: HostFunctionsProvider>(
    proof_specs: &ProofSpecs,
    prefix: &CommitmentPrefix,
    proof: &CommitmentProofBytes,
    root: &CommitmentRoot,
    path: Path,
    value: Vec<u8>,
) -> Result<(), ClientError> {
    let merkle_path = apply_prefix(prefix, vec![path.to_string()]);
    let merkle_proof = MerkleProof::try_from(proof).map_err(ClientError::InvalidCommitmentProof)?;

    merkle_proof
        .verify_membership::<H>(proof_specs, root.clone().into(), merkle_path, value, 0)
        .map_err(ClientError::Ics23Verification)
}

/// Verify that the given value does not belong in the client's merkle proof.
///
/// Note that this function is typically implemented as part of the
/// [`ClientStateCommon`] trait, but has been made a standalone function
/// in order to make the ClientState APIs more flexible.
pub fn verify_non_membership<H: HostFunctionsProvider>(
    proof_specs: &ProofSpecs,
    prefix: &CommitmentPrefix,
    proof: &CommitmentProofBytes,
    root: &CommitmentRoot,
    path: Path,
) -> Result<(), ClientError> {
    let merkle_path = apply_prefix(prefix, vec![path.to_string()]);
    let merkle_proof = MerkleProof::try_from(proof).map_err(ClientError::InvalidCommitmentProof)?;

    merkle_proof
        .verify_non_membership::<H>(proof_specs, root.clone().into(), merkle_path)
        .map_err(ClientError::Ics23Verification)
}
