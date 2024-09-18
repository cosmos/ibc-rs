//! Defines the handler error type

use derive_more::From;
use displaydoc::Display;
use ibc_core_channel_types::error::ChannelError;
use ibc_core_client_types::error::ClientError;
use ibc_core_connection_types::error::ConnectionError;
use ibc_core_router_types::error::RouterError;
use ibc_primitives::prelude::*;

/// Top-level type that surfaces errors from the core ibc-rs crates.
#[derive(Debug, Display, From)]
pub enum HandlerError {
    /// ICS02 Client error: {0}
    Client(ClientError),
    /// ICS03 Connection error: {0}
    Connection(ConnectionError),
    /// ICS04 Channel error: {0}
    Channel(ChannelError),
    /// ICS26 Routing error: {0}
    Router(RouterError),
}

#[cfg(feature = "std")]
impl std::error::Error for HandlerError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Client(e) => Some(e),
            Self::Connection(e) => Some(e),
            Self::Channel(e) => Some(e),
            Self::Router(e) => Some(e),
        }
    }
}
