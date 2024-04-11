//! Defines the misbehaviour type for the tendermint light client

use ibc_core_client_types::error::ClientError;
use ibc_core_host_types::identifiers::ClientId;
use ibc_primitives::prelude::*;
use ibc_proto::google::protobuf::Any;
use ibc_proto::ibc::lightclients::tendermint::v1::Misbehaviour as RawMisbehaviour;
use ibc_proto::Protobuf;
use tendermint::crypto::Sha256;
use tendermint::merkle::MerkleHash;

use crate::error::Error;
use crate::header::Header;

pub const TENDERMINT_MISBEHAVIOUR_TYPE_URL: &str = "/ibc.lightclients.tendermint.v1.Misbehaviour";

/// Tendermint light client's misbehaviour type
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Misbehaviour {
    client_id: ClientId,
    header1: Box<Header>,
    header2: Box<Header>,
}

impl Misbehaviour {
    pub fn new(client_id: ClientId, header1: Header, header2: Header) -> Self {
        Self {
            client_id,
            header1: Box::new(header1),
            header2: Box::new(header2),
        }
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

    pub fn validate_basic<H: MerkleHash + Sha256 + Default>(&self) -> Result<(), Error> {
        self.header1.validate_basic::<H>()?;
        self.header2.validate_basic::<H>()?;

        if self.header1.signed_header.header.chain_id != self.header2.signed_header.header.chain_id
        {
            return Err(Error::InvalidRawMisbehaviour {
                reason: "headers must have identical chain_ids".to_owned(),
            });
        }

        if self.header1.height() < self.header2.height() {
            return Err(Error::InvalidRawMisbehaviour {
                reason: format!(
                    "header1 height is less than header2 height ({} < {})",
                    self.header1.height(),
                    self.header2.height()
                ),
            });
        }

        Ok(())
    }
}

impl Protobuf<RawMisbehaviour> for Misbehaviour {}

impl TryFrom<RawMisbehaviour> for Misbehaviour {
    type Error = Error;
    #[allow(deprecated)]
    fn try_from(raw: RawMisbehaviour) -> Result<Self, Self::Error> {
        let client_id = raw.client_id.parse()?;

        let header1: Header = raw
            .header_1
            .ok_or_else(|| Error::InvalidRawMisbehaviour {
                reason: "missing header1".into(),
            })?
            .try_into()?;

        let header2: Header = raw
            .header_2
            .ok_or_else(|| Error::InvalidRawMisbehaviour {
                reason: "missing header2".into(),
            })?
            .try_into()?;

        Ok(Self::new(client_id, header1, header2))
    }
}

impl From<Misbehaviour> for RawMisbehaviour {
    fn from(value: Misbehaviour) -> Self {
        #[allow(deprecated)]
        RawMisbehaviour {
            client_id: value.client_id.to_string(),
            header_1: Some((*value.header1).into()),
            header_2: Some((*value.header2).into()),
        }
    }
}

impl Protobuf<Any> for Misbehaviour {}

impl TryFrom<Any> for Misbehaviour {
    type Error = ClientError;

    fn try_from(raw: Any) -> Result<Self, ClientError> {
        fn decode_misbehaviour(value: &[u8]) -> Result<Misbehaviour, ClientError> {
            let misbehaviour =
                Protobuf::<RawMisbehaviour>::decode(value).map_err(|e| ClientError::Other {
                    description: e.to_string(),
                })?;
            Ok(misbehaviour)
        }
        match raw.type_url.as_str() {
            TENDERMINT_MISBEHAVIOUR_TYPE_URL => decode_misbehaviour(&raw.value),
            _ => Err(ClientError::UnknownMisbehaviourType {
                misbehaviour_type: raw.type_url,
            }),
        }
    }
}

impl From<Misbehaviour> for Any {
    fn from(misbehaviour: Misbehaviour) -> Self {
        Any {
            type_url: TENDERMINT_MISBEHAVIOUR_TYPE_URL.to_string(),
            value: Protobuf::<RawMisbehaviour>::encode_vec(misbehaviour),
        }
    }
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
