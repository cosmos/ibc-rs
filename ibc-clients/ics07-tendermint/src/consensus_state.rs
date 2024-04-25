//! This module includes trait implementations for the
//! `ibc_client_tendermint_types::ConsensusState` type. It implements the
//! `ConsensusStateTrait` for `ConsensusState` by defining a newtype wrapper in
//! order to circumvent Rust's orphan rule, which disallows foreign traits from
//! being implemented on foreign types. This module also includes some trait
//! implementations that serve to pass through traits implemented on the wrapped
//! `ConsensusState` type.

use ibc_client_tendermint_types::error::Error;
use ibc_client_tendermint_types::proto::v1::ConsensusState as RawTmConsensusState;
use ibc_client_tendermint_types::ConsensusState as ConsensusStateType;
use ibc_core_client::context::consensus_state::ConsensusState as ConsensusStateTrait;
use ibc_core_client::types::error::ClientError;
use ibc_core_commitment_types::commitment::CommitmentRoot;
use ibc_primitives::prelude::*;
use ibc_primitives::proto::{Any, Protobuf};
use ibc_primitives::Timestamp;
use tendermint::{Hash, Time};

/// Newtype wrapper around the `ConsensusState` type imported from the
/// `ibc-client-tendermint-types` crate. This wrapper exists so that we can
/// bypass Rust's orphan rules and implement traits from
/// `ibc::core::client::context` on the `ConsensusState` type.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq, derive_more::From)]
pub struct ConsensusState(ConsensusStateType);

impl ConsensusState {
    pub fn inner(&self) -> &ConsensusStateType {
        &self.0
    }

    pub fn into_inner(self) -> ConsensusStateType {
        self.0
    }

    pub fn timestamp(&self) -> Time {
        self.0.timestamp
    }

    pub fn next_validators_hash(&self) -> Hash {
        self.0.next_validators_hash
    }
}

impl Protobuf<RawTmConsensusState> for ConsensusState {}

impl TryFrom<RawTmConsensusState> for ConsensusState {
    type Error = Error;

    fn try_from(raw: RawTmConsensusState) -> Result<Self, Self::Error> {
        Ok(Self(ConsensusStateType::try_from(raw)?))
    }
}

impl From<ConsensusState> for RawTmConsensusState {
    fn from(consensus_state: ConsensusState) -> Self {
        consensus_state.0.into()
    }
}

impl Protobuf<Any> for ConsensusState {}

impl TryFrom<Any> for ConsensusState {
    type Error = ClientError;

    fn try_from(raw: Any) -> Result<Self, Self::Error> {
        Ok(Self(ConsensusStateType::try_from(raw)?))
    }
}

impl From<ConsensusState> for Any {
    fn from(client_state: ConsensusState) -> Self {
        client_state.0.into()
    }
}

impl From<tendermint::block::Header> for ConsensusState {
    fn from(header: tendermint::block::Header) -> Self {
        Self(ConsensusStateType::from(header))
    }
}

impl ConsensusStateTrait for ConsensusState {
    fn root(&self) -> &CommitmentRoot {
        &self.0.root
    }

    fn timestamp(&self) -> Timestamp {
        self.0.timestamp.into()
    }
}
