//! Foundational error types that are applicable across multiple ibc-rs workspaces.

use alloc::string::{FromUtf8Error, String};

use base64::DecodeError as Base64Error;
use displaydoc::Display;
use ibc_primitives::prelude::*;
use prost::DecodeError as ProstError;
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
    /// base64 decoding error: `{0}`
    Base64(Base64Error),
    /// utf-8 decoding error: `{0}`
    Utf8(FromUtf8Error),
    /// protobuf decoding error: `{0}`
    Protobuf(ProtoError),
    /// prost decoding error: `{0}`
    Prost(ProstError),
    /// invalid JSON data: `{description}`
    InvalidJson { description: String },
    /// invalid identifier error: `{0}`
    InvalidIdentifier(IdentifierError),
    /// mismatched type URLs: expected `{expected}`, actual `{actual}`
    MismatchedTypeUrls { expected: String, actual: String },
}

impl From<ProtoError> for DecodingError {
    fn from(e: ProtoError) -> Self {
        Self::Protobuf(e)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for IdentifierError {}

#[cfg(feature = "std")]
impl std::error::Error for DecodingError {}
