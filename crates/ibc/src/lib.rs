#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![no_std]
#![deny(
    warnings,
    trivial_casts,
    trivial_numeric_casts,
    unused_import_braces,
    unused_qualifications,
    rust_2018_idioms
)]
#![forbid(unsafe_code)]
// https://github.com/cosmos/ibc-rs/issues/342
#![allow(clippy::result_large_err)]
//! This library implements the InterBlockchain Communication (IBC) protocol in Rust. IBC is
//! a distributed protocol that enables communication between distinct sovereign blockchains.
//!
//! The layout of this crate mirrors the organization of the [IBC
//! Standard][ibc-standard]:
//!
//! + [Core](core) implements the transport, authentication, and ordering layers of the IBC protocol.
//!
//! + [Clients](clients) consists of implementations of client verification algorithms (following the base
//! client interface that is defined in `Core`) for specific consensus algorithms. A chain uses these
//! verification algorithms to verify the state of remote chains.
//!
//! + [Applications](applications) consists of implementations of some IBC applications. This is the part of
//! the protocol that abstracts away the core protocol and focuses solely on business logic.
//!
//! When processing a given message `M`, if any method in this library returns an error, the runtime
//! is expected to rollback all state modifications made to the context
//! (e.g. [`ExecutionContext`](core::ExecutionContext)) while processing `M`. If a transaction on your
//! blockchain contains multiple messages, then typically the state modifications from all messages
//! is expected to be rolled back as well.
//!
//! # Note
//!
//! Currently, the `serde` feature (required by the token transfer app) does not work in `no_std` environments.
//! See context [here](https://github.com/cosmos/ibc-proto-rs/pull/92). If this is a blocker for you, please
//! open a Github issue.
//!
//! [ibc-standard]: https://github.com/cosmos/ibc

extern crate alloc;
#[cfg(any(test, feature = "std"))]
extern crate std;

/// Represents a block height
pub use crate::core::ics02_client::height::Height;
pub use signer::Signer;

pub mod applications;
pub mod clients;
pub mod core;
pub mod hosts;

#[cfg(any(test, feature = "mocks", feature = "mocks-no-std"))]
pub mod mock;
#[cfg(any(test, feature = "mocks"))]
pub mod test_utils; // Context mock, the underlying host chain, and client types: for testing all handlers.

mod prelude;
mod signer;
mod utils;

#[cfg(feature = "serde")]
pub mod serializers;

#[cfg(test)]
mod test;

/// Re-export the `Any` type which used across the library.
pub use ibc_proto::google::protobuf::Any;
