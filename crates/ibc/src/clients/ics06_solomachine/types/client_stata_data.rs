use crate::clients::ics06_solomachine::client_state::ClientState;
use crate::clients::ics06_solomachine::error::Error;
use crate::prelude::*;
use ibc_proto::ibc::lightclients::solomachine::v1::ClientStateData as RawClientStateData;
use ibc_proto::protobuf::Protobuf;

/// ClientStateData returns the SignBytes data for client state verification.
#[derive(Clone, PartialEq)]
pub struct ClientStateData {
    pub path: Vec<u8>,
    // Ics06 solomachine client state
    pub client_state: Option<ClientState>,
}

impl Protobuf<RawClientStateData> for ClientStateData {}

impl TryFrom<RawClientStateData> for ClientStateData {
    type Error = Error;

    fn try_from(raw: RawClientStateData) -> Result<Self, Self::Error> {
        todo!()
    }
}

impl From<ClientStateData> for RawClientStateData {
    fn from(value: ClientStateData) -> Self {
        todo!()
    }
}
