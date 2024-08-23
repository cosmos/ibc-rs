use displaydoc::Display;
use ibc_core_host_types::identifiers::PortId;
use ibc_primitives::prelude::*;

/// Error type for the router module.
#[derive(Debug, Display)]
pub enum RouterError {
    /// malformed message that could not be decoded: `{description}`
    MalformedMessageBytes { description: String },
    /// missing module
    MissingModule,
    /// unknown message type URL `{0}`
    UnknownMessageTypeUrl(String),
    /// unknown port `{0}`
    UnknownPort(PortId),
}

#[cfg(feature = "std")]
impl std::error::Error for RouterError {}
