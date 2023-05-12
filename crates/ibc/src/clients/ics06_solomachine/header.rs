use crate::clients::ics06_solomachine::error::Error;
use crate::core::ics02_client::error::ClientError;
use crate::core::timestamp::Timestamp;
use crate::prelude::*;
use crate::Height;
use bytes::Buf;
use core::fmt::{Display, Error as FmtError, Formatter};
use ibc_proto::google::protobuf::Any;
use ibc_proto::ibc::lightclients::solomachine::v2::Header as RawSolHeader;
use ibc_proto::protobuf::Protobuf;
use prost::Message;
pub const SOLOMACHINE_HEADER_TYPE_URL: &str = "/ibc.lightclients.solomachine.v1.Header";

/// Header defines a solo machine consensus header
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, PartialEq)]
pub struct Header {
    /// sequence to update solo machine public key at
    pub sequence: Height,
    pub timestamp: Timestamp,
    pub signature: Vec<u8>,
    pub new_public_key: Option<Any>,
    pub new_diversifier: String,
}

impl core::fmt::Debug for Header {
    fn fmt(&self, _f: &mut Formatter<'_>) -> Result<(), FmtError> {
        todo!()
    }
}

impl Display for Header {
    fn fmt(&self, _f: &mut Formatter<'_>) -> Result<(), FmtError> {
        todo!()
    }
}

impl crate::core::ics02_client::header::Header for Header {
    fn height(&self) -> Height {
        self.sequence
    }

    fn timestamp(&self) -> Timestamp {
        self.timestamp
    }
}

impl Protobuf<RawSolHeader> for Header {}

impl TryFrom<RawSolHeader> for Header {
    type Error = Error;

    fn try_from(_raw: RawSolHeader) -> Result<Self, Self::Error> {
        todo!()
    }
}

impl From<Header> for RawSolHeader {
    fn from(_value: Header) -> Self {
        todo!()
    }
}

impl Protobuf<Any> for Header {}

impl TryFrom<Any> for Header {
    type Error = ClientError;

    fn try_from(raw: Any) -> Result<Self, Self::Error> {
        use core::ops::Deref;

        match raw.type_url.as_str() {
            SOLOMACHINE_HEADER_TYPE_URL => decode_header(raw.value.deref()).map_err(Into::into),
            _ => Err(ClientError::UnknownHeaderType {
                header_type: raw.type_url,
            }),
        }
    }
}

impl From<Header> for Any {
    fn from(header: Header) -> Self {
        Any {
            type_url: SOLOMACHINE_HEADER_TYPE_URL.to_string(),
            value: Protobuf::<RawSolHeader>::encode_vec(&header),
        }
    }
}

pub fn decode_header<B: Buf>(buf: B) -> Result<Header, Error> {
    RawSolHeader::decode(buf).map_err(Error::Decode)?.try_into()
}
