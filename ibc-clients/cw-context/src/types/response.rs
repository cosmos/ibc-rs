//! Contains the response types for the CosmWasm contract.
use cosmwasm_schema::cw_serde;
use ibc_client_wasm_types::Bytes;
use ibc_core::client::types::Height;

#[cw_serde]
pub struct GenesisMetadata {
    pub key: Bytes,
    pub value: Bytes,
}

#[cw_serde]
pub struct QueryResponse {
    pub is_valid: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub genesis_metadata: Option<Vec<GenesisMetadata>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub found_misbehaviour: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<u64>,
}

impl QueryResponse {
    pub fn success() -> Self {
        Self {
            is_valid: true,
            status: None,
            genesis_metadata: None,
            found_misbehaviour: None,
            timestamp: None,
        }
    }

    pub fn status(mut self, status: String) -> Self {
        self.status = Some(status);
        self
    }

    pub fn genesis_metadata(mut self, genesis_metadata: Option<Vec<GenesisMetadata>>) -> Self {
        self.genesis_metadata = genesis_metadata;
        self
    }

    pub fn misbehaviour(mut self, found_misbehavior: bool) -> Self {
        self.found_misbehaviour = Some(found_misbehavior);
        self
    }

    pub fn timestamp(mut self, timestamp: u64) -> Self {
        self.timestamp = Some(timestamp);
        self
    }
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
