use crate::clients::ics06_solomachine::error::Error;
use crate::core::ics02_client::error::ClientError;
use crate::prelude::*;
use bytes::Buf;
use ibc_proto::google::protobuf::Any;
use ibc_proto::ibc::lightclients::solomachine::v1::Misbehaviour as RawSolMisbehaviour;
use ibc_proto::protobuf::Protobuf;
use prost::Message;

pub mod signature_and_data;

use signature_and_data::SignatureAndData;

pub const SOLOMACHINE_MISBEHAVIOUR_TYPE_URL: &str = "/ibc.lightclients.solomachine.v1.Misbehaviour";

/// Misbehaviour defines misbehaviour for a solo machine which consists
/// of a sequence and two signatures over different messages at that sequence.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, PartialEq)]
pub struct Misbehaviour {
    pub client_id: String,
    pub sequence: u64,
    pub signature_one: Option<SignatureAndData>,
    pub signature_two: Option<SignatureAndData>,
}

impl Protobuf<RawSolMisbehaviour> for Misbehaviour {}

impl TryFrom<RawSolMisbehaviour> for Misbehaviour {
    type Error = Error;

    fn try_from(raw: RawSolMisbehaviour) -> Result<Self, Self::Error> {
        todo!()
    }
}

impl From<Misbehaviour> for RawSolMisbehaviour {
    fn from(value: Misbehaviour) -> Self {
        todo!()
    }
}

impl Protobuf<Any> for Misbehaviour {}

impl TryFrom<Any> for Misbehaviour {
    type Error = ClientError;

    fn try_from(raw: Any) -> Result<Self, ClientError> {
        use core::ops::Deref;

        fn decode_misbehaviour<B: Buf>(buf: B) -> Result<Misbehaviour, Error> {
            RawSolMisbehaviour::decode(buf)
                .map_err(Error::Decode)?
                .try_into()
        }

        match raw.type_url.as_str() {
            SOLOMACHINE_MISBEHAVIOUR_TYPE_URL => {
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
            type_url: SOLOMACHINE_MISBEHAVIOUR_TYPE_URL.to_string(),
            value: Protobuf::<RawSolMisbehaviour>::encode_vec(&misbehaviour)
                .expect("encoding to `Any` from `RawSolMisbehaviour`"),
        }
    }
}

pub fn decode_misbehaviour<B: Buf>(buf: B) -> Result<Misbehaviour, Error> {
    RawSolMisbehaviour::decode(buf)
        .map_err(Error::Decode)?
        .try_into()
}

impl core::fmt::Display for Misbehaviour {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        todo!()
    }
}
