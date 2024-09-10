//! Foundational error types that are applicable across multiple ibc-rs workspaces.

use alloc::string::{FromUtf8Error, String};
use core::str::Utf8Error;

use base64::DecodeError as Base64Error;
use displaydoc::Display;
use ibc_primitives::prelude::*;
use ibc_primitives::proto::Error as ProtoError;
use prost::DecodeError as ProstError;

/// Errors that originate from host implementations.
#[derive(Debug, Display)]
pub enum HostError {
    /// invalid data: `{description}`
    InvalidData { description: String },
    /// missing data: `{description}`
    MissingData { description: String },
    /// unexpected data: `{description}`
    UnexpectedData { description: String },
    /// failed to update store: `{description}`
    FailedToUpdateStore { description: String },
    /// failed to store data: `{description}`
    FailedToStoreData { description: String },
    /// failed to retrieve data from store: `{description}`
    FailedToRetrieveFromStore { description: String },
    /// failed to parse data: `{description}`
    FailedToParseData { description: String },
    /// failed to validate client: `{description}`
    FailedToValidateClient { description: String },
    /// non-existent type: `{description}`
    NonexistentType { description: String },
    /// other error: `{description}`
    Other { description: String },
}

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

/// Errors that occur during the process of decoding, deserializing,
/// and/or converting raw types into domain types.
#[derive(Debug, Display)]
pub enum DecodingError {
    /// identifier error: `{0}`
    Identifier(IdentifierError),
    /// base64 decoding error: `{0}`
    Base64(Base64Error),
    /// utf-8 String decoding error: `{0}`
    StringUtf8(FromUtf8Error),
    /// utf-8 str decoding error: `{0}`
    StrUtf8(Utf8Error),
    /// protobuf decoding error: `{0}`
    Protobuf(ProtoError),
    /// prost decoding error: `{0}`
    Prost(ProstError),
    /// invalid hash bytes: `{description}`
    InvalidHash { description: String },
    /// invalid JSON data: `{description}`
    InvalidJson { description: String },
    /// invalid raw data: `{description}`
    InvalidRawData { description: String },
    /// missing raw data: `{description}`
    MissingRawData { description: String },
    /// mismatched type URLs: expected `{expected}`, actual `{actual}`
    MismatchedTypeUrls { expected: String, actual: String },
    /// unknown type URL: `{0}`
    UnknownTypeUrl(String),
}

impl From<ProtoError> for DecodingError {
    fn from(e: ProtoError) -> Self {
        Self::Protobuf(e)
    }
}

impl From<IdentifierError> for DecodingError {
    fn from(e: IdentifierError) -> Self {
        Self::Identifier(e)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for IdentifierError {}

#[cfg(feature = "std")]
impl std::error::Error for DecodingError {}

#[cfg(feature = "std")]
impl std::error::Error for HostError {}
