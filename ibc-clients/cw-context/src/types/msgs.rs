//! Defines the messages sent to the CosmWasm contract by the 08-wasm proxy
//! light client.
use std::str::FromStr;

use cosmwasm_schema::{cw_serde, QueryResponses};
use ibc_client_wasm_types::serializer::Base64;
use ibc_client_wasm_types::Bytes;
use ibc_core::client::types::proto::v1::Height as RawHeight;
use ibc_core::client::types::Height;
use ibc_core::commitment_types::commitment::{CommitmentPrefix, CommitmentProofBytes};
use ibc_core::host::types::path::Path;
use ibc_core::primitives::proto::Any;
use prost::Message;

use super::error::ContractError;

// ------------------------------------------------------------
// Implementation of the InstantiateMsg struct
// ------------------------------------------------------------

#[cw_serde]
pub struct InstantiateMsg {
    #[schemars(with = "String")]
    #[serde(with = "Base64", default)]
    pub client_state: Bytes,
    #[schemars(with = "String")]
    #[serde(with = "Base64", default)]
    pub consensus_state: Bytes,
    #[schemars(with = "String")]
    #[serde(with = "Base64", default)]
    pub checksum: Bytes,
}

// ------------------------------------------------------------
// Implementation of the SudoMsg enum and its variants
// ------------------------------------------------------------

#[derive(derive_more::From)]
#[cw_serde]
pub enum SudoMsg {
    UpdateState(UpdateStateMsgRaw),
    UpdateStateOnMisbehaviour(UpdateStateOnMisbehaviourMsgRaw),
    VerifyUpgradeAndUpdateState(VerifyUpgradeAndUpdateStateMsgRaw),
    VerifyMembership(VerifyMembershipMsgRaw),
    VerifyNonMembership(VerifyNonMembershipMsgRaw),
    MigrateClientStore(MigrateClientStoreMsg),
}

#[cw_serde]
pub struct UpdateStateOnMisbehaviourMsgRaw {
    #[schemars(with = "String")]
    #[serde(with = "Base64", default)]
    pub client_message: Bytes,
}

pub struct UpdateStateOnMisbehaviourMsg {
    pub client_message: Any,
}

impl TryFrom<UpdateStateOnMisbehaviourMsgRaw> for UpdateStateOnMisbehaviourMsg {
    type Error = ContractError;

    fn try_from(raw: UpdateStateOnMisbehaviourMsgRaw) -> Result<Self, Self::Error> {
        let client_message = Any::decode(&mut raw.client_message.as_slice())?;

        Ok(Self { client_message })
    }
}

#[cw_serde]
pub struct UpdateStateMsgRaw {
    #[schemars(with = "String")]
    #[serde(with = "Base64", default)]
    pub client_message: Bytes,
}

pub struct UpdateStateMsg {
    pub client_message: Any,
}

impl TryFrom<UpdateStateMsgRaw> for UpdateStateMsg {
    type Error = ContractError;

    fn try_from(raw: UpdateStateMsgRaw) -> Result<Self, Self::Error> {
        let client_message = Any::decode(&mut raw.client_message.as_slice())?;

        Ok(Self { client_message })
    }
}

#[cw_serde]
pub struct CheckSubstituteAndUpdateStateMsg {}

#[cw_serde]
pub struct VerifyUpgradeAndUpdateStateMsgRaw {
    #[schemars(with = "String")]
    #[serde(with = "Base64", default)]
    pub upgrade_client_state: Bytes,
    #[schemars(with = "String")]
    #[serde(with = "Base64", default)]
    pub upgrade_consensus_state: Bytes,
    #[schemars(with = "String")]
    #[serde(with = "Base64", default)]
    pub proof_upgrade_client: Bytes,
    #[schemars(with = "String")]
    #[serde(with = "Base64", default)]
    pub proof_upgrade_consensus_state: Bytes,
}

pub struct VerifyUpgradeAndUpdateStateMsg {
    pub upgrade_client_state: Any,
    pub upgrade_consensus_state: Any,
    pub proof_upgrade_client: CommitmentProofBytes,
    pub proof_upgrade_consensus_state: CommitmentProofBytes,
}

impl TryFrom<VerifyUpgradeAndUpdateStateMsgRaw> for VerifyUpgradeAndUpdateStateMsg {
    type Error = ContractError;

    fn try_from(raw: VerifyUpgradeAndUpdateStateMsgRaw) -> Result<Self, Self::Error> {
        let upgrade_client_state = Any::decode(&mut raw.upgrade_client_state.as_slice())?;

        let upgrade_consensus_state = Any::decode(&mut raw.upgrade_consensus_state.as_slice())?;

        Ok(VerifyUpgradeAndUpdateStateMsg {
            upgrade_client_state,
            upgrade_consensus_state,
            proof_upgrade_client: CommitmentProofBytes::try_from(raw.proof_upgrade_client)?,
            proof_upgrade_consensus_state: CommitmentProofBytes::try_from(
                raw.proof_upgrade_consensus_state,
            )?,
        })
    }
}

#[cw_serde]
pub struct MerklePath {
    pub key_path: Vec<String>,
}

#[cw_serde]
pub struct VerifyMembershipMsgRaw {
    #[schemars(with = "String")]
    #[serde(with = "Base64", default)]
    pub proof: Bytes,
    pub path: MerklePath,
    #[schemars(with = "String")]
    #[serde(with = "Base64", default)]
    pub value: Bytes,
    pub height: RawHeight,
    pub delay_block_period: u64,
    pub delay_time_period: u64,
}

pub struct VerifyMembershipMsg {
    pub prefix: CommitmentPrefix,
    pub proof: CommitmentProofBytes,
    pub path: Path,
    pub value: Vec<u8>,
    pub height: Height,
    pub delay_block_period: u64,
    pub delay_time_period: u64,
}

impl TryFrom<VerifyMembershipMsgRaw> for VerifyMembershipMsg {
    type Error = ContractError;

    fn try_from(mut raw: VerifyMembershipMsgRaw) -> Result<Self, Self::Error> {
        let proof = CommitmentProofBytes::try_from(raw.proof)?;
        let prefix = raw.path.key_path.remove(0).into_bytes();
        let path_str = raw.path.key_path.join("");
        let path = Path::from_str(&path_str)?;
        let height = Height::try_from(raw.height)?;

        Ok(Self {
            proof,
            path,
            value: raw.value,
            height,
            prefix: CommitmentPrefix::try_from(prefix)?,
            delay_block_period: raw.delay_block_period,
            delay_time_period: raw.delay_time_period,
        })
    }
}

#[cw_serde]
pub struct VerifyNonMembershipMsgRaw {
    #[schemars(with = "String")]
    #[serde(with = "Base64", default)]
    pub proof: Bytes,
    pub path: MerklePath,
    pub height: RawHeight,
    pub delay_block_period: u64,
    pub delay_time_period: u64,
}

pub struct VerifyNonMembershipMsg {
    pub prefix: CommitmentPrefix,
    pub proof: CommitmentProofBytes,
    pub path: Path,
    pub height: Height,
    pub delay_block_period: u64,
    pub delay_time_period: u64,
}

impl TryFrom<VerifyNonMembershipMsgRaw> for VerifyNonMembershipMsg {
    type Error = ContractError;

    fn try_from(mut raw: VerifyNonMembershipMsgRaw) -> Result<Self, Self::Error> {
        let proof = CommitmentProofBytes::try_from(raw.proof)?;
        let prefix = raw.path.key_path.remove(0).into_bytes();
        let path_str = raw.path.key_path.join("");
        let path = Path::from_str(&path_str)?;
        let height = raw.height.try_into()?;
        Ok(Self {
            proof,
            path,
            height,
            prefix: CommitmentPrefix::try_from(prefix)?,
            delay_block_period: raw.delay_block_period,
            delay_time_period: raw.delay_time_period,
        })
    }
}

#[cw_serde]
pub struct MigrateClientStoreMsg {}

// ------------------------------------------------------------
// Implementation of the QueryMsg enum and its variants
// ------------------------------------------------------------

#[derive(QueryResponses, derive_more::From)]
#[cw_serde]
pub enum QueryMsg {
    #[returns(crate::types::response::QueryResponse)]
    Status(StatusMsg),
    #[returns(crate::types::response::QueryResponse)]
    ExportMetadata(ExportMetadataMsg),
    #[returns(crate::types::response::QueryResponse)]
    TimestampAtHeight(TimestampAtHeightMsg),
    #[returns(crate::types::response::QueryResponse)]
    VerifyClientMessage(VerifyClientMessageRaw),
    #[returns(crate::types::response::QueryResponse)]
    CheckForMisbehaviour(CheckForMisbehaviourMsgRaw),
}

#[cw_serde]
pub struct StatusMsg {}

#[cw_serde]
pub struct ExportMetadataMsg {}

#[cw_serde]
pub struct TimestampAtHeightMsg {
    pub height: Height,
}

#[cw_serde]
pub struct VerifyClientMessageRaw {
    #[schemars(with = "String")]
    #[serde(with = "Base64", default)]
    pub client_message: Bytes,
}

pub struct VerifyClientMessageMsg {
    pub client_message: Any,
}

impl TryFrom<VerifyClientMessageRaw> for VerifyClientMessageMsg {
    type Error = ContractError;

    fn try_from(raw: VerifyClientMessageRaw) -> Result<Self, Self::Error> {
        let client_message = Any::decode(&mut raw.client_message.as_slice())?;

        Ok(Self { client_message })
    }
}

#[cw_serde]
pub struct CheckForMisbehaviourMsgRaw {
    #[schemars(with = "String")]
    #[serde(with = "Base64", default)]
    pub client_message: Bytes,
}

pub struct CheckForMisbehaviourMsg {
    pub client_message: Any,
}

impl TryFrom<CheckForMisbehaviourMsgRaw> for CheckForMisbehaviourMsg {
    type Error = ContractError;

    fn try_from(raw: CheckForMisbehaviourMsgRaw) -> Result<Self, Self::Error> {
        let client_message = Any::decode(&mut raw.client_message.as_slice())?;

        Ok(Self { client_message })
    }
}
