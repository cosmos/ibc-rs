//! Definition of domain `Plan` type.

use ibc_core_client_types::error::UpgradeClientError;
use ibc_core_host_types::error::{DecodingError, IdentifierError};
use ibc_primitives::prelude::*;
use ibc_proto::cosmos::upgrade::v1beta1::Plan as RawPlan;
use ibc_proto::google::protobuf::Any;
use ibc_proto::Protobuf;

pub const TYPE_URL: &str = "/cosmos.upgrade.v1beta1.Plan";

/// Specifies information about a planned upgrade and at which height it should
/// be performed.
///
/// Note: Time based upgrade logic has been removed from the SDK, so the `time`
/// field of the proto is deprecated and should be empty.
#[derive(Clone, Debug)]
pub struct Plan {
    // Sets the name for the upgrade. This name might be used by the upgraded
    // version of a host chain to apply any special "on-upgrade" commands during
    // the first block generation after the upgrade is applied.
    pub name: String,
    // The height at which the upgrade must be performed.
    pub height: u64,
    // Any application specific upgrade info to be included on-chain
    pub info: String,
}

impl Protobuf<RawPlan> for Plan {}

impl TryFrom<RawPlan> for Plan {
    type Error = DecodingError;

    fn try_from(raw: RawPlan) -> Result<Self, Self::Error> {
        if raw.name.is_empty() {
            return Err(DecodingError::InvalidRawData {
                description: "upgrade plan name cannot be empty".to_string(),
            });
        }

        #[allow(deprecated)]
        if raw.time.is_some() {
            return Err(DecodingError::InvalidRawData {
                description: "upgrade plan time must be empty".to_string(),
            });
        }

        #[allow(deprecated)]
        if raw.upgraded_client_state.is_some() {
            return Err(DecodingError::InvalidRawData {
                description: "upgrade plan `upgraded_client_state` field must be empty".to_string(),
            });
        }

        Ok(Self {
            name: raw.name,
            height: u64::try_from(raw.height).map_err(|_| {
                DecodingError::Identifier(IdentifierError::OverflowedRevisionNumber)
            })?,
            info: raw.info,
        })
    }
}

impl From<Plan> for RawPlan {
    fn from(value: Plan) -> Self {
        #[allow(deprecated)]
        Self {
            name: value.name,
            time: None,
            height: i64::try_from(value.height).expect("height overflow"),
            info: value.info,
            upgraded_client_state: None,
        }
    }
}

impl Protobuf<Any> for Plan {}

impl TryFrom<Any> for Plan {
    type Error = UpgradeClientError;

    fn try_from(any: Any) -> Result<Self, Self::Error> {
        match any.type_url.as_str() {
            TYPE_URL => {
                Ok(Protobuf::<RawPlan>::decode_vec(&any.value).map_err(DecodingError::Protobuf)?)
            }
            _ => Err(DecodingError::MismatchedTypeUrls {
                expected: TYPE_URL.to_string(),
                actual: any.type_url,
            })?,
        }
    }
}

impl From<Plan> for Any {
    fn from(value: Plan) -> Self {
        Any {
            type_url: TYPE_URL.to_string(),
            value: Protobuf::<RawPlan>::encode_vec(value),
        }
    }
}
