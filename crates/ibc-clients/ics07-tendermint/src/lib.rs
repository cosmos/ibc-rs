//! Light client implementations to be used in [core](crate::core).
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
#![allow(clippy::result_large_err)]

use core::any::Any;

extern crate alloc;

#[cfg(any(test, feature = "std"))]
extern crate std;

pub mod client_state;
pub mod context;

pub const TENDERMINT_CLIENT_TYPE: &str = "07-tendermint";

#[doc(inline)]
pub use ibc_client_tendermint_types as types;

/// Allows type to be converted to `&dyn Any`
pub trait AsAny: Any {
    fn as_any(&self) -> &dyn Any;
}

impl<M: Any> AsAny for M {
    fn as_any(&self) -> &dyn Any {
        self
    }
}
