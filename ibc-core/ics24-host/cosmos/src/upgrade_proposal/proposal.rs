//! Definition of domain `UpgradeProposal` type for handling upgrade client proposal

use ibc_core_host_types::error::DecodingError;
use ibc_primitives::prelude::*;
use ibc_proto::google::protobuf::Any;
use ibc_proto::ibc::core::client::v1::UpgradeProposal as RawUpgradeProposal;
use ibc_proto::Protobuf;

use super::Plan;

pub const UPGRADE_PROPOSAL_TYPE_URL: &str = "/ibc.core.client.v1.UpgradeProposal";

/// Defines a governance proposal of type `Content` that enables the initiation
/// of an IBC breaking upgrade and specifies the new client state that should be
/// utilized following the upgrade.
#[derive(Clone, Debug)]
pub struct UpgradeProposal {
    // Title of the proposal
    pub title: String,
    // Description of the proposal
    pub description: String,
    // The upgrade plan
    pub plan: Plan,
    // The upgraded client state
    pub upgraded_client_state: Any,
}

impl Protobuf<RawUpgradeProposal> for UpgradeProposal {}

impl TryFrom<RawUpgradeProposal> for UpgradeProposal {
    type Error = DecodingError;

    fn try_from(raw: RawUpgradeProposal) -> Result<Self, Self::Error> {
        if raw.title.is_empty() {
            return Err(DecodingError::InvalidRawData {
                description: "upgrade proposal missing title field".to_string(),
            });
        }

        if raw.description.is_empty() {
            return Err(DecodingError::InvalidRawData {
                description: "upgrade proposal missing description field".to_string(),
            });
        }

        let plan = if let Some(plan) = raw.plan {
            plan.try_into()?
        } else {
            return Err(DecodingError::InvalidRawData {
                description: "upgrade proposal missing plan field".to_string(),
            });
        };

        let upgraded_client_state =
            raw.upgraded_client_state
                .ok_or_else(|| DecodingError::InvalidRawData {
                    description: "upgrade proposal missing upgraded client state".to_string(),
                })?;

        Ok(Self {
            title: raw.title,
            description: raw.description,
            plan,
            upgraded_client_state,
        })
    }
}

impl From<UpgradeProposal> for RawUpgradeProposal {
    fn from(value: UpgradeProposal) -> Self {
        Self {
            title: value.title,
            description: value.description,
            plan: Some(value.plan.into()),
            upgraded_client_state: Some(value.upgraded_client_state),
        }
    }
}
