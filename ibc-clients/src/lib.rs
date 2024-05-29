//! Exports data structures and implementations of different IBC applications.
#![no_std]
#![forbid(unsafe_code)]
#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::disallowed_methods, clippy::disallowed_types))]
#![deny(
    warnings,
    trivial_numeric_casts,
    unused_import_braces,
    unused_qualifications,
    rust_2018_idioms
)]

/// Re-exports implementations of ICS-07 Tendermint light client.
pub mod tendermint {
    #[doc(inline)]
    pub use ibc_client_tendermint::*;
}

/// Re-exports implementations of ICS-08 Wasm light client types.
pub mod wasm_types {
    #[doc(inline)]
    pub use ibc_client_wasm_types::*;
}
