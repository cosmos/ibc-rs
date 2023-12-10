//! Definition of domain type msg `MsgUpgradeClient`.

use core::str::FromStr;

use ibc_core_commitment_types::commitment::CommitmentProofBytes;
use ibc_core_commitment_types::error::CommitmentError;
use ibc_core_host_types::identifiers::ClientId;
use ibc_primitives::prelude::*;
use ibc_primitives::Signer;
use ibc_proto::google::protobuf::Any;
use ibc_proto::ibc::core::client::v1::MsgUpgradeClient as RawMsgUpgradeClient;
use ibc_proto::Protobuf;

use crate::error::{ClientError, UpgradeClientError};

pub const UPGRADE_CLIENT_TYPE_URL: &str = "/ibc.core.client.v1.MsgUpgradeClient";

/// A type of message that triggers the upgrade of an on-chain (IBC) client.
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, PartialEq, Eq)]
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
