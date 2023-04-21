use crate::clients::ics06_solomachine::error::Error;
use crate::core::ics03_connection::connection::ConnectionEnd;
use crate::prelude::*;
use ibc_proto::ibc::lightclients::solomachine::v1::ConnectionStateData as RawConnectionStateData;
use ibc_proto::protobuf::Protobuf;

/// ConnectionStateData returns the SignBytes data for connection state
/// verification.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, PartialEq)]
pub struct ConnectionStateData {
    pub path: Vec<u8>,
    pub connection: Option<ConnectionEnd>,
}

impl Protobuf<RawConnectionStateData> for ConnectionStateData {}

impl TryFrom<RawConnectionStateData> for ConnectionStateData {
    type Error = Error;

    fn try_from(raw: RawConnectionStateData) -> Result<Self, Self::Error> {
        todo!()
    }
}

impl From<ConnectionStateData> for RawConnectionStateData {
    fn from(value: ConnectionStateData) -> Self {
        todo!()
    }
}
