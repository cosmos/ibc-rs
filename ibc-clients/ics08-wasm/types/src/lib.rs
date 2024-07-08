//! Definitions of domain types used in the ICS-08 Wasm light client implementation.
#![forbid(unsafe_code)]
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::disallowed_methods, clippy::disallowed_types))]
#![deny(
    warnings,
    trivial_casts,
    trivial_numeric_casts,
    unused_import_braces,
    unused_qualifications,
    rust_2018_idioms
)]

pub mod client_message;
pub mod client_state;
pub mod consensus_state;
pub mod error;
pub mod msgs;

#[cfg(feature = "cosmwasm")]
pub mod serializer;

use core::str::FromStr;

use ibc_core_host_types::identifiers::ClientType;
#[cfg(not(feature = "std"))]
use ibc_primitives::prelude::Vec;

pub type Bytes = Vec<u8>;

/// Re-exports ICS-08 Wasm light client proto types from `ibc-proto` crate.
pub mod proto {
    pub use ibc_proto::ibc::lightclients::wasm::*;
}

pub static SUBJECT_PREFIX: &[u8] = b"subject/";
pub static SUBSTITUTE_PREFIX: &[u8] = b"substitute/";

pub const WASM_CLIENT_TYPE: &str = "08-wasm";

/// Returns the wasm `ClientType`
pub fn client_type() -> ClientType {
    ClientType::from_str(WASM_CLIENT_TYPE).expect("Never fails because it's valid")
}

#[cfg(test)]
mod tests {
    use super::*;

    // Ensures that the validation in `ClientType::from_str` doesn't fail for the wasm client type
    #[test]
    pub fn test_wasm_client_type() {
        let _ = ClientType::from_str(WASM_CLIENT_TYPE).unwrap();
    }
}
