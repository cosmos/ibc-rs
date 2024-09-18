use ibc::core::client::context::consensus_state::ConsensusState;
use ibc::core::commitment_types::commitment::CommitmentRoot;
use ibc::core::host::types::error::DecodingError;
use ibc::core::primitives::prelude::*;
use ibc::core::primitives::Timestamp;
use ibc::primitives::proto::{Any, Protobuf};

use crate::testapp::ibc::clients::mock::header::MockHeader;
use crate::testapp::ibc::clients::mock::proto::ConsensusState as RawMockConsensusState;
pub const MOCK_CONSENSUS_STATE_TYPE_URL: &str = "/ibc.mock.ConsensusState";

/// The mock consensus state type used within ibc-testkit for testing situations
/// when a consensus state is required.
///
/// Note, this type slightly differs from the [`RawMockConsensusState`] type exposed by
/// ibc-proto. It contains a (private) `root` field to easily return a
/// reference to the mock consensus state's dummy [`CommitmentRoot`].
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
    type Error = DecodingError;

    fn try_from(raw: RawMockConsensusState) -> Result<Self, Self::Error> {
        let raw_header = raw
            .header
            .ok_or(DecodingError::missing_raw_data("missing header"))?;

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
    type Error = DecodingError;

    fn try_from(raw: Any) -> Result<Self, Self::Error> {
        fn decode_consensus_state(value: &[u8]) -> Result<MockConsensusState, DecodingError> {
            let mock_consensus_state = Protobuf::<RawMockConsensusState>::decode(value)?;
            Ok(mock_consensus_state)
        }
        match raw.type_url.as_str() {
            MOCK_CONSENSUS_STATE_TYPE_URL => decode_consensus_state(&raw.value),
            _ => Err(DecodingError::MismatchedTypeUrls {
                expected: MOCK_CONSENSUS_STATE_TYPE_URL.to_string(),
                actual: raw.type_url,
            })?,
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
