//! Exports data structures and implementations of different IBC core (TAO) components.
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

pub mod entrypoint;

pub mod types {
    #[doc(inline)]
    pub use ibc_core_handler_types::*;
}
