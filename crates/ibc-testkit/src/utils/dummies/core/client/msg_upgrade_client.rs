use ibc::core::client::types::msgs::MsgUpgradeClient;
use ibc::core::client::types::proto::v1::MsgUpgradeClient as RawMsgUpgradeClient;
use ibc::core::client::types::Height;
use ibc::core::host::types::identifiers::ClientId;

use crate::testapp::ibc::clients::mock::client_state::MockClientState;
use crate::testapp::ibc::clients::mock::consensus_state::MockConsensusState;
use crate::testapp::ibc::clients::mock::header::MockHeader;
use crate::utils::dummies::core::commitment::dummy_commitment_proof_bytes;
use crate::utils::dummies::core::signer::{dummy_account_id, dummy_bech32_account};

/// Returns a dummy `MsgUpgradeClient`, for testing purposes only!
pub fn dummy_msg_upgrade_client(client_id: ClientId, upgrade_height: Height) -> MsgUpgradeClient {
    MsgUpgradeClient {
        client_id,
        upgraded_client_state: MockClientState::new(MockHeader::new(upgrade_height)).into(),
        upgraded_consensus_state: MockConsensusState::new(MockHeader::new(upgrade_height)).into(),
        proof_upgrade_client: dummy_commitment_proof_bytes(),
        proof_upgrade_consensus_state: dummy_commitment_proof_bytes(),
        signer: dummy_account_id(),
    }
}

/// Returns a dummy `RawMsgUpgradeClient`, for testing purposes only!
pub fn dummy_raw_msg_upgrade_client() -> RawMsgUpgradeClient {
    let client_id = "07-tendermint-0".parse().unwrap();

    let upgrade_height = Height::new(0, 10).expect("Never fails");

    RawMsgUpgradeClient {
        client_id,
        client_state: Some(MockClientState::new(MockHeader::new(upgrade_height)).into()),
        consensus_state: Some(MockConsensusState::new(MockHeader::new(upgrade_height)).into()),
        proof_upgrade_client: dummy_commitment_proof_bytes().into(),
        proof_upgrade_consensus_state: dummy_commitment_proof_bytes().into(),
        signer: dummy_bech32_account(),
    }
}
