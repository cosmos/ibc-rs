use core::str::FromStr;
use ibc::core::ics02_client::client_type::ClientType;

extern crate alloc;

pub mod error;
pub mod client_state;
pub mod consensus_state;
pub mod header;
pub mod misbehaviour;
pub mod trust_threshold;

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