//! Encapsulates essential data structures facilitating the seamless
//! interaction between an implemented IBC module using ibc-rs and the
//! underlying host blockchain.
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

pub mod error;
pub mod events;
pub mod msgs;

/// Re-exports necessary proto types from the `ibc-proto-rs` crate, which are
/// instrumental in the implementation of the higher-level `ibc-core-handler`
/// crate.
pub mod proto {
    pub use ibc_proto::google::protobuf::Any;
}
