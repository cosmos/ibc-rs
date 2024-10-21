use displaydoc::Display;
use ibc_core_host_types::error::HostError;
use ibc_primitives::prelude::*;

/// Error type for the router module.
#[derive(Debug, Display, derive_more::From)]
pub enum RouterError {
    /// host error: {0}
    Host(HostError),
    /// missing module
    MissingModule,
}

#[cfg(feature = "std")]
impl std::error::Error for RouterError {}
