//! Defines the error type for the ICS-08 Wasm light client.

use displaydoc::Display;
use ibc_primitives::prelude::*;

/// The main error type
#[derive(Debug, Display)]
pub enum Error {
    /// decoding error: `{reason}`
    DecodeError { reason: String },
    /// invalid client state latest height: `{reason}`
    InvalidLatestHeight { reason: String },
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}
