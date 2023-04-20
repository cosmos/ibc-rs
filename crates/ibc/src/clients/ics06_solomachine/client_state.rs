use crate::clients::ics06_solomachine::consensus_state::ConsensusState;
use crate::clients::ics06_solomachine::error::Error;
use crate::core::ics02_client::error::ClientError;
use crate::prelude::*;
use ibc_proto::google::protobuf::Any;
use ibc_proto::ibc::lightclients::solomachine::v1::ClientState as RawSolClientState;
use ibc_proto::protobuf::Protobuf;
use prost::Message;

pub const SOLOMACHINE_CLIENT_STATE_TYPE_URL: &str = "/ibc.lightclients.solomachine.v1.ClientState";

/// ClientState defines a solo machine client that tracks the current consensus
/// state and if the client is frozen.
#[derive(Clone, PartialEq)]
pub struct ClientState {
    /// latest sequence of the client state
    pub sequence: u64,
    /// frozen sequence of the solo machine
    pub frozen_sequence: u64,
    pub consensus_state: Option<ConsensusState>,
    /// when set to true, will allow governance to update a solo machine client.
    /// The client will be unfrozen if it is frozen.
    pub allow_update_after_proposal: bool,
}

impl Protobuf<RawSolClientState> for ClientState {}

impl TryFrom<RawSolClientState> for ClientState {
    type Error = Error;

    fn try_from(raw: RawSolClientState) -> Result<Self, Self::Error> {
        todo!()
    }
}

impl From<ClientState> for RawSolClientState {
    fn from(value: ClientState) -> Self {
        todo!()
    }
}

impl Protobuf<Any> for ClientState {}

impl TryFrom<Any> for ClientState {
    type Error = ClientError;

    fn try_from(raw: Any) -> Result<Self, Self::Error> {
        use bytes::Buf;
        use core::ops::Deref;

        fn decode_client_state<B: Buf>(buf: B) -> Result<ClientState, Error> {
            RawSolClientState::decode(buf)
                .map_err(Error::Decode)?
                .try_into()
        }

        match raw.type_url.as_str() {
            SOLOMACHINE_CLIENT_STATE_TYPE_URL => {
                decode_client_state(raw.value.deref()).map_err(Into::into)
            }
            _ => Err(ClientError::UnknownClientStateType {
                client_state_type: raw.type_url,
            }),
        }
    }
}

impl From<ClientState> for Any {
    fn from(client_state: ClientState) -> Self {
        Any {
            type_url: SOLOMACHINE_CLIENT_STATE_TYPE_URL.to_string(),
            value: Protobuf::<RawSolClientState>::encode_vec(&client_state)
                .expect("encoding to `Any` from `TmClientState`"),
        }
    }
}
