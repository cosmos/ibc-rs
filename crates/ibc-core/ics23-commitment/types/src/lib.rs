//! ICS-23: Commitment implementation of a cryptographic scheme that verifies
//! state transitions between chains.
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

pub mod commitment;
pub mod error;
pub mod merkle;
pub mod specs;

#[cfg(feature = "serde")]
pub mod serializer;

/// Re-exports ICS-23 proto types from the `ibc-proto` crate, which are
/// used in the implementation of dependent IBC crates.
pub mod proto {
    pub use ibc_proto::ibc::core::commitment::*;
    pub use ibc_proto::ics23;
}
