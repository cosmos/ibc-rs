//! Definition of domain type msg `MsgUpgradeClient`.

use crate::prelude::*;

use core::str::FromStr;

use ibc_proto::google::protobuf::Any;
use ibc_proto::ibc::core::client::v1::MsgUpgradeClient as RawMsgUpgradeClient;
use ibc_proto::protobuf::Protobuf;

use crate::core::ics02_client::error::{ClientError, UpgradeClientError};
use crate::core::ics23_commitment::commitment::CommitmentProofBytes;
use crate::core::ics23_commitment::error::CommitmentError;
use crate::core::ics24_host::identifier::ClientId;
use crate::core::Msg;
use crate::signer::Signer;

pub(crate) const TYPE_URL: &str = "/ibc.core.client.v1.MsgUpgradeClient";

/// A type of message that triggers the upgrade of an on-chain (IBC) client.
#[derive(Clone, Debug, PartialEq)]
pub struct MsgUpgradeClient {
    // client unique identifier
    pub client_id: ClientId,
    // Upgraded client state
    pub upgraded_client_state: Any,
    // Upgraded consensus state, only contains enough information
    // to serve as a basis of trust in update logic
    pub upgraded_consensus_state: Any,
    // proof that old chain committed to new client
    pub proof_upgrade_client: CommitmentProofBytes,
    // proof that old chain committed to new consensus state
    pub proof_upgrade_consensus_state: CommitmentProofBytes,
    // signer address
    pub signer: Signer,
}

impl Msg for MsgUpgradeClient {
    type Raw = RawMsgUpgradeClient;

    fn type_url(&self) -> String {
        TYPE_URL.to_string()
    }
}

impl Protobuf<RawMsgUpgradeClient> for MsgUpgradeClient {}

impl From<MsgUpgradeClient> for RawMsgUpgradeClient {
    fn from(dm_msg: MsgUpgradeClient) -> RawMsgUpgradeClient {
        RawMsgUpgradeClient {
            client_id: dm_msg.client_id.to_string(),
            client_state: Some(dm_msg.upgraded_client_state),
            consensus_state: Some(dm_msg.upgraded_consensus_state),
            proof_upgrade_client: dm_msg.proof_upgrade_client.into(),
            proof_upgrade_consensus_state: dm_msg.proof_upgrade_consensus_state.into(),
            signer: dm_msg.signer.to_string(),
        }
    }
}

impl TryFrom<RawMsgUpgradeClient> for MsgUpgradeClient {
    type Error = ClientError;

    fn try_from(proto_msg: RawMsgUpgradeClient) -> Result<Self, Self::Error> {
        let raw_client_state = proto_msg
            .client_state
            .ok_or(ClientError::MissingRawClientState)?;

        let raw_consensus_state = proto_msg
            .consensus_state
            .ok_or(ClientError::MissingRawConsensusState)?;

        let c_bytes =
            CommitmentProofBytes::try_from(proto_msg.proof_upgrade_client).map_err(|_| {
                UpgradeClientError::InvalidUpgradeClientProof(CommitmentError::EmptyMerkleProof)
            })?;
        let cs_bytes = CommitmentProofBytes::try_from(proto_msg.proof_upgrade_consensus_state)
            .map_err(|_| {
                UpgradeClientError::InvalidUpgradeConsensusStateProof(
                    CommitmentError::EmptyMerkleProof,
                )
            })?;

        Ok(MsgUpgradeClient {
            client_id: ClientId::from_str(&proto_msg.client_id)
                .map_err(ClientError::InvalidClientIdentifier)?,
            upgraded_client_state: raw_client_state,
            upgraded_consensus_state: raw_consensus_state,
            proof_upgrade_client: c_bytes,
            proof_upgrade_consensus_state: cs_bytes,
            signer: proto_msg.signer.into(),
        })
    }
}

#[cfg(test)]
pub mod test_util {
    use super::*;

    use crate::core::ics23_commitment::commitment::test_util::get_dummy_commitment_proof_bytes;
    use crate::core::{ics02_client::height::Height, ics24_host::identifier::ClientId};
    use crate::mock::client_state::client_type as mock_client_type;
    use crate::mock::{
        client_state::MockClientState, consensus_state::MockConsensusState, header::MockHeader,
    };
    use crate::test_utils::{get_dummy_account_id, get_dummy_bech32_account, get_dummy_proof};

    /// Extends the implementation with additional helper methods.
    impl MsgUpgradeClient {
        pub fn new_dummy(upgrade_height: Height) -> Self {
            MsgUpgradeClient {
                client_id: ClientId::new(mock_client_type(), 0).unwrap(),
                upgraded_client_state: MockClientState::new(MockHeader::new(upgrade_height)).into(),
                upgraded_consensus_state: MockConsensusState::new(MockHeader::new(upgrade_height))
                    .into(),
                proof_upgrade_client: get_dummy_commitment_proof_bytes(),
                proof_upgrade_consensus_state: get_dummy_commitment_proof_bytes(),
                signer: get_dummy_account_id(),
            }
        }

        pub fn with_client_id(self, client_id: ClientId) -> Self {
            MsgUpgradeClient { client_id, ..self }
        }
    }

    /// Returns a dummy `RawMsgUpgradeClient`, for testing only!
    pub fn get_dummy_raw_msg_upgrade_client(upgrade_height: Height) -> RawMsgUpgradeClient {
        RawMsgUpgradeClient {
            client_id: mock_client_type().to_string(),
            client_state: Some(MockClientState::new(MockHeader::new(upgrade_height)).into()),
            consensus_state: Some(MockConsensusState::new(MockHeader::new(upgrade_height)).into()),
            proof_upgrade_client: get_dummy_proof(),
            proof_upgrade_consensus_state: get_dummy_proof(),
            signer: get_dummy_bech32_account(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::ics02_client::height::Height;

    #[test]
    fn msg_upgrade_client_serialization() {
        let msg = MsgUpgradeClient::new_dummy(Height::new(1, 1).unwrap());
        let raw: RawMsgUpgradeClient = RawMsgUpgradeClient::from(msg.clone());
        let msg_back = MsgUpgradeClient::try_from(raw.clone()).unwrap();
        let raw_back: RawMsgUpgradeClient = RawMsgUpgradeClient::from(msg_back.clone());
        assert_eq!(msg, msg_back);
        assert_eq!(raw, raw_back);
    }
}
