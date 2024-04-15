#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![deny(
    warnings,
    trivial_casts,
    trivial_numeric_casts,
    unused_import_braces,
    unused_qualifications,
    rust_2018_idioms
)]
#![forbid(unsafe_code)]

extern crate alloc;

pub mod api;
pub mod context;
pub mod handlers;
pub mod types;
pub mod utils;
