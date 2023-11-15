//! Implementation of the [fungible token transfer module](https://github.com/cosmos/ibc/blob/main/spec/app/ics-020-fungible-token-transfer/README.md) (ICS-20)
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
#![allow(clippy::result_large_err)]
#![cfg_attr(not(test), deny(clippy::disallowed_methods, clippy::disallowed_types,))]
#![forbid(unsafe_code)]

#[cfg(any(test, feature = "std"))]
extern crate std;

#[doc(inline)]
pub use ibc_app_transfer_types as types;

pub mod context;
#[cfg(feature = "serde")]
pub mod handler;
#[cfg(feature = "serde")]
pub mod module;
