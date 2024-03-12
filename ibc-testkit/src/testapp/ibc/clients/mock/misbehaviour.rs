use ibc::core::client::types::error::ClientError;
use ibc::core::host::types::identifiers::ClientId;
use ibc::core::primitives::prelude::*;
use ibc::primitives::proto::{Any, Protobuf};

use crate::testapp::ibc::clients::mock::header::MockHeader;
use crate::testapp::ibc::clients::mock::proto::Misbehaviour as RawMisbehaviour;

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
            client_id: ClientId::new("07-tendermint", 0).expect("no error"),
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
        fn decode_misbehaviour(value: &[u8]) -> Result<Misbehaviour, ClientError> {
            let raw_misbehaviour =
                Protobuf::<RawMisbehaviour>::decode(value).map_err(|e| ClientError::Other {
                    description: e.to_string(),
                })?;
            Ok(raw_misbehaviour)
        }
        match raw.type_url.as_str() {
            MOCK_MISBEHAVIOUR_TYPE_URL => decode_misbehaviour(&raw.value),
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
            value: Protobuf::<RawMisbehaviour>::encode_vec(misbehaviour),
        }
    }
}
