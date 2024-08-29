use displaydoc::Display;
use ibc_core_host_types::error::DecodingError;
use ibc_core_host_types::identifiers::PortId;
use ibc_primitives::prelude::*;

/// Error type for the router module.
#[derive(Debug, Display, derive_more::From)]
pub enum RouterError {
    /// decoding error: `{0}`
    Decoding(DecodingError),
    /// missing module
    MissingModule,
    /// unknown port `{0}`
    UnknownPort(PortId),
}

#[cfg(feature = "std")]
impl std::error::Error for RouterError {}
