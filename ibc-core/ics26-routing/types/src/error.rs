use displaydoc::Display;
use ibc_core_host_types::identifiers::PortId;
use ibc_primitives::prelude::*;

/// Error type for the router module.
#[derive(Debug, Display)]
pub enum RouterError {
    /// unknown type URL `{url}`
    UnknownMessageTypeUrl { url: String },
    /// the message is malformed and cannot be decoded error: `{reason}`
    MalformedMessageBytes { reason: String },
    /// port `{port_id}` is unknown
    UnknownPort { port_id: PortId },
    /// module not found
    ModuleNotFound,
}

#[cfg(feature = "std")]
impl std::error::Error for RouterError {}
