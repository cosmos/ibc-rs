//! Contains the response types for the CosmWasm contract.
use cosmwasm_schema::cw_serde;
use cosmwasm_std::Binary;
use ibc_core::client::types::Height;

/// The response to [`super::msgs::QueryMsg::Status`]
#[cw_serde]
pub struct Status {
    /// The status of the client
    // TODO: Turn this into an enum
    pub status: String,
}

/// The response to [`super::msgs::QueryMsg::ExportMetadata`]
#[cw_serde]
pub struct ExportMetadata {
    /// The genesis metadata
    pub metadata: Vec<GenesisMetadata>,
}

/// The response to [`super::msgs::QueryMsg::TimestampAtHeight`]
#[cw_serde]
pub struct TimestampAtHeight {
    /// The timestamp at the given height
    pub timestamp: u64,
}

/// The response to [`super::QueryMsg::VerifyClientMessage`]
#[cw_serde]
pub struct VerifyClientMessage {
    /// Whether the client message is valid
    pub is_valid: bool,
}

/// The response to [`super::msgs::QueryMsg::CheckForMisbehaviour`]
#[cw_serde]
pub struct CheckForMisbehaviour {
    /// Whether misbehaviour was found
    pub found_misbehaviour: bool,
}

#[cw_serde]
pub struct GenesisMetadata {
    pub key: Binary,
    pub value: Binary,
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
