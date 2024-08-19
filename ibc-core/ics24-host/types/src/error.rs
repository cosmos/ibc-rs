use displaydoc::Display;
use ibc_primitives::prelude::*;

/// Errors that arise when parsing identifiers.
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[derive(Debug, Display)]
pub enum IdentifierError {
    /// id `{actual}` has invalid length; must be between [`{min}`,`{max}`)
    InvalidLength { actual: String, min: u64, max: u64 },
    /// id `{0}` can only contain alphanumeric characters or `.`, `_`, `+`, `-`, `#`, - `[`, `]`, `<`, `>`
    InvalidCharacter(String),
    /// invalid prefix: `{0}`
    InvalidPrefix(String),
    /// invalid revision number for chain ID: `{0}`
    InvalidRevisionNumber(String),
    /// invalid packet sequence `{sequence}`: `{description}`
    InvalidPacketSequence {
        sequence: String,
        description: String,
    },
    /// overflowed revision number
    OverflowedRevisionNumber,
}

#[cfg(feature = "std")]
impl std::error::Error for IdentifierError {}
