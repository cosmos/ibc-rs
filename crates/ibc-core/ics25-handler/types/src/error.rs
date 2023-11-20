//! Defines the context error type

use derive_more::From;
use displaydoc::Display;
use ibc_core_channel_types::error::{ChannelError, PacketError};
use ibc_core_client_types::error::ClientError;
use ibc_core_connection_types::error::ConnectionError;
use ibc_core_router_types::error::RouterError;
use ibc_primitives::prelude::*;

/// Top-level error
#[derive(Debug, Display, From)]
pub enum ContextError {
    /// ICS02 Client error: {0}
    ClientError(ClientError),
    /// ICS03 Connection error: {0}
    ConnectionError(ConnectionError),
    /// ICS04 Channel error: {0}
    ChannelError(ChannelError),
    /// ICS04 Packet error: {0}
    PacketError(PacketError),
    /// ICS26 Routing error: {0}
    RouterError(RouterError),
}

impl From<ContextError> for ClientError {
    fn from(context_error: ContextError) -> Self {
        match context_error {
            ContextError::ClientError(e) => e,
            _ => ClientError::Other {
                description: context_error.to_string(),
            },
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for ContextError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::ClientError(e) => Some(e),
            Self::ConnectionError(e) => Some(e),
            Self::ChannelError(e) => Some(e),
            Self::PacketError(e) => Some(e),
            Self::RouterError(e) => Some(e),
        }
    }
}
