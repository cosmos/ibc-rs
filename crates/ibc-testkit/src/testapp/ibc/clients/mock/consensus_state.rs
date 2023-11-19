use ibc::core::client::context::consensus_state::ConsensusState;
use ibc::core::client::types::error::ClientError;
use ibc::core::commitment_types::commitment::CommitmentRoot;
use ibc::core::primitives::prelude::*;
use ibc::core::primitives::Timestamp;
use ibc::proto::{Any, Protobuf};

use crate::testapp::ibc::clients::mock::header::MockHeader;
use crate::testapp::ibc::clients::mock::proto::{
    ConsensusState as RawMockConsensusState, Header as RawMockHeader,
};
pub const MOCK_CONSENSUS_STATE_TYPE_URL: &str = "/ibc.mock.ConsensusState";

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MockConsensusState {
    pub header: MockHeader,
    pub root: CommitmentRoot,
}

impl MockConsensusState {
    pub fn new(header: MockHeader) -> Self {
        MockConsensusState {
            header,
            root: CommitmentRoot::from(vec![0]),
        }
    }

    pub fn timestamp(&self) -> Timestamp {
        self.header.timestamp
    }
}

impl Protobuf<RawMockConsensusState> for MockConsensusState {}

impl TryFrom<RawMockConsensusState> for MockConsensusState {
    type Error = ClientError;

    fn try_from(raw: RawMockConsensusState) -> Result<Self, Self::Error> {
        let raw_header = raw.header.ok_or(ClientError::MissingRawConsensusState)?;

        Ok(Self {
            header: MockHeader::try_from(raw_header)?,
            root: CommitmentRoot::from(vec![0]),
        })
    }
}

impl From<MockConsensusState> for RawMockConsensusState {
    fn from(value: MockConsensusState) -> Self {
        RawMockConsensusState {
            header: Some(RawMockHeader {
                height: Some(value.header.height().into()),
                timestamp: value.header.timestamp.nanoseconds(),
            }),
        }
    }
}

impl Protobuf<Any> for MockConsensusState {}

impl TryFrom<Any> for MockConsensusState {
    type Error = ClientError;

    fn try_from(raw: Any) -> Result<Self, Self::Error> {
        use core::ops::Deref;

        use bytes::Buf;
        use prost::Message;

        fn decode_consensus_state<B: Buf>(buf: B) -> Result<MockConsensusState, ClientError> {
            RawMockConsensusState::decode(buf)
                .map_err(ClientError::Decode)?
                .try_into()
        }

        match raw.type_url.as_str() {
            MOCK_CONSENSUS_STATE_TYPE_URL => {
                decode_consensus_state(raw.value.deref()).map_err(Into::into)
            }
            _ => Err(ClientError::UnknownConsensusStateType {
                consensus_state_type: raw.type_url,
            }),
        }
    }
}

impl From<MockConsensusState> for Any {
    fn from(consensus_state: MockConsensusState) -> Self {
        Any {
            type_url: MOCK_CONSENSUS_STATE_TYPE_URL.to_string(),
            value: Protobuf::<RawMockConsensusState>::encode_vec(consensus_state),
        }
    }
}

impl ConsensusState for MockConsensusState {
    fn root(&self) -> &CommitmentRoot {
        &self.root
    }

    fn timestamp(&self) -> Timestamp {
        self.header.timestamp
    }

    fn encode_vec(self) -> Vec<u8> {
        <Self as Protobuf<Any>>::encode_vec(self)
    }
}
