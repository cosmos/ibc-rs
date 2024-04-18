//! ICS 07: Tendermint light client implementation along with re-exporting the
//! necessary types from `ibc-client-tendermint-types` crate.
#![no_std]
#![forbid(unsafe_code)]
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

#[cfg(any(test, feature = "std"))]
extern crate std;

pub mod client_state;
pub mod consensus_state;

pub const TENDERMINT_CLIENT_TYPE: &str = "07-tendermint";

/// Re-exports Tendermint light client data structures from the
/// `ibc-client-tendermint-type` crate.
pub mod types {
    #[doc(inline)]
    pub use ibc_client_tendermint_types::*;
}
