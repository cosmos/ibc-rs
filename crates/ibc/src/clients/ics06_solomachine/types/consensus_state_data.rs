use crate::clients::ics06_solomachine::consensus_state::ConsensusState;
use crate::clients::ics06_solomachine::error::Error;
use crate::prelude::*;
use ibc_proto::ibc::lightclients::solomachine::v2::ConsensusStateData as RawConsensusStateData;
use ibc_proto::protobuf::Protobuf;

/// ConsensusStateData returns the SignBytes data for consensus state
/// verification.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq)]
pub struct ConsensusStateData {
    pub path: Vec<u8>,
    // ics06 solomachine client consensus state
    pub consensus_state: ConsensusState,
}

impl Protobuf<RawConsensusStateData> for ConsensusStateData {}

impl TryFrom<RawConsensusStateData> for ConsensusStateData {
    type Error = Error;

    fn try_from(raw: RawConsensusStateData) -> Result<Self, Self::Error> {
        Ok(Self {
            path: raw.path,
            consensus_state: raw
                .consensus_state
                .ok_or(Error::ConsensusStateIsEmpty)?
                .try_into()
                .map_err(Error::ClientError)?,
        })
    }
}

impl From<ConsensusStateData> for RawConsensusStateData {
    fn from(value: ConsensusStateData) -> Self {
        Self {
            path: value.path,
            consensus_state: Some(value.consensus_state.into()),
        }
    }
}
