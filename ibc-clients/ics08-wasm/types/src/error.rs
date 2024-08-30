//! Defines the error type for the ICS-08 Wasm light client.

use displaydoc::Display;
use ibc_core_host_types::error::{DecodingError, IdentifierError};
use ibc_primitives::prelude::*;

/// The main error type
#[derive(Debug, Display)]
pub enum WasmClientError {
    /// decoding error: `{0}`
    Decoding(DecodingError),
    /// invalid identifier: `{0}`
    InvalidIdentifier(IdentifierError),
    /// invalid client state latest height
    InvalidLatestHeight,
    /// missing latest height
    MissingLatestHeight,
}

#[cfg(feature = "std")]
impl std::error::Error for WasmClientError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidIdentifier(e) => Some(e),
            Self::Decoding(e) => Some(e),
            _ => None,
        }
    }
}

impl From<IdentifierError> for WasmClientError {
    fn from(e: IdentifierError) -> Self {
        Self::InvalidIdentifier(e)
    }
}

impl From<DecodingError> for WasmClientError {
    fn from(e: DecodingError) -> Self {
        Self::Decoding(e)
    }
}
