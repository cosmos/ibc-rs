use crate::clients::ics06_solomachine::error::Error;
use crate::core::ics02_client::error::ClientError;
use crate::prelude::*;
use ibc_proto::google::protobuf::Any;
use ibc_proto::ibc::lightclients::solomachine::v1::ConsensusState as RawSolConsensusState;
use ibc_proto::protobuf::Protobuf;
use prost::Message;

pub const SOLOMACHINE_CONSENSUS_STATE_TYPE_URL: &str =
    "/ibc.lightclients.solomachine.v1.ConsensusState";

/// ConsensusState defines a solo machine consensus state. The sequence of a
/// consensus state is contained in the "height" key used in storing the
/// consensus state.
#[derive(Clone, PartialEq)]
pub struct ConsensusState {
    /// public key of the solo machine
    pub public_key: Option<Any>,
    /// diversifier allows the same public key to be re-used across different solo
    /// machine clients (potentially on different chains) without being considered
    /// misbehaviour.
    pub diversifier: ::prost::alloc::string::String,
    pub timestamp: u64,
}

impl Protobuf<RawSolConsensusState> for ConsensusState {}

impl TryFrom<RawSolConsensusState> for ConsensusState {
    type Error = Error;

    fn try_from(raw: RawSolConsensusState) -> Result<Self, Self::Error> {
        todo!()
    }
}

impl From<ConsensusState> for RawSolConsensusState {
    fn from(value: ConsensusState) -> Self {
        todo!()
    }
}

impl Protobuf<Any> for ConsensusState {}

impl TryFrom<Any> for ConsensusState {
    type Error = ClientError;

    fn try_from(raw: Any) -> Result<Self, Self::Error> {
        use bytes::Buf;
        use core::ops::Deref;

        fn decode_consensus_state<B: Buf>(buf: B) -> Result<ConsensusState, Error> {
            RawSolConsensusState::decode(buf)
                .map_err(Error::Decode)?
                .try_into()
        }

        match raw.type_url.as_str() {
            SOLOMACHINE_CONSENSUS_STATE_TYPE_URL => {
                decode_consensus_state(raw.value.deref()).map_err(Into::into)
            }
            _ => Err(ClientError::UnknownConsensusStateType {
                consensus_state_type: raw.type_url,
            }),
        }
    }
}

impl From<ConsensusState> for Any {
    fn from(consensus_state: ConsensusState) -> Self {
        Any {
            type_url: SOLOMACHINE_CONSENSUS_STATE_TYPE_URL.to_string(),
            value: Protobuf::<RawSolConsensusState>::encode_vec(&consensus_state)
                .expect("encoding to `Any` from `TmConsensusState`"),
        }
    }
}
