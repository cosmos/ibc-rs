use ibc::core::client::types::msgs::MsgUpgradeClient;
use ibc::core::client::types::proto::v1::MsgUpgradeClient as RawMsgUpgradeClient;
use ibc::core::client::types::Height;
use ibc::core::host::types::identifiers::ClientId;

use crate::fixtures::core::commitment::dummy_commitment_proof_bytes;
use crate::fixtures::core::signer::{dummy_account_id, dummy_bech32_account};
use crate::testapp::ibc::clients::mock::client_state::MockClientState;
use crate::testapp::ibc::clients::mock::consensus_state::MockConsensusState;
use crate::testapp::ibc::clients::mock::header::MockHeader;

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
    let client_id = "07-tendermint-0".parse().expect("no error");

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn msg_upgrade_client_serialization() {
        let raw = dummy_raw_msg_upgrade_client();
        let msg = MsgUpgradeClient::try_from(raw.clone()).unwrap();
        let raw_back: RawMsgUpgradeClient = RawMsgUpgradeClient::from(msg.clone());
        let msg_back = MsgUpgradeClient::try_from(raw_back.clone()).unwrap();
        assert_eq!(msg, msg_back);
        assert_eq!(raw, raw_back);
    }
}
