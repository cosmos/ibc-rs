use crate::clients::ics06_solomachine::error::Error;
use crate::core::ics02_client::error::ClientError;
use crate::core::ics23_commitment::commitment::CommitmentRoot;
use crate::core::timestamp::Timestamp;
use crate::prelude::*;
use ibc_proto::google::protobuf::Any;
use ibc_proto::ibc::lightclients::solomachine::v2::ConsensusState as RawSolConsensusState;
use ibc_proto::protobuf::Protobuf;
use prost::Message;

pub const SOLOMACHINE_CONSENSUS_STATE_TYPE_URL: &str =
    "/ibc.lightclients.solomachine.v1.ConsensusState";

/// ConsensusState defines a solo machine consensus state. The sequence of a
/// consensus state is contained in the "height" key used in storing the
/// consensus state.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, PartialEq, Debug)]
pub struct ConsensusState {
    /// public key of the solo machine
    pub public_key: Option<Any>,
    /// diversifier allows the same public key to be re-used across different solo
    /// machine clients (potentially on different chains) without being considered
    /// misbehaviour.
    pub diversifier: String,
    pub timestamp: Timestamp,
}

impl ConsensusState {
    pub fn new(public_key: Option<Any>, diversifier: String, timestamp: Timestamp) -> Self {
        Self {
            public_key,
            diversifier,
            timestamp,
        }
    }

    pub fn valida_basic(&self) -> Result<(), Error> {
        todo!()
    }

    // GetPubKey unmarshals the public key into a cryptotypes.PubKey type.
    // An error is returned if the public key is nil or the cached value
    // is not a PubKey.

    // publicKey, ok := cs.PublicKey.GetCachedValue().(cryptotypes.PubKey)
    // if !ok {
    // 	return nil, errorsmod.Wrap(clienttypes.ErrInvalidConsensus, "consensus state PublicKey is not cryptotypes.PubKey")
    // }

    // return publicKey, nil
    //    }
    pub fn public_key(&self) -> Result<(), Error> {
        if self.public_key.is_none() {
            return Err(Error::EmptyConsensusStatePublicKey);
        }
        todo!()
    }
}

impl crate::core::ics02_client::consensus_state::ConsensusState for ConsensusState {
    fn root(&self) -> &CommitmentRoot {
        todo!()
    }

    fn timestamp(&self) -> Timestamp {
        self.timestamp
    }
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
            value: Protobuf::<RawSolConsensusState>::encode_vec(&consensus_state),
        }
    }
}
