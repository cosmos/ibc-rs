use crate::prelude::*;

use crate::core::ics02_client;
use crate::core::ics03_connection;
use crate::core::ics04_channel;
use displaydoc::Display;

#[derive(Debug, Display)]
pub enum Error {
    /// ICS02 client error(`{0}`)
    Ics02Client(ics02_client::error::Error),
    /// ICS03 connection error(`{0}`)
    Ics03Connection(ics03_connection::error::Error),
    /// ICS04 channel error(`{0}`)
    Ics04Channel(ics04_channel::error::Error),
    /// unknown type URL `{url}`
    UnknownMessageTypeUrl { url: String },
    /// the message is malformed and cannot be decoded, error(`{0}`)
    MalformedMessageBytes(ibc_proto::protobuf::Error),
}
