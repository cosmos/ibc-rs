use displaydoc::Display;

use crate::core::ics02_client::error::ClientError;
use crate::core::ics03_connection::error::ConnectionError;
use crate::core::ics04_channel::error::ChannelError;
use crate::core::ics04_channel::error::PacketError;

/// Top-level error
#[derive(Debug, Display)]
pub enum ContextError {
    /// ICS02 Client error: {0}
    ClientError(ClientError),
    /// ICS03 Connection error: {0}
    ConnectionError(ConnectionError),
    /// ICS04 Channel error: {0}
    ChannelError(ChannelError),
    /// ICS04 Packet error: {0}
    PacketError(PacketError),
}

impl From<ClientError> for ContextError {
    fn from(err: ClientError) -> ContextError {
        Self::ClientError(err)
    }
}

impl From<ConnectionError> for ContextError {
    fn from(err: ConnectionError) -> ContextError {
        Self::ConnectionError(err)
    }
}

impl From<ChannelError> for ContextError {
    fn from(err: ChannelError) -> ContextError {
        Self::ChannelError(err)
    }
}

impl From<PacketError> for ContextError {
    fn from(err: PacketError) -> ContextError {
        Self::PacketError(err)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for ContextError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self {
            Self::ClientError(e) => Some(e),
            Self::ConnectionError(e) => Some(e),
            Self::ChannelError(e) => Some(e),
            Self::PacketError(e) => Some(e),
        }
    }
}
