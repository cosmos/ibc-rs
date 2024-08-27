//! Foundational error types that are applicable across multiple ibc-rs workspaces.

use alloc::string::String;

use displaydoc::Display;
use http::uri::InvalidUri;

use tendermint_proto::Error as ProtoError;

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
    /// malformed bytes that could not be decoded: `{description}`
    MalformedBytes { description: String },
    /// missing field: `{0}`
    MissingField(String),
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
impl std::error::Error for DecodingError {}
