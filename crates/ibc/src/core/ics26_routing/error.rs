use crate::prelude::*;

use crate::core::ics02_client;
use crate::core::ics03_connection;
use crate::core::ics04_channel;
use displaydoc::Display;

#[derive(Debug, Display)]
pub enum Error {
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
impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self {
            Error::Client(e) => Some(e),
            Error::Connection(e) => Some(e),
            Error::Channel(e) => Some(e),
            Error::UnknownMessageTypeUrl { .. } => None,
            Error::MalformedMessageBytes(e) => Some(e),
            Error::Packet(e) => Some(e),
        }
    }
}
