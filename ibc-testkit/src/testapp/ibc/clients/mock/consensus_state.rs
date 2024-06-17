use ibc::core::client::context::consensus_state::ConsensusState;
use ibc::core::client::types::error::ClientError;
use ibc::core::commitment_types::commitment::CommitmentRoot;
use ibc::core::primitives::prelude::*;
use ibc::core::primitives::Timestamp;
use ibc::primitives::proto::{Any, Protobuf};

use crate::testapp::ibc::clients::mock::header::MockHeader;
use crate::testapp::ibc::clients::mock::proto::ConsensusState as RawMockConsensusState;
pub const MOCK_CONSENSUS_STATE_TYPE_URL: &str = "/ibc.mock.ConsensusState";

/// The mock consensus state type used within ibc-testkit for testing situations
/// when a consensus state is required.
///
/// Note this type differs from the `mock::ConsensusState` type exposed by
/// ibc-proto in that it contains the `root` field for easily returning a
/// reference to the consensus state's `CommitmentRoot`.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MockConsensusState {
    pub header: MockHeader,
    root: CommitmentRoot,
}

impl MockConsensusState {
    pub fn new(header: MockHeader) -> Self {
        Self {
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
            header: raw_header.try_into()?,
            root: CommitmentRoot::from(vec![0]),
        })
    }
}

impl From<MockConsensusState> for RawMockConsensusState {
    fn from(value: MockConsensusState) -> Self {
        Self {
            header: Some(value.header.into()),
        }
    }
}

impl Protobuf<Any> for MockConsensusState {}

impl TryFrom<Any> for MockConsensusState {
    type Error = ClientError;

    fn try_from(raw: Any) -> Result<Self, Self::Error> {
        fn decode_consensus_state(value: &[u8]) -> Result<MockConsensusState, ClientError> {
            let mock_consensus_state =
                Protobuf::<RawMockConsensusState>::decode(value).map_err(|e| {
                    ClientError::Other {
                        description: e.to_string(),
                    }
                })?;
            Ok(mock_consensus_state)
        }
        match raw.type_url.as_str() {
            MOCK_CONSENSUS_STATE_TYPE_URL => decode_consensus_state(&raw.value),
            _ => Err(ClientError::UnknownConsensusStateType {
                consensus_state_type: raw.type_url,
            }),
        }
    }
}

impl From<MockConsensusState> for Any {
    fn from(consensus_state: MockConsensusState) -> Self {
        Self {
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
}
