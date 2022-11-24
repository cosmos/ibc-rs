use crate::prelude::*;
use displaydoc::Display;
use serde::Serialize;

#[derive(Debug, Display, Serialize)]
pub enum ValidationError {
    /// identifier `{id}` cannot contain separator '/'
    ContainSeparator { id: String },
    /// identifier `{id}` has invalid length `{length}` must be between `{min}`-`{max}` characters
    InvalidLength {
        id: String,
        length: usize,
        min: usize,
        max: usize,
    },
    /// identifier `{id}` must only contain alphanumeric characters or `.`, `_`, `+`, `-`, `#`, - `[`, `]`, `<`, `>`
    InvalidCharacter { id: String },
    /// identifier cannot be empty
    Empty,
    /// chain identifiers are expected to be in epoch format `{id}`
    ChainIdInvalidFormat { id: String },
    /// Invalid channel id in counterparty
    InvalidCounterpartyChannelId,
}

#[cfg(feature = "std")]
impl std::error::Error for ValidationError {}
