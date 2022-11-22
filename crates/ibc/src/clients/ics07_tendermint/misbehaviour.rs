use crate::prelude::*;

use bytes::Buf;
use ibc_proto::google::protobuf::Any;
use ibc_proto::ibc::lightclients::tendermint::v1::Misbehaviour as RawMisbehaviour;
use ibc_proto::protobuf::Protobuf;
use prost::Message;
use serde::{Deserialize, Serialize};
use tendermint_light_client_verifier::ProdVerifier;

use crate::clients::ics07_tendermint::error::Error;
use crate::clients::ics07_tendermint::header::Header;
use crate::core::ics02_client::error::Error as Ics02Error;
use crate::core::ics24_host::identifier::ClientId;
use crate::Height;

pub const TENDERMINT_MISBEHAVIOR_TYPE_URL: &str = "/ibc.lightclients.tendermint.v1.Misbehaviour";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Misbehaviour {
    client_id: ClientId,
    header1: Header,
    header2: Header,
}

impl Misbehaviour {
    pub fn new(client_id: ClientId, header1: Header, header2: Header) -> Result<Self, Error> {
        if header1.signed_header.header.chain_id != header2.signed_header.header.chain_id {
            return Err(Error::invalid_raw_misbehaviour(
                "headers must have identical chain_ids".into(),
            ));
        }

        if header1.height() < header2.height() {
            return Err(Error::invalid_raw_misbehaviour(format!(
                "headers1 height is less than header2 height ({} < {})",
                header1.height(),
                header2.height()
            )));
        }

        let untrusted_state_1 = header1.as_untrusted_block_state();
        let untrusted_state_2 = header2.as_untrusted_block_state();

        let verifier = ProdVerifier::default();

        verifier.validate(&untrusted_state_1)?;
        verifier.validate(&untrusted_state_2)?;

        verifier.verify_commit(&untrusted_state_1)?;
        verifier.verify_commit(&untrusted_state_2)?;

        Ok(Self {
            client_id,
            header1,
            header2,
        })
    }

    pub fn client_id(&self) -> &ClientId {
        &self.client_id
    }

    pub fn header1(&self) -> &Header {
        &self.header1
    }

    pub fn header2(&self) -> &Header {
        &self.header2
    }
}

impl crate::core::ics02_client::misbehaviour::Misbehaviour for Misbehaviour {
    fn client_id(&self) -> &ClientId {
        &self.client_id
    }

    fn height(&self) -> Height {
        self.header1.height()
    }
}

impl Protobuf<RawMisbehaviour> for Misbehaviour {}

impl TryFrom<RawMisbehaviour> for Misbehaviour {
    type Error = Error;

    fn try_from(raw: RawMisbehaviour) -> Result<Self, Self::Error> {
        let client_id = raw
            .client_id
            .parse()
            .map_err(|_| Error::invalid_raw_client_id(raw.client_id.clone()))?;
        let header1: Header = raw
            .header_1
            .ok_or_else(|| Error::invalid_raw_misbehaviour("missing header1".into()))?
            .try_into()?;
        let header2: Header = raw
            .header_2
            .ok_or_else(|| Error::invalid_raw_misbehaviour("missing header2".into()))?
            .try_into()?;

        Self::new(client_id, header1, header2)
    }
}

impl From<Misbehaviour> for RawMisbehaviour {
    fn from(value: Misbehaviour) -> Self {
        RawMisbehaviour {
            client_id: value.client_id.to_string(),
            header_1: Some(value.header1.into()),
            header_2: Some(value.header2.into()),
        }
    }
}

impl Protobuf<Any> for Misbehaviour {}

impl TryFrom<Any> for Misbehaviour {
    type Error = Ics02Error;

    fn try_from(raw: Any) -> Result<Self, Ics02Error> {
        use core::ops::Deref;

        fn decode_misbehaviour<B: Buf>(buf: B) -> Result<Misbehaviour, Error> {
            RawMisbehaviour::decode(buf)
                .map_err(Error::decode)?
                .try_into()
        }

        match raw.type_url.as_str() {
            TENDERMINT_MISBEHAVIOR_TYPE_URL => {
                decode_misbehaviour(raw.value.deref()).map_err(Into::into)
            }
            _ => Err(Ics02Error::unknown_misbehaviour_type(raw.type_url)),
        }
    }
}

impl From<Misbehaviour> for Any {
    fn from(misbehaviour: Misbehaviour) -> Self {
        Any {
            type_url: TENDERMINT_MISBEHAVIOR_TYPE_URL.to_string(),
            value: Protobuf::<RawMisbehaviour>::encode_vec(&misbehaviour)
                .expect("encoding to `Any` from `TmMisbehaviour`"),
        }
    }
}

pub fn decode_misbehaviour<B: Buf>(buf: B) -> Result<Misbehaviour, Error> {
    RawMisbehaviour::decode(buf)
        .map_err(Error::decode)?
        .try_into()
}

impl core::fmt::Display for Misbehaviour {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        write!(
            f,
            "{} h1: {}-{} h2: {}-{}",
            self.client_id,
            self.header1.height(),
            self.header1.trusted_height,
            self.header2.height(),
            self.header2.trusted_height,
        )
    }
}
