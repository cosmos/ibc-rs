//! Tendermint light client implementation to be used in [core](crate::core).
use core::str::FromStr;
use ibc::core::ics02_client::client_type::ClientType;

mod error;

pub mod impls;
pub mod types;

pub use impls::context::*;

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