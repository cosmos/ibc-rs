use displaydoc::Display;

use ibc_core_host_types::identifiers::PortId;
use ibc_primitives::prelude::*;

/// Error type for the router module.
#[derive(Debug, Display, derive_more::From)]
pub enum RouterError {
    /// missing module
    MissingModule,

    // TODO(seanchen1991): This variant needs to be moved to HostError
    /// unknown port `{0}`
    UnknownPort(PortId),
}

#[cfg(feature = "std")]
impl std::error::Error for RouterError {}
