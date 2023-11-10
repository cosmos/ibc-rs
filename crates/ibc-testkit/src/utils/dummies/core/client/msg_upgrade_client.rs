use ibc::core::ics02_client::height::Height;
use ibc::core::ics02_client::msgs::upgrade_client::MsgUpgradeClient;
use ibc::core::ics24_host::identifier::ClientId;

use crate::testapp::ibc::clients::mock::client_state::MockClientState;
use crate::testapp::ibc::clients::mock::consensus_state::MockConsensusState;
use crate::testapp::ibc::clients::mock::header::MockHeader;
use crate::utils::dummies::core::commitment::dummy_commitment_proof_bytes;
use crate::utils::dummies::core::signer::dummy_account_id;

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
