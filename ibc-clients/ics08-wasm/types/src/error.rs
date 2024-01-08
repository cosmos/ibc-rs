//! Defines the error type for the ICS-08 Wasm light client.

use displaydoc::Display;
use ibc_core_host_types::error::IdentifierError;
use ibc_primitives::prelude::*;

/// The main error type
#[derive(Debug, Display)]
pub enum Error {
    /// invalid identifier: `{0}`
    InvalidIdentifier(IdentifierError),
    /// decoding error: `{reason}`
    DecodeError { reason: String },
    /// invalid client state latest height: `{reason}`
    InvalidLatestHeight { reason: String },
}

#[cfg(feature = "std")]
impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::InvalidIdentifier(err) => Some(err),
            _ => None,
        }
    }
}

impl From<IdentifierError> for Error {
    fn from(e: IdentifierError) -> Self {
        Self::InvalidIdentifier(e)
    }
}
