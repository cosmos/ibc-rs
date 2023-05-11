use crate::clients::ics06_solomachine::error::Error;
use crate::prelude::*;
use ibc_proto::google::protobuf::Any;
use ibc_proto::ibc::lightclients::solomachine::v2::HeaderData as RawHeaderData;
use ibc_proto::protobuf::Protobuf;

/// HeaderData returns the SignBytes data for update verification.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq)]
pub struct HeaderData {
    /// header public key
    pub new_pub_key: Option<Any>,
    /// header diversifier
    pub new_diversifier: String,
}

impl Protobuf<RawHeaderData> for HeaderData {}

impl TryFrom<RawHeaderData> for HeaderData {
    type Error = Error;

    fn try_from(raw: RawHeaderData) -> Result<Self, Self::Error> {
        todo!()
    }
}

impl From<HeaderData> for RawHeaderData {
    fn from(value: HeaderData) -> Self {
        todo!()
    }
}
