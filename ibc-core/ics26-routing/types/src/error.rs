use displaydoc::Display;
use ibc_primitives::prelude::*;

/// Error type for the router module.
#[derive(Debug, Display, derive_more::From)]
pub enum RouterError {
    /// missing module
    MissingModule,
}

#[cfg(feature = "std")]
impl std::error::Error for RouterError {}
