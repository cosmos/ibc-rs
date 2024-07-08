//! Defines the consensus state type for the ICS-08 Wasm light client.

#[cfg(feature = "schema")]
use cosmwasm_schema::cw_serde;
use cosmwasm_std::Binary;
use ibc_core_client::types::error::ClientError;
use ibc_primitives::prelude::*;
use ibc_primitives::proto::{Any, Protobuf};
use ibc_proto::ibc::lightclients::wasm::v1::ConsensusState as RawConsensusState;

pub const WASM_CONSENSUS_STATE_TYPE_URL: &str = "/ibc.lightclients.wasm.v1.ConsensusState";

#[cfg_attr(feature = "schema", cw_serde)]
#[cfg_attr(not(feature = "schema"), derive(Clone, Debug, PartialEq))]
#[derive(Eq)]
pub struct ConsensusState {
    pub data: Binary,
}

impl ConsensusState {
    pub fn new(data: Binary) -> Self {
        Self { data }
    }
}

impl Protobuf<RawConsensusState> for ConsensusState {}

impl From<ConsensusState> for RawConsensusState {
    fn from(value: ConsensusState) -> Self {
        Self {
            data: value.data.into(),
        }
    }
}

impl From<RawConsensusState> for ConsensusState {
    fn from(value: RawConsensusState) -> Self {
        Self {
            data: value.data.into(),
        }
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
        fn decode_consensus_state(value: &[u8]) -> Result<ConsensusState, ClientError> {
            let consensus_state =
                Protobuf::<RawConsensusState>::decode(value).map_err(|e| ClientError::Other {
                    description: e.to_string(),
                })?;
            Ok(consensus_state)
        }
        match any.type_url.as_str() {
            WASM_CONSENSUS_STATE_TYPE_URL => decode_consensus_state(&any.value),
            _ => Err(ClientError::Other {
                description: "type_url does not match".into(),
            }),
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
        let msg = ConsensusState::from(raw_msg.clone());
        assert_eq!(RawConsensusState::from(msg.clone()), raw_msg);
        assert_eq!(
            ConsensusState::try_from(Any::from(msg.clone())).unwrap(),
            msg
        );
    }
}
