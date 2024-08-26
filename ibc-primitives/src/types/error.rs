//! Foundational error types that are applicable across multiple ibc-rs workspaces.

use alloc::string::String;

use displaydoc::Display;
use http::uri::InvalidUri;

/// Causes of decoding failures
#[derive(Debug, Display)]
pub enum DecodingError {
    /// invalid identifier error: `{0}`
    InvalidIdentifier(String),
    /// invalid field: `{0}`
    InvalidField(String),
    /// invalid JSON data: `{description}`
    InvalidJson { description: String },
    /// invalid UTF-8 data: `{description}`
    InvalidUtf8 { description: String },
    /// invalid URI: `{0}`
    InvalidUri(InvalidUri),
    /// missing field: `{0}`
    MissingField(String),
    /// mismatched type URLs: expected `{expected}`, actual `{actual}`
    MismatchedTypeUrls { expected: String, actual: String },
    /// failed to decode a raw value: `{description}`
    FailedToDecodeRawValue { description: String },
}

#[cfg(feature = "std")]
impl std::error::Error for DecodingError {}
