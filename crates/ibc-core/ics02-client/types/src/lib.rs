//! Implementation of the IBC [fungible token transfer](https://github.com/cosmos/ibc/blob/main/spec/app/ics-020-fungible-token-transfer/README.md) (ICS-20) data structures.

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
mod height;
pub mod msgs;
mod status;

pub use height::*;
pub use status::*;

/// Re-exports pertinent ibc proto types from the `ibc-proto-rs` crate for added convenience
pub mod proto {
    pub use ibc_proto::google::protobuf::Any;
    pub use ibc_proto::ibc::core::client::*;
    pub use ibc_proto::Protobuf;
}

pub mod primitives {
    pub use ibc_primitives::*;
}
