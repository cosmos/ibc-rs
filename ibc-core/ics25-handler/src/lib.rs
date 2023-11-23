//! Exposes IBC handler entry points for an integrated IBC core modules. These
//! entry points are responsible for processing incoming IBC messages,
//! performing validation, and execution logics by invoking the appropriate
//! module handler.
//!
//! When processing a given message `M`, if any method in this library returns
//! an error, the runtime is expected to rollback all state modifications made
//! to the context (e.g. [`ExecutionContext`](ibc_core_host::ExecutionContext))
//! while processing `M`. If the transaction containing `M` consists of multiple
//! messages, then typically the state modifications from all messages is
//! expected to be rolled back as well.
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

#[cfg(any(test, feature = "std"))]
extern crate std;

pub mod entrypoint;

/// Re-export IBC handler types from `ibc-core-handler-types` crate.
pub mod types {
    #[doc(inline)]
    pub use ibc_core_handler_types::*;
}
