//! Foundational error types that are applicable across multiple ibc-rs workspaces.

use alloc::string::String;

use displaydoc::Display;

/// Causes of decoding failures
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[derive(Debug, Display)]
pub enum DecodingError {
    /// invalid identifier: `{0}`
    InvalidIdentifier(String),
    /// invalid field: `{0}`
    InvalidField(String),
    /// invalid JSON data: `{description}`
    InvalidJson { description: String },
    /// invalid UTF-8 data: `{description}`
    InvalidUtf8 { description: String },
    /// missing field: `{0}`
    MissingField(String),
    /// mismatched type URLs: expected `{expected}`, actual `{actual}`
    MismatchedTypeUrls { expected: String, actual: String },
    /// failed to decode: `{description}`
    FailedToDecode { description: String },
}

#[cfg(feature = "std")]
impl std::error::Error for DecodingError {}
