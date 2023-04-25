use crate::clients::ics06_solomachine::consensus_state::ConsensusState;
use crate::clients::ics06_solomachine::error::Error;
use crate::prelude::*;
use ibc_proto::ibc::lightclients::solomachine::v1::ConsensusStateData as RawConsensusStateData;
use ibc_proto::protobuf::Protobuf;

/// ConsensusStateData returns the SignBytes data for consensus state
/// verification.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq)]
pub struct ConsensusStateData {
    pub path: Vec<u8>,
    // ics06 solomachine client consensus state
    pub consensus_state: Option<ConsensusState>,
}

impl Protobuf<RawConsensusStateData> for ConsensusStateData {}

impl TryFrom<RawConsensusStateData> for ConsensusStateData {
    type Error = Error;

    fn try_from(raw: RawConsensusStateData) -> Result<Self, Self::Error> {
        todo!()
    }
}

impl From<ConsensusStateData> for RawConsensusStateData {
    fn from(value: ConsensusStateData) -> Self {
        todo!()
    }
}
