use crate::prelude::*;

use bytes::Buf;
use ibc_proto::google::protobuf::Any;
use ibc_proto::ibc::mock::Misbehaviour as RawMisbehaviour;
use ibc_proto::protobuf::Protobuf;

use crate::core::ics02_client::error::ClientError;
use crate::core::ics24_host::identifier::ClientId;
use crate::mock::header::MockHeader;

pub const MOCK_MISBEHAVIOUR_TYPE_URL: &str = "/ibc.mock.Misbehavior";

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Misbehaviour {
    pub client_id: ClientId,
    pub header1: MockHeader,
    pub header2: MockHeader,
}

impl Protobuf<RawMisbehaviour> for Misbehaviour {}

impl TryFrom<RawMisbehaviour> for Misbehaviour {
    type Error = ClientError;

    fn try_from(raw: RawMisbehaviour) -> Result<Self, Self::Error> {
        Ok(Self {
            client_id: Default::default(),
            header1: raw
                .header1
                .ok_or(ClientError::MissingRawMisbehaviour)?
                .try_into()?,
            header2: raw
                .header2
                .ok_or(ClientError::MissingRawMisbehaviour)?
                .try_into()?,
        })
    }
}

impl From<Misbehaviour> for RawMisbehaviour {
    fn from(value: Misbehaviour) -> Self {
        RawMisbehaviour {
            client_id: value.client_id.to_string(),
            header1: Some(value.header1.into()),
            header2: Some(value.header2.into()),
        }
    }
}

impl Protobuf<Any> for Misbehaviour {}

impl TryFrom<Any> for Misbehaviour {
    type Error = ClientError;

    fn try_from(raw: Any) -> Result<Self, ClientError> {
        use core::ops::Deref;

        fn decode_misbehaviour<B: Buf>(buf: B) -> Result<Misbehaviour, ClientError> {
            use prost::Message;

            RawMisbehaviour::decode(buf)
                .map_err(ClientError::Decode)?
                .try_into()
        }

        match raw.type_url.as_str() {
            MOCK_MISBEHAVIOUR_TYPE_URL => {
                decode_misbehaviour(raw.value.deref()).map_err(Into::into)
            }
            _ => Err(ClientError::UnknownMisbehaviourType {
                misbehaviour_type: raw.type_url,
            }),
        }
    }
}

impl From<Misbehaviour> for Any {
    fn from(misbehaviour: Misbehaviour) -> Self {
        Any {
            type_url: MOCK_MISBEHAVIOUR_TYPE_URL.to_string(),
            value: Protobuf::<RawMisbehaviour>::encode_vec(&misbehaviour),
        }
    }
}
