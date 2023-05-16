use crate::core::ics02_client::error::ClientError;
use alloc::string::{String, ToString};
use ibc_proto::cosmos::upgrade::v1beta1::Plan as RawPlan;
use ibc_proto::google::protobuf::Any;
use ibc_proto::ibc::core::client::v1::UpgradeProposal as RawUpgradeProposal;
use ibc_proto::protobuf::Protobuf;

#[derive(Clone, Debug)]
pub struct UpgradeProposal {
    pub title: String,
    pub description: String,
    pub plan: Plan,
    pub upgraded_client_state: Any,
}

impl Protobuf<RawUpgradeProposal> for UpgradeProposal {}

impl TryFrom<RawUpgradeProposal> for UpgradeProposal {
    type Error = ClientError;

    fn try_from(raw: RawUpgradeProposal) -> Result<Self, Self::Error> {
        if raw.title.is_empty() {
            return Err(ClientError::InvalidUpgradeProposal {
                reason: "title field cannot be empty".to_string(),
            });
        }

        if raw.description.is_empty() {
            return Err(ClientError::InvalidUpgradeProposal {
                reason: "description field cannot be empty".to_string(),
            });
        }

        let plan = if let Some(plan) = raw.plan {
            plan.try_into()?
        } else {
            return Err(ClientError::InvalidUpgradeProposal {
                reason: "plan field cannot be empty".to_string(),
            });
        };

        let upgraded_client_state = if let Some(upgraded_client_state) = raw.upgraded_client_state {
            upgraded_client_state
        } else {
            return Err(ClientError::InvalidUpgradeProposal {
                reason: "upgraded_client_state field cannot be empty".to_string(),
            });
        };

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

#[derive(Clone, Debug)]
pub struct Plan {
    pub name: String,
    pub height: u64,
    pub info: String,
}

impl Protobuf<RawPlan> for Plan {}

impl TryFrom<RawPlan> for Plan {
    type Error = ClientError;

    fn try_from(raw: RawPlan) -> Result<Self, Self::Error> {
        if raw.name.is_empty() {
            return Err(ClientError::Other {
                description: "name field cannot be empty".to_string(),
            });
        }

        #[allow(deprecated)]
        if raw.time.is_some() {
            return Err(ClientError::Other {
                description: "time field must be empty".to_string(),
            });
        }

        #[allow(deprecated)]
        if raw.upgraded_client_state.is_some() {
            return Err(ClientError::Other {
                description: "upgraded_client_state field must be empty".to_string(),
            });
        }

        Ok(Self {
            name: raw.name,
            height: u64::try_from(raw.height).map_err(|_| ClientError::InvalidHeight)?,
            info: raw.info,
        })
    }
}

impl From<Plan> for RawPlan {
    fn from(value: Plan) -> Self {
        #[allow(deprecated)]
        Self {
            name: value.name,
            height: i64::try_from(value.height).expect("height overflow"),
            info: value.info,
            time: None,
            upgraded_client_state: None,
        }
    }
}
