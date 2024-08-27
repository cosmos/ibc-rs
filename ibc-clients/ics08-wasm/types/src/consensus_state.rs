//! Defines the consensus state type for the ICS-08 Wasm light client.

use ibc_core_client::types::error::ClientError;
use ibc_core_host_types::error::DecodingError;
use ibc_primitives::prelude::*;
use ibc_primitives::proto::{Any, Protobuf};
use ibc_proto::ibc::lightclients::wasm::v1::ConsensusState as RawConsensusState;

#[cfg(feature = "serde")]
use crate::serializer::Base64;
use crate::Bytes;

pub const WASM_CONSENSUS_STATE_TYPE_URL: &str = "/ibc.lightclients.wasm.v1.ConsensusState";

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ConsensusState {
    #[cfg_attr(feature = "schema", schemars(with = "String"))]
    #[cfg_attr(feature = "serde", serde(with = "Base64", default))]
    pub data: Bytes,
}

impl ConsensusState {
    pub fn new(data: Bytes) -> Self {
        Self { data }
    }
}

impl Protobuf<RawConsensusState> for ConsensusState {}

impl From<ConsensusState> for RawConsensusState {
    fn from(value: ConsensusState) -> Self {
        Self { data: value.data }
    }
}

impl TryFrom<RawConsensusState> for ConsensusState {
    type Error = ClientError;

    fn try_from(value: RawConsensusState) -> Result<Self, Self::Error> {
        Ok(Self { data: value.data })
    }
}

impl Protobuf<Any> for ConsensusState {}

impl From<ConsensusState> for Any {
    fn from(value: ConsensusState) -> Self {
        Self {
            type_url: WASM_CONSENSUS_STATE_TYPE_URL.to_string(),
            value: Protobuf::<RawConsensusState>::encode_vec(value),
        }
    }
}

impl TryFrom<Any> for ConsensusState {
    type Error = ClientError;

    fn try_from(any: Any) -> Result<Self, Self::Error> {
        fn decode_consensus_state(value: &[u8]) -> Result<ConsensusState, DecodingError> {
            let consensus_state = Protobuf::<RawConsensusState>::decode(value)?;
            Ok(consensus_state)
        }
        match any.type_url.as_str() {
            WASM_CONSENSUS_STATE_TYPE_URL => {
                decode_consensus_state(&any.value).map_err(ClientError::Decoding)
            }
            _ => Err(ClientError::Decoding(DecodingError::MismatchedTypeUrls {
                expected: WASM_CONSENSUS_STATE_TYPE_URL.to_string(),
                actual: any.type_url.to_string(),
            })),
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case(b"data")]
    fn test_roundtrip(#[case] data: &[u8]) {
        let raw_msg = RawConsensusState {
            data: data.to_vec(),
        };
        let msg = ConsensusState::try_from(raw_msg.clone()).unwrap();
        assert_eq!(RawConsensusState::from(msg.clone()), raw_msg);
        assert_eq!(
            ConsensusState::try_from(Any::from(msg.clone())).unwrap(),
            msg
        );
    }
}
