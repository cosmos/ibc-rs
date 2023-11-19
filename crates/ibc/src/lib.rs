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
//! When processing a given message `M`, if any method in this library returns an error, the runtime
//! is expected to rollback all state modifications made to the context
//! (e.g. [`ExecutionContext`](crate::core::host::ExecutionContext)) while processing `M`. If a transaction on your
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

pub mod clients;

pub mod apps {
    #[doc(inline)]
    pub use ibc_apps::*;
}

pub mod core {
    #[doc(inline)]
    pub use ibc_core::*;
}
/// Re-exports pertinent ibc proto types from the `ibc-proto-rs` crate for added convenience
pub mod proto {
    pub use ibc_proto::google::protobuf::Any;
    pub use ibc_proto::ibc::lightclients::tendermint;
    pub use ibc_proto::Protobuf;
}
