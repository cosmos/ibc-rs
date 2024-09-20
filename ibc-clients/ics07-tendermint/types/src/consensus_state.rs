//! Defines Tendermint's `ConsensusState` type

use ibc_core_commitment_types::commitment::CommitmentRoot;
use ibc_core_host_types::error::DecodingError;
use ibc_primitives::prelude::*;
use ibc_proto::google::protobuf::Any;
use ibc_proto::ibc::lightclients::tendermint::v1::ConsensusState as RawConsensusState;
use ibc_proto::Protobuf;
use tendermint::hash::Algorithm;
use tendermint::time::Time;
use tendermint::Hash;
use tendermint_proto::google::protobuf as tpb;

use crate::header::Header;

pub const TENDERMINT_CONSENSUS_STATE_TYPE_URL: &str =
    "/ibc.lightclients.tendermint.v1.ConsensusState";

/// Defines the Tendermint light client's consensus state
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConsensusState {
    pub timestamp: Time,
    pub root: CommitmentRoot,
    pub next_validators_hash: Hash,
}

impl ConsensusState {
    pub fn new(root: CommitmentRoot, timestamp: Time, next_validators_hash: Hash) -> Self {
        Self {
            timestamp,
            root,
            next_validators_hash,
        }
    }

    pub fn timestamp(&self) -> Time {
        self.timestamp
    }

    pub fn root(&self) -> CommitmentRoot {
        self.root.clone()
    }
}

impl Protobuf<RawConsensusState> for ConsensusState {}

impl TryFrom<RawConsensusState> for ConsensusState {
    type Error = DecodingError;

    fn try_from(raw: RawConsensusState) -> Result<Self, Self::Error> {
        let proto_root = raw
            .root
            .ok_or(DecodingError::missing_raw_data(
                "consensus state commitment root",
            ))?
            .hash;

        let ibc_proto::google::protobuf::Timestamp { seconds, nanos } = raw
            .timestamp
            .ok_or(DecodingError::missing_raw_data("consensus state timestamp"))?;
        // FIXME: shunts like this are necessary due to
        // https://github.com/informalsystems/tendermint-rs/issues/1053
        let proto_timestamp = tpb::Timestamp { seconds, nanos };
        let timestamp = proto_timestamp
            .try_into()
            .map_err(|e| DecodingError::invalid_raw_data(format!("timestamp: {e}")))?;

        let next_validators_hash = Hash::from_bytes(Algorithm::Sha256, &raw.next_validators_hash)
            .map_err(|e| {
            DecodingError::invalid_raw_data(format!("next validators hash: {e}"))
        })?;

        Ok(Self {
            root: proto_root.into(),
            timestamp,
            next_validators_hash,
        })
    }
}

impl From<ConsensusState> for RawConsensusState {
    fn from(value: ConsensusState) -> Self {
        // FIXME: shunts like this are necessary due to
        // https://github.com/informalsystems/tendermint-rs/issues/1053
        let tpb::Timestamp { seconds, nanos } = value.timestamp.into();
        let timestamp = ibc_proto::google::protobuf::Timestamp { seconds, nanos };

        RawConsensusState {
            timestamp: Some(timestamp),
            root: Some(ibc_proto::ibc::core::commitment::v1::MerkleRoot {
                hash: value.root.into_vec(),
            }),
            next_validators_hash: value.next_validators_hash.as_bytes().to_vec(),
        }
    }
}

impl Protobuf<Any> for ConsensusState {}

impl TryFrom<Any> for ConsensusState {
    type Error = DecodingError;

    fn try_from(raw: Any) -> Result<Self, Self::Error> {
        fn decode_consensus_state(value: &[u8]) -> Result<ConsensusState, DecodingError> {
            let client_state = Protobuf::<RawConsensusState>::decode(value)?;
            Ok(client_state)
        }

        match raw.type_url.as_str() {
            TENDERMINT_CONSENSUS_STATE_TYPE_URL => decode_consensus_state(&raw.value),
            _ => Err(DecodingError::MismatchedResourceName {
                expected: TENDERMINT_CONSENSUS_STATE_TYPE_URL.to_string(),
                actual: raw.type_url,
            })?,
        }
    }
}

impl From<ConsensusState> for Any {
    fn from(consensus_state: ConsensusState) -> Self {
        Any {
            type_url: TENDERMINT_CONSENSUS_STATE_TYPE_URL.to_string(),
            value: Protobuf::<RawConsensusState>::encode_vec(consensus_state),
        }
    }
}

impl From<tendermint::block::Header> for ConsensusState {
    fn from(header: tendermint::block::Header) -> Self {
        Self {
            root: CommitmentRoot::from_bytes(header.app_hash.as_ref()),
            timestamp: header.time,
            next_validators_hash: header.next_validators_hash,
        }
    }
}

impl From<Header> for ConsensusState {
    fn from(header: Header) -> Self {
        Self::from(header.signed_header.header)
    }
}
