use displaydoc::Display;
use ibc_primitives::prelude::*;

#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[derive(Debug, Display)]
pub enum IdentifierError {
    /// identifier `{id}` has invalid length; must be between `{min}` and `{max}` characters
    InvalidLength { id: String, min: u64, max: u64 },
    /// identifier `{id}` must only contain alphanumeric characters or `.`, `_`, `+`, `-`, `#`, - `[`, `]`, `<`, `>`
    InvalidCharacter { id: String },
    /// identifier prefix `{prefix}` is invalid
    InvalidPrefix { prefix: String },
    /// chain identifier is not formatted with revision number
    UnformattedRevisionNumber { chain_id: String },
    /// revision number overflowed
    RevisionNumberOverflow,
    /// String `{value}` cannot be converted to packet sequence, error: `{reason}`
    InvalidStringAsSequence { value: String, reason: String },
}

#[cfg(feature = "std")]
impl std::error::Error for IdentifierError {}
