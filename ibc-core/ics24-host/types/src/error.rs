use displaydoc::Display;
use ibc_primitives::prelude::*;

/// Errors that arise when parsing identifiers.
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[derive(Debug, Display)]
pub enum IdentifierError {
    /// id `{actual}` has invalid length; must be between [`{min}`,`{max}`)
    InvalidLength { actual: String, min: u64, max: u64 },
    /// id `{actual}` can only contain alphanumeric characters or `.`, `_`, `+`, `-`, `#`, - `[`, `]`, `<`, `>`
    InvalidCharacter { actual: String },
    /// invalid prefix: `{actual}`
    InvalidPrefix { actual: String },
    /// invalid revision number for chain ID: `{chain_id}`
    InvalidRevisionNumber { chain_id: String },
    /// invalid packet sequence `{actual}`: `{description}`
    InvalidPacketSequence { actual: String, description: String },
    /// overflowed revision number
    OverflowedRevisionNumber,
}

#[cfg(feature = "std")]
impl std::error::Error for IdentifierError {}
