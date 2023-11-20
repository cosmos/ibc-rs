//! ICS-07: Tendermint Client implements a client verification algorithm for blockchains which use
//! the Tendermint consensus algorithm.

use core::str::FromStr;

pub mod client_state;
pub mod consensus_state;
pub mod error;
pub mod header;
pub mod misbehaviour;
pub mod trust_threshold;

mod context;
pub use context::*;
use ibc_core::host::types::identifiers::ClientType;

pub const TENDERMINT_CLIENT_TYPE: &str = "07-tendermint";

/// Returns the tendermint `ClientType`
pub fn client_type() -> ClientType {
    ClientType::from_str(TENDERMINT_CLIENT_TYPE).expect("Never fails because it's valid")
}

#[cfg(test)]
mod tests {
    use super::*;

    // Ensures that the validation in `ClientType::from_str` doesn't fail for the tendermint client type
    #[test]
    pub fn test_tm_client_type() {
        let _ = ClientType::from_str(TENDERMINT_CLIENT_TYPE).unwrap();
    }
}
