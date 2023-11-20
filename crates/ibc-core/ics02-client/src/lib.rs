//! ICS-02: Client Semantics implementation for verifying remote IBC-enabled chains,
//! along with re-exporting data structures from `ibc-core-client-types` crate.
#![no_std]
#![forbid(unsafe_code)]
#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::disallowed_methods, clippy::disallowed_types,))]
#![deny(
    warnings,
    trivial_numeric_casts,
    unused_import_braces,
    unused_qualifications,
    rust_2018_idioms
)]

#[cfg(feature = "std")]
extern crate std;

pub mod handler;

/// Re-exports ICS-02 traits from `ibc-core-client-context` for custom IBC
/// client implementation.
pub mod context {
    #[doc(inline)]
    pub use ibc_core_client_context::*;
}

/// Re-exports ICS-02 data structures from the `ibc-core-client-types` crate.
pub mod types {
    #[doc(inline)]
    pub use ibc_core_client_types::*;
}
