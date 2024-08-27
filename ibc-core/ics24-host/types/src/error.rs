//! Foundational error types that are applicable across multiple ibc-rs workspaces.

use alloc::string::String;

use displaydoc::Display;
use http::uri::InvalidUri;

use ibc_primitives::prelude::*;
use tendermint_proto::Error as ProtoError;

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
    /// failed to parse `{value}`: `{description}`
    FailedToParse { value: String, description: String },
    /// overflowed revision number
    OverflowedRevisionNumber,
}

/// Errors that result in decoding failures
#[derive(Debug, Display)]
pub enum DecodingError {
    /// invalid identifier error: `{0}`
    InvalidIdentifier(IdentifierError),
    /// invalid JSON data: `{description}`
    InvalidJson { description: String },
    /// invalid UTF-8 data: `{description}`
    InvalidUtf8 { description: String },
    /// invalid URI: `{0}`
    InvalidUri(InvalidUri),
    /// malformed bytes that could not be decoded: `{description}`
    MalformedBytes { description: String },
    /// mismatched type URLs: expected `{expected}`, actual `{actual}`
    MismatchedTypeUrls { expected: String, actual: String },
    /// failed to decode proto; error: `{0}`
    FailedToDecodeProto(ProtoError),
}

impl From<ProtoError> for DecodingError {
    fn from(e: ProtoError) -> Self {
        Self::FailedToDecodeProto(e)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for IdentifierError {}

#[cfg(feature = "std")]
impl std::error::Error for DecodingError {}
