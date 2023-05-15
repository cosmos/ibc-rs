use crate::clients::ics06_solomachine::error::Error;
use crate::core::ics02_client::error::ClientError;
use crate::core::ics24_host::identifier::ClientId;
use crate::{prelude::*, Height};
use bytes::Buf;
use ibc_proto::google::protobuf::Any;
use ibc_proto::ibc::lightclients::solomachine::v2::Misbehaviour as RawSmMisbehaviour;
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
    pub client_id: ClientId,
    pub sequence: Height,
    pub signature_one: Option<SignatureAndData>,
    pub signature_two: Option<SignatureAndData>,
}

impl Protobuf<RawSmMisbehaviour> for Misbehaviour {}

impl TryFrom<RawSmMisbehaviour> for Misbehaviour {
    type Error = Error;

    fn try_from(raw: RawSmMisbehaviour) -> Result<Self, Self::Error> {
        let client_id: ClientId = raw
            .client_id
            .parse()
            .map_err(|_| Error::InvalidRawClientId {
                client_id: raw.client_id.clone(),
            })?;
        let sequence = Height::new(0, raw.sequence).map_err(Error::InvalidHeight)?;
        let signature_one: Option<SignatureAndData> =
            raw.signature_one.map(TryInto::try_into).transpose()?;
        let signature_two: Option<SignatureAndData> =
            raw.signature_two.map(TryInto::try_into).transpose()?;
        Ok(Self {
            client_id,
            sequence,
            signature_one,
            signature_two,
        })
    }
}

impl From<Misbehaviour> for RawSmMisbehaviour {
    fn from(value: Misbehaviour) -> Self {
        let client_id = value.client_id.to_string();
        let sequence = value.sequence.revision_height();

        Self {
            client_id,
            sequence,
            signature_one: value.signature_one.map(Into::into),
            signature_two: value.signature_two.map(Into::into),
        }
    }
}

impl Protobuf<Any> for Misbehaviour {}

impl TryFrom<Any> for Misbehaviour {
    type Error = ClientError;

    fn try_from(raw: Any) -> Result<Self, ClientError> {
        use core::ops::Deref;

        fn decode_misbehaviour<B: Buf>(buf: B) -> Result<Misbehaviour, Error> {
            RawSmMisbehaviour::decode(buf)
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
            value: Protobuf::<RawSmMisbehaviour>::encode_vec(&misbehaviour),
        }
    }
}

pub fn decode_misbehaviour<B: Buf>(buf: B) -> Result<Misbehaviour, Error> {
    RawSmMisbehaviour::decode(buf)
        .map_err(Error::Decode)?
        .try_into()
}

impl core::fmt::Display for Misbehaviour {
    fn fmt(&self, _f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        todo!()
    }
}
