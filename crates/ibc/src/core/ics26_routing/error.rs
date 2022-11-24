use crate::prelude::*;

use crate::core::ics02_client;
use crate::core::ics03_connection;
use crate::core::ics04_channel;
use displaydoc::Display;

#[derive(Debug, Display)]
pub enum RouterError {
    /// ICS02 client error
    Client(ics02_client::error::ClientError),
    /// ICS03 connection error
    Connection(ics03_connection::error::ConnectionError),
    /// ICS04 channel error
    Channel(ics04_channel::error::ChannelError),
    /// ICS04 Packet error
    Packet(ics04_channel::error::PacketError),
    /// unknown type URL `{url}`
    UnknownMessageTypeUrl { url: String },
    /// the message is malformed and cannot be decoded
    MalformedMessageBytes(ibc_proto::protobuf::Error),
}

#[cfg(feature = "std")]
impl std::error::Error for RouterError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self {
            Self::Client(e) => Some(e),
            Self::Connection(e) => Some(e),
            Self::Channel(e) => Some(e),
            Self::UnknownMessageTypeUrl { .. } => None,
            Self::MalformedMessageBytes(e) => Some(e),
            Self::Packet(e) => Some(e),
        }
    }
}
