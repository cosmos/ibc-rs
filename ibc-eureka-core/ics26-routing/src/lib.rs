//! This library contains necessary traits to implement an IBC router module when integrating with `ibc-rs`.
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

pub mod module;
pub mod router;

/// Re-exports router data structures from the `ibc-core-router-types` crate
pub mod types {
    #[doc(inline)]
    pub use ibc_core_router_types::*;
}
