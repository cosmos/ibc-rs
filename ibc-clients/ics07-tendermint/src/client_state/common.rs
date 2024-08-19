use core::time::Duration;

use ibc_client_tendermint_types::{client_type as tm_client_type, ClientState as ClientStateType};
use ibc_core_client::context::client_state::ClientStateCommon;
use ibc_core_client::context::consensus_state::ConsensusState;
use ibc_core_client::types::error::{ClientError, UpgradeClientError};
use ibc_core_client::types::{Height, Status};
use ibc_core_commitment_types::commitment::{
    CommitmentPrefix, CommitmentProofBytes, CommitmentRoot,
};
use ibc_core_commitment_types::error::CommitmentError;
use ibc_core_commitment_types::merkle::{MerklePath, MerkleProof};
use ibc_core_commitment_types::proto::ics23::{HostFunctionsManager, HostFunctionsProvider};
use ibc_core_commitment_types::specs::ProofSpecs;
use ibc_core_host::types::identifiers::ClientType;
use ibc_core_host::types::path::{
    Path, PathBytes, UpgradeClientStatePath, UpgradeConsensusStatePath,
};
use ibc_primitives::prelude::*;
use ibc_primitives::proto::Any;
use ibc_primitives::{Timestamp, ToVec};

use super::ClientState;
use crate::consensus_state::ConsensusState as TmConsensusState;

impl ClientStateCommon for ClientState {
    fn verify_consensus_state(
        &self,
        consensus_state: Any,
        host_timestamp: &Timestamp,
    ) -> Result<(), ClientError> {
        verify_consensus_state(
            consensus_state,
            host_timestamp,
            self.inner().trusting_period,
        )
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
        let last_height = self.latest_height().revision_height();

        // The client state's upgrade path vector needs to parsed into a tuple in the form
        // of `(upgrade_path_prefix, upgrade_path)`. Given the length of the client
        // state's upgrade path vector, the following determinations are made:
        // 1: The commitment prefix is left empty and the upgrade path is used as-is.
        // 2: The commitment prefix and upgrade path are both taken as-is.
        let upgrade_path = &self.inner().upgrade_path;
        let (upgrade_path_prefix, upgrade_path) = match upgrade_path.len() {
            0 => {
                return Err(UpgradeClientError::InvalidUpgradePath {
                    reason: "no upgrade path has been set".to_string(),
                }
                .into());
            }
            1 => (CommitmentPrefix::empty(), upgrade_path[0].clone()),
            2 => (
                upgrade_path[0].as_bytes().to_vec().into(),
                upgrade_path[1].clone(),
            ),
            _ => {
                return Err(UpgradeClientError::InvalidUpgradePath {
                    reason: "upgrade path is too long".to_string(),
                }
                .into());
            }
        };

        let upgrade_client_path_bytes =
            self.serialize_path(Path::UpgradeClientState(UpgradeClientStatePath {
                upgrade_path: upgrade_path.clone(),
                height: last_height,
            }))?;

        let upgrade_consensus_path_bytes =
            self.serialize_path(Path::UpgradeConsensusState(UpgradeConsensusStatePath {
                upgrade_path,
                height: last_height,
            }))?;

        verify_upgrade_client::<HostFunctionsManager>(
            self.inner(),
            upgraded_client_state,
            upgraded_consensus_state,
            proof_upgrade_client,
            proof_upgrade_consensus_state,
            upgrade_path_prefix,
            upgrade_client_path_bytes,
            upgrade_consensus_path_bytes,
            root,
        )
    }

    fn serialize_path(&self, path: Path) -> Result<PathBytes, ClientError> {
        Ok(path.to_string().into_bytes().into())
    }

    fn verify_membership_raw(
        &self,
        prefix: &CommitmentPrefix,
        proof: &CommitmentProofBytes,
        root: &CommitmentRoot,
        path: PathBytes,
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

    fn verify_non_membership_raw(
        &self,
        prefix: &CommitmentPrefix,
        proof: &CommitmentProofBytes,
        root: &CommitmentRoot,
        path: PathBytes,
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
pub fn verify_consensus_state(
    consensus_state: Any,
    host_timestamp: &Timestamp,
    trusting_period: Duration,
) -> Result<(), ClientError> {
    let tm_consensus_state = TmConsensusState::try_from(consensus_state)?;

    if tm_consensus_state.root().is_empty() {
        return Err(ClientError::Other {
            description: "empty commitment root".into(),
        });
    };

    if consensus_state_status(&tm_consensus_state, host_timestamp, trusting_period)?.is_expired() {
        return Err(ClientError::InvalidStatus(Status::Expired));
    }

    Ok(())
}

/// Determines the `Status`, whether it is `Active` or `Expired`, of a consensus
/// state, using its timestamp, the host's timestamp, and the trusting period.
pub fn consensus_state_status<CS: ConsensusState>(
    consensus_state: &CS,
    host_timestamp: &Timestamp,
    trusting_period: Duration,
) -> Result<Status, ClientError> {
    // Note: if the `duration_since()` is `None`, indicating that the latest
    // consensus state is in the future, then we don't consider the client
    // to be expired.
    if let Some(elapsed_since_latest_consensus_state) =
        host_timestamp.duration_since(&consensus_state.timestamp())
    {
        // Note: The equality is considered as expired to stay consistent with
        // the check in tendermint-rs, where a header at `trusted_header_time +
        // trusting_period` is considered expired.
        if elapsed_since_latest_consensus_state >= trusting_period {
            return Ok(Status::Expired);
        }
    }

    Ok(Status::Active)
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
            actual: latest_height,
            expected: proof_height,
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
#[allow(clippy::too_many_arguments)]
pub fn verify_upgrade_client<H: HostFunctionsProvider>(
    client_state: &ClientStateType,
    upgraded_client_state: Any,
    upgraded_consensus_state: Any,
    proof_upgrade_client: CommitmentProofBytes,
    proof_upgrade_consensus_state: CommitmentProofBytes,
    upgrade_path_prefix: CommitmentPrefix,
    upgrade_client_path_bytes: PathBytes,
    upgrade_consensus_path_bytes: PathBytes,
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
            upgraded_height: upgraded_tm_client_state_height,
            client_height: latest_height,
        })?
    }

    // Verify the proof of the upgraded client state
    verify_membership::<H>(
        &client_state.proof_specs,
        &upgrade_path_prefix,
        &proof_upgrade_client,
        root,
        upgrade_client_path_bytes,
        upgraded_client_state.to_vec(),
    )?;

    // Verify the proof of the upgraded consensus state
    verify_membership::<H>(
        &client_state.proof_specs,
        &upgrade_path_prefix,
        &proof_upgrade_consensus_state,
        root,
        upgrade_consensus_path_bytes,
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
    path: PathBytes,
    value: Vec<u8>,
) -> Result<(), ClientError> {
    if prefix.is_empty() {
        return Err(ClientError::FailedIcs23Verification(
            CommitmentError::EmptyCommitmentPrefix,
        ));
    }

    let merkle_path = MerklePath::new(vec![prefix.as_bytes().to_vec().into(), path]);
    let merkle_proof =
        MerkleProof::try_from(proof).map_err(ClientError::FailedIcs23Verification)?;

    merkle_proof
        .verify_membership::<H>(proof_specs, root.clone().into(), merkle_path, value, 0)
        .map_err(ClientError::FailedIcs23Verification)
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
    path: PathBytes,
) -> Result<(), ClientError> {
    let merkle_path = MerklePath::new(vec![prefix.as_bytes().to_vec().into(), path]);
    let merkle_proof =
        MerkleProof::try_from(proof).map_err(ClientError::FailedIcs23Verification)?;

    merkle_proof
        .verify_non_membership::<H>(proof_specs, root.clone().into(), merkle_path)
        .map_err(ClientError::FailedIcs23Verification)
}
