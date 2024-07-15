//! Contains the response types for the CosmWasm contract.
use cosmwasm_schema::cw_serde;
use ibc_core::client::types::Height;

/// The response to [`super::msgs::QueryMsg::Status`]
#[cw_serde]
pub struct StatusResponse {
    /// The status of the client
    // TODO: Turn this into an enum
    pub status: String,
}

/// The response to [`super::msgs::QueryMsg::TimestampAtHeight`]
#[cw_serde]
pub struct TimestampAtHeightResponse {
    /// The timestamp at the given height
    pub timestamp: u64,
}

/// The response to [`super::QueryMsg::VerifyClientMessage`]
#[cw_serde]
pub struct VerifyClientMessageResponse {
    /// Whether the client message is valid
    pub is_valid: bool,
}

/// The response to [`super::msgs::QueryMsg::CheckForMisbehaviour`]
#[cw_serde]
pub struct CheckForMisbehaviourResponse {
    /// Whether misbehaviour was found
    pub found_misbehaviour: bool,
}

#[cw_serde]
pub struct ContractResult {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heights: Option<Vec<Height>>,
}

impl ContractResult {
    pub fn success() -> Self {
        Self { heights: None }
    }

    pub fn heights(mut self, heights: Vec<Height>) -> Self {
        self.heights = Some(heights);
        self
    }
}
