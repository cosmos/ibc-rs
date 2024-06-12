//! Defines the context error type
use core::fmt::{Debug, Display};

use derive_more::From;
use displaydoc::Display;
use ibc_core_channel_types::error::{ChannelError, PacketError};
use ibc_core_client_types::error::ClientError;
use ibc_core_connection_types::error::ConnectionError;
use ibc_core_router_types::error::RouterError;
use ibc_primitives::prelude::*;

/// Top-level ibc-rs error type that distinguishes between external host or internal protocol errors:
/// - Errors that originate at the host level that need to be dealt with by the IBC module.
/// - Errors that originate from internal ibc-rs code paths that need to be surfaced to the host.
#[derive(Debug, Display)]
pub enum ContextError<E: Debug + Display> {
    /// Host-defined errors
    Host(E),
    /// Internal protocol-level errors
    Protocol(ProtocolError),
}

/// Encapsulates all internal ibc-rs errors.
///
/// These are not meant to be handled by users; their primary purpose is
/// to aid in debugging.
#[derive(Debug, Display, From)]
pub enum ProtocolError {
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

impl<E: Display + Debug> From<ProtocolError> for ContextError<E> {
    fn from(protocol_error: ProtocolError) -> Self {
        Self::Protocol(protocol_error)
    }
}

impl<E: Display + Debug> From<ClientError> for ContextError<E> {
    fn from(client_error: ClientError) -> Self {
        Self::Protocol(ProtocolError::ClientError(client_error))
    }
}

impl From<ProtocolError> for ClientError {
    fn from(protocol_error: ProtocolError) -> Self {
        match protocol_error {
            ProtocolError::ClientError(e) => e,
            _ => ClientError::Other {
                description: protocol_error.to_string(),
            },
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for ProtocolError {
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
