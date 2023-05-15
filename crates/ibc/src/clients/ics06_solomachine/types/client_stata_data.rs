use crate::clients::ics06_solomachine::client_state::ClientState;
use crate::clients::ics06_solomachine::error::Error;
use crate::prelude::*;
use ibc_proto::ibc::lightclients::solomachine::v2::ClientStateData as RawClientStateData;
use ibc_proto::protobuf::Protobuf;

/// ClientStateData returns the SignBytes data for client state verification.
#[derive(Clone, PartialEq)]
pub struct ClientStateData {
    pub path: Vec<u8>,
    // Ics06 solomachine client state
    pub client_state: ClientState,
}

impl Protobuf<RawClientStateData> for ClientStateData {}

impl TryFrom<RawClientStateData> for ClientStateData {
    type Error = Error;

    fn try_from(raw: RawClientStateData) -> Result<Self, Self::Error> {
        Ok(Self {
            path: raw.path,
            client_state: raw
                .client_state
                .ok_or(Error::ClientStateIsEmpty)?
                .try_into()
                .map_err(Error::ClientError)?,
        })
    }
}

impl From<ClientStateData> for RawClientStateData {
    fn from(value: ClientStateData) -> Self {
        Self {
            path: value.path,
            client_state: Some(value.client_state.into()),
        }
    }
}
