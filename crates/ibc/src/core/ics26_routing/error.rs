use crate::core::context::ContextError;
use crate::prelude::*;

use displaydoc::Display;

#[derive(Debug, Display)]
pub enum RouterError {
    /// context error
    ContextError(ContextError),
    /// unknown type URL `{url}`
    UnknownMessageTypeUrl { url: String },
    /// the message is malformed and cannot be decoded
    MalformedMessageBytes(ibc_proto::protobuf::Error),
}

#[cfg(feature = "std")]
impl std::error::Error for RouterError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self {
            Self::ContextError(e) => Some(e),
            Self::UnknownMessageTypeUrl { .. } => None,
            Self::MalformedMessageBytes(e) => Some(e),
        }
    }
}
