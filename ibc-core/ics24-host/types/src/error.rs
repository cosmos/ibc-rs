//! Foundational error types that are applicable across multiple ibc-rs workspaces.

use alloc::string::{FromUtf8Error, String};
use core::num::ParseIntError;
use core::str::Utf8Error;

use base64::DecodeError as Base64Error;
use displaydoc::Display;
use ibc_primitives::prelude::*;
use ibc_primitives::proto::Error as ProtoError;
use prost::DecodeError as ProstError;

/// Errors that originate from host implementations.
#[derive(Debug, Display)]
pub enum HostError {
    /// invalid state: `{description}`
    InvalidState { description: String },
    /// missing state: `{description}`
    MissingState { description: String },
    /// failed to update store: `{description}`
    FailedToStore { description: String },
    /// failed to retrieve from store: `{description}`
    FailedToRetrieve { description: String },
    /// other error: `{description}`
    Other { description: String },
}

impl HostError {
    pub fn invalid_state<T: ToString>(description: T) -> Self {
        Self::InvalidState {
            description: description.to_string(),
        }
    }

    pub fn missing_state<T: ToString>(description: T) -> Self {
        Self::MissingState {
            description: description.to_string(),
        }
    }

    pub fn failed_to_retrieve<T: ToString>(description: T) -> Self {
        Self::FailedToRetrieve {
            description: description.to_string(),
        }
    }

    pub fn failed_to_store<T: ToString>(description: T) -> Self {
        Self::FailedToStore {
            description: description.to_string(),
        }
    }
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
    /// failed to parse: `{description}`
    FailedToParse { description: String },
    /// mismatched event kind: expected {expected}, actual {actual}
    MismatchedEventKind { expected: String, actual: String },
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
    /// integer parsing error: `{0}`
    ParseInt(ParseIntError),
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

impl DecodingError {
    pub fn invalid_raw_data<T: ToString>(description: T) -> Self {
        Self::InvalidRawData {
            description: description.to_string(),
        }
    }

    pub fn missing_raw_data<T: ToString>(description: T) -> Self {
        Self::MissingRawData {
            description: description.to_string(),
        }
    }
}

impl From<IdentifierError> for DecodingError {
    fn from(e: IdentifierError) -> Self {
        Self::Identifier(e)
    }
}

impl From<ProtoError> for DecodingError {
    fn from(e: ProtoError) -> Self {
        Self::Protobuf(e)
    }
}

impl From<Base64Error> for DecodingError {
    fn from(e: Base64Error) -> Self {
        Self::Base64(e)
    }
}

impl From<FromUtf8Error> for DecodingError {
    fn from(e: FromUtf8Error) -> Self {
        Self::StringUtf8(e)
    }
}

impl From<Utf8Error> for DecodingError {
    fn from(e: Utf8Error) -> Self {
        Self::StrUtf8(e)
    }
}

impl From<ParseIntError> for DecodingError {
    fn from(e: ParseIntError) -> Self {
        Self::ParseInt(e)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for IdentifierError {}

#[cfg(feature = "std")]
impl std::error::Error for DecodingError {}

#[cfg(feature = "std")]
impl std::error::Error for HostError {}
