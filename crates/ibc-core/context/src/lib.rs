#![no_std]
#![forbid(unsafe_code)]
#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::disallowed_methods, clippy::disallowed_types))]
#![deny(
    warnings,
    trivial_numeric_casts,
    unused_import_braces,
    unused_qualifications,
    rust_2018_idioms
)]

pub(crate) mod block_delay;

mod context;
pub use context::*;

pub mod types {
    #[doc(inline)]
    pub use ibc_core_context_types::*;
}
