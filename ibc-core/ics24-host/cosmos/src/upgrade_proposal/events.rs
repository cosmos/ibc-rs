//! Definitions of events emitted when an upgrade client is proposed or executed.

use derive_more::From;
use ibc_primitives::prelude::*;
use tendermint::abci;

const UPGRADE_CHAIN_EVENT: &str = "upgrade_chain";
const UPGRADE_CLIENT_PROPOSAL_EVENT: &str = "upgrade_client_proposal";

const KEY_UPGRADE_STORE_ATTRIBUTE_KEY: &str = "upgrade_store";
const UPGRADE_PLAN_HEIGHT_ATTRIBUTE_KEY: &str = "upgrade_plan_height";
const UPGRADE_PLAN_TITLE_ATTRIBUTE_KEY: &str = "title";

#[cfg_attr(
    feature = "parity-scale-codec",
    derive(
        parity_scale_codec::Encode,
        parity_scale_codec::Decode,
        scale_info::TypeInfo
    )
)]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, From, PartialEq, Eq)]
struct UpgradeStoreAttribute {
    upgrade_store: String,
}

impl From<UpgradeStoreAttribute> for abci::EventAttribute {
    fn from(attr: UpgradeStoreAttribute) -> Self {
        (KEY_UPGRADE_STORE_ATTRIBUTE_KEY, attr.upgrade_store).into()
    }
}

#[cfg_attr(
    feature = "parity-scale-codec",
    derive(
        parity_scale_codec::Encode,
        parity_scale_codec::Decode,
        scale_info::TypeInfo
    )
)]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, From, PartialEq, Eq)]
struct UpgradePlanHeightAttribute {
    plan_height: u64,
}

impl From<UpgradePlanHeightAttribute> for abci::EventAttribute {
    fn from(attr: UpgradePlanHeightAttribute) -> Self {
        (
            UPGRADE_PLAN_HEIGHT_ATTRIBUTE_KEY,
            attr.plan_height.to_string(),
        )
            .into()
    }
}

#[cfg_attr(
    feature = "parity-scale-codec",
    derive(
        parity_scale_codec::Encode,
        parity_scale_codec::Decode,
        scale_info::TypeInfo
    )
)]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, From, PartialEq, Eq)]
struct UpgradePlanTitleAttribute {
    title: String,
}

impl From<UpgradePlanTitleAttribute> for abci::EventAttribute {
    fn from(attr: UpgradePlanTitleAttribute) -> Self {
        (UPGRADE_PLAN_TITLE_ATTRIBUTE_KEY, attr.title).into()
    }
}

/// Event type emitted by the host chain when an upgrade plan is executed.
#[cfg_attr(
    feature = "parity-scale-codec",
    derive(
        parity_scale_codec::Encode,
        parity_scale_codec::Decode,
        scale_info::TypeInfo
    )
)]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UpgradeChain {
    // The height at which the upgrade performed.
    plan_height: UpgradePlanHeightAttribute,
    // The key of the store where the upgrade plan is stored.
    upgrade_store: UpgradeStoreAttribute,
}

impl UpgradeChain {
    pub fn new(plan_height: u64, upgrade_store: String) -> Self {
        Self {
            plan_height: UpgradePlanHeightAttribute::from(plan_height),
            upgrade_store: UpgradeStoreAttribute::from(upgrade_store),
        }
    }
    pub fn event_type(&self) -> &str {
        UPGRADE_CHAIN_EVENT
    }
}

impl From<UpgradeChain> for abci::Event {
    fn from(u: UpgradeChain) -> Self {
        Self {
            kind: UPGRADE_CHAIN_EVENT.to_owned(),
            attributes: vec![u.plan_height.into(), u.upgrade_store.into()],
        }
    }
}

/// Event type emitted by the host chain when an upgrade plan is proposed.
#[cfg_attr(
    feature = "parity-scale-codec",
    derive(
        parity_scale_codec::Encode,
        parity_scale_codec::Decode,
        scale_info::TypeInfo
    )
)]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UpgradeClientProposal {
    // The title of the upgrade plan
    plan_title: UpgradePlanTitleAttribute,
    // The height at which the upgrade must be performed.
    plan_height: UpgradePlanHeightAttribute,
}

impl UpgradeClientProposal {
    pub fn new(plan_title: String, plan_height: u64) -> Self {
        Self {
            plan_title: UpgradePlanTitleAttribute::from(plan_title),
            plan_height: UpgradePlanHeightAttribute::from(plan_height),
        }
    }
    pub fn event_type(&self) -> &str {
        UPGRADE_CLIENT_PROPOSAL_EVENT
    }
}

impl From<UpgradeClientProposal> for abci::Event {
    fn from(u: UpgradeClientProposal) -> Self {
        Self {
            kind: UPGRADE_CLIENT_PROPOSAL_EVENT.to_owned(),
            attributes: vec![u.plan_title.into(), u.plan_height.into()],
        }
    }
}
