use crate::clients::ics06_solomachine::error::Error;
use crate::core::ics02_client::error::ClientError;
use crate::core::timestamp::Timestamp;
use crate::prelude::*;
use crate::Height;
use bytes::Buf;
use core::fmt::{Display, Error as FmtError, Formatter};
use cosmrs::crypto::PublicKey;
use ibc_proto::google::protobuf::Any;
use ibc_proto::ibc::lightclients::solomachine::v2::Header as RawSmHeader;
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
    pub new_public_key: PublicKey,
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

impl Protobuf<RawSmHeader> for Header {}

impl TryFrom<RawSmHeader> for Header {
    type Error = Error;

    fn try_from(raw: RawSmHeader) -> Result<Self, Self::Error> {
        let sequence = Height::new(0, raw.sequence).map_err(Error::InvalidHeight)?;
        let timestamp =
            Timestamp::from_nanoseconds(raw.timestamp).map_err(Error::ParseTimeError)?;
        let signature = raw.signature;

        let new_public_key =
            PublicKey::try_from(raw.new_public_key.ok_or(Error::PublicKeyIsEmpty)?)
                .map_err(Error::PublicKeyParseFailed)?;
        let new_diversifier = raw.new_diversifier;
        Ok(Self {
            sequence,
            timestamp,
            signature,
            new_public_key,
            new_diversifier,
        })
    }
}

impl From<Header> for RawSmHeader {
    fn from(value: Header) -> Self {
        Self {
            sequence: value.sequence.revision_height(),
            timestamp: value.timestamp.nanoseconds(),
            signature: value.signature,
            new_public_key: Some(value.new_public_key.to_any().expect("never failed")),
            new_diversifier: value.new_diversifier,
        }
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
            value: Protobuf::<RawSmHeader>::encode_vec(&header),
        }
    }
}

pub fn decode_header<B: Buf>(buf: B) -> Result<Header, Error> {
    RawSmHeader::decode(buf).map_err(Error::Decode)?.try_into()
}
