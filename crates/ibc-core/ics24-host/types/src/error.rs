use displaydoc::Display;

#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[derive(Debug, Display)]
pub enum IdentifierError {
    /// identifier `{id}` cannot contain separator '/'
    ContainSeparator { id: String },
    /// identifier `{id}` has invalid length `{length}` must be between `{min}`-`{max}` characters
    InvalidLength {
        id: String,
        length: u64,
        min: u64,
        max: u64,
    },
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
    /// identifier cannot be empty
    Empty,
}

#[cfg(feature = "std")]
impl std::error::Error for IdentifierError {}
