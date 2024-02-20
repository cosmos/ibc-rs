//! ICS-07: Tendermint Client implements a client verification algorithm for blockchains which use
//! the Tendermint consensus algorithm.
#![no_std]
#![forbid(unsafe_code)]
#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::disallowed_methods, clippy::disallowed_types,))]
#![deny(
    warnings,
    trivial_casts,
    trivial_numeric_casts,
    unused_import_braces,
    unused_qualifications,
    rust_2018_idioms
)]

use core::str::FromStr;

use ibc_core_host_types::identifiers::ClientType;

#[cfg(any(test, feature = "std"))]
extern crate std;

mod client_state;
mod consensus_state;
mod header;
mod misbehaviour;
mod trust_threshold;

pub use client_state::*;
pub use consensus_state::*;
pub use header::*;
pub use misbehaviour::*;
pub use trust_threshold::*;

pub mod error;

/// Re-exports ICS-07 Tendermint light client from `ibc-proto` crate.
pub mod proto {
    pub use ibc_proto::ibc::lightclients::tendermint::*;
}

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
