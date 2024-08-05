//! Defines the messages sent to the CosmWasm contract by the 08-wasm proxy
//! light client.
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Binary;
use ibc_core::client::types::proto::v1::Height as RawHeight;
use ibc_core::client::types::Height;
use ibc_core::commitment_types::commitment::{CommitmentPrefix, CommitmentProofBytes};
use ibc_core::host::types::path::PathBytes;
use ibc_core::primitives::proto::Any;
use prost::Message;

use super::error::ContractError;

// ------------------------------------------------------------
// Implementation of the InstantiateMsg struct
// ------------------------------------------------------------

#[cw_serde]
pub struct InstantiateMsg {
    pub client_state: Binary,
    pub consensus_state: Binary,
    pub checksum: Binary,
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
    pub client_message: Binary,
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
    pub client_message: Binary,
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
    pub upgrade_client_state: Binary,
    pub upgrade_consensus_state: Binary,
    pub proof_upgrade_client: Binary,
    pub proof_upgrade_consensus_state: Binary,
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
            proof_upgrade_client: CommitmentProofBytes::try_from(
                raw.proof_upgrade_client.to_vec(),
            )?,
            proof_upgrade_consensus_state: CommitmentProofBytes::try_from(
                raw.proof_upgrade_consensus_state.to_vec(),
            )?,
        })
    }
}

#[cw_serde]
pub struct MerklePath {
    pub key_path: Vec<Binary>,
}

#[cw_serde]
pub struct VerifyMembershipMsgRaw {
    pub proof: Binary,
    pub path: MerklePath,
    pub value: Binary,
    pub height: RawHeight,
    pub delay_block_period: u64,
    pub delay_time_period: u64,
}

pub struct VerifyMembershipMsg {
    pub prefix: CommitmentPrefix,
    pub proof: CommitmentProofBytes,
    pub path: PathBytes,
    pub value: Vec<u8>,
    pub height: Height,
    pub delay_block_period: u64,
    pub delay_time_period: u64,
}

impl TryFrom<VerifyMembershipMsgRaw> for VerifyMembershipMsg {
    type Error = ContractError;

    fn try_from(mut raw: VerifyMembershipMsgRaw) -> Result<Self, Self::Error> {
        let proof = CommitmentProofBytes::try_from(raw.proof.to_vec())?;
        let prefix = CommitmentPrefix::from_bytes(raw.path.key_path.remove(0));
        let path = PathBytes::flatten(raw.path.key_path);
        let height = Height::try_from(raw.height)?;

        Ok(Self {
            proof,
            path,
            value: raw.value.into(),
            height,
            prefix,
            delay_block_period: raw.delay_block_period,
            delay_time_period: raw.delay_time_period,
        })
    }
}

#[cw_serde]
pub struct VerifyNonMembershipMsgRaw {
    pub proof: Binary,
    pub path: MerklePath,
    pub height: RawHeight,
    pub delay_block_period: u64,
    pub delay_time_period: u64,
}

pub struct VerifyNonMembershipMsg {
    pub prefix: CommitmentPrefix,
    pub proof: CommitmentProofBytes,
    pub path: PathBytes,
    pub height: Height,
    pub delay_block_period: u64,
    pub delay_time_period: u64,
}

impl TryFrom<VerifyNonMembershipMsgRaw> for VerifyNonMembershipMsg {
    type Error = ContractError;

    fn try_from(mut raw: VerifyNonMembershipMsgRaw) -> Result<Self, Self::Error> {
        let proof = CommitmentProofBytes::try_from(raw.proof.to_vec())?;
        let prefix = CommitmentPrefix::from_bytes(raw.path.key_path.remove(0));
        let path = PathBytes::flatten(raw.path.key_path);
        let height = raw.height.try_into()?;

        Ok(Self {
            proof,
            path,
            height,
            prefix,
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
    #[returns(crate::types::response::StatusResponse)]
    Status(StatusMsg),
    #[returns(crate::types::response::TimestampAtHeightResponse)]
    TimestampAtHeight(TimestampAtHeightMsg),
    #[returns(crate::types::response::VerifyClientMessageResponse)]
    VerifyClientMessage(VerifyClientMessageRaw),
    #[returns(crate::types::response::CheckForMisbehaviourResponse)]
    CheckForMisbehaviour(CheckForMisbehaviourMsgRaw),
}

#[cw_serde]
pub struct StatusMsg {}

#[cw_serde]
pub struct TimestampAtHeightMsg {
    pub height: Height,
}

#[cw_serde]
pub struct VerifyClientMessageRaw {
    pub client_message: Binary,
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
    pub client_message: Binary,
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

#[cfg(test)]
mod test {
    use super::{InstantiateMsg, SudoMsg};

    #[test]
    fn verify_membership_from_json() {
        let sudo_msg = r#"{
            "verify_membership":{
                "height":
                    {"revision_height":57},
                "delay_time_period":0,
                "delay_block_period":0,
                "proof":"CuECCt4CChhjb25uZWN0aW9ucy9jb25uZWN0aW9uLTASWgoPMDctdGVuZGVybWludC0wEiMKATESDU9SREVSX09SREVSRUQSD09SREVSX1VOT1JERVJFRBgCIiAKCTA4LXdhc20tMBIMY29ubmVjdGlvbi0wGgUKA2liYxoLCAEYASABKgMAAkgiKQgBEiUCBHAg3HTYmBAMxlr6u0mv6wCpm3ur2WQc7A3Af6aV7Ye0Fe0gIisIARIEBAZwIBohIHXEkQ9RIH08ZZYBIP6THxOOJiRmjXWGn1G4RCWT3V6rIisIARIEBgxwIBohIEUjGWV7YLPEzdFVLAb0lv4VvP7A+l1TqFkjpx1kDKAPIikIARIlCBhwILWsAKEot+2MknVyn5zcS0qsqVhRj4AHpgDx7fNPbfhtICIpCAESJQxAcCCzyYMGE+CdCltudr1ddHvCJrqv3kl/i7YnMLx3XWJt/yAK/AEK+QEKA2liYxIg2nvqL76rejXXGlX6ng/UKrbw+72C8uKKgM2vP0JKj1QaCQgBGAEgASoBACIlCAESIQEGuZwNgRn/HtvL4WXQ8ZM327wIDmd8iOV6oq52fr8PDyInCAESAQEaIKplBAbqDXbjndQ9LqapHj/aockI/CGnymjl5izIEVY5IiUIARIhAdt4G8DCLINAaaJnhUMIzv74AV3zZiugAyyZ/lWYRv+cIiUIARIhAf+sohoEV+uWeKThAPEbqCUivWT4H8KNT7Giw9//LsyvIicIARIBARogNHO4HC5KxPCwBdQGgBCscVtEKw+YSn2pnf654Y3Oxik=",
                "path":{"key_path":["aWJj","Y29ubmVjdGlvbnMvY29ubmVjdGlvbi0w"]},
                "value":"Cg8wNy10ZW5kZXJtaW50LTASIwoBMRINT1JERVJfT1JERVJFRBIPT1JERVJfVU5PUkRFUkVEGAIiIAoJMDgtd2FzbS0wEgxjb25uZWN0aW9uLTAaBQoDaWJj"
            }
        }"#;
        assert!(matches!(
            serde_json::from_str::<SudoMsg>(sudo_msg).unwrap(),
            SudoMsg::VerifyMembership(_)
        ));
    }

    #[test]
    fn instantiate_msg_from_json() {
        let instantiate_msg = r#"{
            "client_state":"Y2xpZW50X3N0YXRlCg==",
            "consensus_state":"Y29uc2Vuc3VzX3N0YXRlCg==",
            "checksum":"Y2hlY2tzdW0K"
        }"#;
        serde_json::from_str::<InstantiateMsg>(instantiate_msg).unwrap();
    }
}
