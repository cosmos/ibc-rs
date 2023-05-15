use crate::clients::ics06_solomachine::error::Error;
use crate::prelude::*;
use cosmrs::crypto::PublicKey;
use ibc_proto::ibc::lightclients::solomachine::v2::HeaderData as RawHeaderData;
use ibc_proto::protobuf::Protobuf;

/// HeaderData returns the SignBytes data for update verification.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq)]
pub struct HeaderData {
    /// header public key
    pub new_pub_key: PublicKey,
    /// header diversifier
    pub new_diversifier: String,
}

impl Protobuf<RawHeaderData> for HeaderData {}

impl TryFrom<RawHeaderData> for HeaderData {
    type Error = Error;

    fn try_from(raw: RawHeaderData) -> Result<Self, Self::Error> {
        let new_pub_key = PublicKey::try_from(raw.new_pub_key.ok_or(Error::PublicKeyIsEmpty)?)
            .map_err(Error::PublicKeyParseFailed)?;
        Ok(Self {
            new_pub_key,
            new_diversifier: raw.new_diversifier,
        })
    }
}

impl From<HeaderData> for RawHeaderData {
    fn from(value: HeaderData) -> Self {
        Self {
            new_pub_key: Some(value.new_pub_key.to_any().expect("never failed")),
            new_diversifier: value.new_diversifier,
        }
    }
}
