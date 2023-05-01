use crate::core::ics24_host::path::{ClientConsensusStatePath, ClientStatePath};
use crate::prelude::*;

use alloc::collections::btree_map::BTreeMap as HashMap;
use core::time::Duration;
use ibc_proto::ibc::core::commitment::v1::MerkleProof;

use ibc_proto::google::protobuf::Any;
use ibc_proto::ibc::mock::ClientState as RawMockClientState;
use ibc_proto::protobuf::Protobuf;

use crate::core::ics02_client::client_state::{ClientState, UpdateKind, UpdatedState};
use crate::core::ics02_client::consensus_state::ConsensusState;
use crate::core::ics02_client::error::ClientError;
use crate::core::ics23_commitment::commitment::{
    CommitmentPrefix, CommitmentProofBytes, CommitmentRoot,
};
use crate::core::ics24_host::identifier::{ChainId, ClientId, ClientType};
use crate::core::ics24_host::Path;
use crate::mock::client_state::client_type as mock_client_type;
use crate::mock::consensus_state::MockConsensusState;
use crate::mock::header::MockHeader;
use crate::mock::misbehaviour::Misbehaviour;

use crate::Height;

use crate::core::{ExecutionContext, ValidationContext};

pub const MOCK_CLIENT_STATE_TYPE_URL: &str = "/ibc.mock.ClientState";

pub const MOCK_CLIENT_TYPE: &str = "9999-mock";

pub fn client_type() -> ClientType {
    ClientType::from(MOCK_CLIENT_TYPE.to_string())
}

/// A mock of an IBC client record as it is stored in a mock context.
/// For testing ICS02 handlers mostly, cf. `MockClientContext`.
#[derive(Clone, Debug)]
pub struct MockClientRecord {
    /// The client state (representing only the latest height at the moment).
    pub client_state: Option<Box<dyn ClientState>>,

    /// Mapping of heights to consensus states for this client.
    pub consensus_states: HashMap<Height, Box<dyn ConsensusState>>,
}

/// A mock of a client state. For an example of a real structure that this mocks, you can see
/// `ClientState` of ics07_tendermint/client_state.rs.

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct MockClientState {
    pub header: MockHeader,
    pub frozen_height: Option<Height>,
}

impl MockClientState {
    pub fn new(header: MockHeader) -> Self {
        Self {
            header,
            frozen_height: None,
        }
    }

    pub fn latest_height(&self) -> Height {
        self.header.height()
    }

    pub fn refresh_time(&self) -> Option<Duration> {
        None
    }

    pub fn with_frozen_height(self, frozen_height: Height) -> Self {
        Self {
            frozen_height: Some(frozen_height),
            ..self
        }
    }
}

impl Protobuf<RawMockClientState> for MockClientState {}

impl TryFrom<RawMockClientState> for MockClientState {
    type Error = ClientError;

    fn try_from(raw: RawMockClientState) -> Result<Self, Self::Error> {
        Ok(Self::new(raw.header.unwrap().try_into()?))
    }
}

impl From<MockClientState> for RawMockClientState {
    fn from(value: MockClientState) -> Self {
        RawMockClientState {
            header: Some(ibc_proto::ibc::mock::Header {
                height: Some(value.header.height().into()),
                timestamp: value.header.timestamp.nanoseconds(),
            }),
        }
    }
}

impl Protobuf<Any> for MockClientState {}

impl TryFrom<Any> for MockClientState {
    type Error = ClientError;

    fn try_from(raw: Any) -> Result<Self, Self::Error> {
        use bytes::Buf;
        use core::ops::Deref;
        use prost::Message;

        fn decode_client_state<B: Buf>(buf: B) -> Result<MockClientState, ClientError> {
            RawMockClientState::decode(buf)
                .map_err(ClientError::Decode)?
                .try_into()
        }

        match raw.type_url.as_str() {
            MOCK_CLIENT_STATE_TYPE_URL => {
                decode_client_state(raw.value.deref()).map_err(Into::into)
            }
            _ => Err(ClientError::UnknownClientStateType {
                client_state_type: raw.type_url,
            }),
        }
    }
}

impl From<MockClientState> for Any {
    fn from(client_state: MockClientState) -> Self {
        Any {
            type_url: MOCK_CLIENT_STATE_TYPE_URL.to_string(),
            value: Protobuf::<RawMockClientState>::encode_vec(&client_state)
                .expect("encoding to `Any` from `MockClientState`"),
        }
    }
}

impl ClientState for MockClientState {
    fn chain_id(&self) -> ChainId {
        unimplemented!()
    }

    fn client_type(&self) -> ClientType {
        mock_client_type()
    }

    fn latest_height(&self) -> Height {
        self.header.height()
    }

    fn validate_proof_height(&self, proof_height: Height) -> Result<(), ClientError> {
        if self.latest_height() < proof_height {
            return Err(ClientError::InvalidProofHeight {
                latest_height: self.latest_height(),
                proof_height,
            });
        }
        Ok(())
    }

    fn confirm_not_frozen(&self) -> Result<(), ClientError> {
        if let Some(frozen_height) = self.frozen_height {
            return Err(ClientError::ClientFrozen {
                description: format!("The client is frozen at height {frozen_height}"),
            });
        }
        Ok(())
    }

    fn zero_custom_fields(&mut self) {
        unimplemented!()
    }

    fn expired(&self, _elapsed: Duration) -> bool {
        false
    }

    fn initialise(&self, consensus_state: Any) -> Result<Box<dyn ConsensusState>, ClientError> {
        MockConsensusState::try_from(consensus_state).map(MockConsensusState::into_box)
    }

    fn verify_upgrade_client(
        &self,
        upgraded_client_state: Any,
        upgraded_consensus_state: Any,
        _proof_upgrade_client: MerkleProof,
        _proof_upgrade_consensus_state: MerkleProof,
        _root: &CommitmentRoot,
    ) -> Result<(), ClientError> {
        let upgraded_mock_client_state = MockClientState::try_from(upgraded_client_state)?;
        MockConsensusState::try_from(upgraded_consensus_state)?;
        if self.latest_height() >= upgraded_mock_client_state.latest_height() {
            return Err(ClientError::LowUpgradeHeight {
                upgraded_height: self.latest_height(),
                client_height: upgraded_mock_client_state.latest_height(),
            });
        }
        Ok(())
    }

    fn update_state_with_upgrade_client(
        &self,
        upgraded_client_state: Any,
        upgraded_consensus_state: Any,
    ) -> Result<UpdatedState, ClientError> {
        let mock_client_state = MockClientState::try_from(upgraded_client_state)?;
        let mock_consensus_state = MockConsensusState::try_from(upgraded_consensus_state)?;
        Ok(UpdatedState {
            client_state: mock_client_state.into_box(),
            consensus_state: mock_consensus_state.into_box(),
        })
    }

    fn verify_membership(
        &self,
        _prefix: &CommitmentPrefix,
        _proof: &CommitmentProofBytes,
        _root: &CommitmentRoot,
        _path: Path,
        _value: Vec<u8>,
    ) -> Result<(), ClientError> {
        Ok(())
    }

    fn verify_non_membership(
        &self,
        _prefix: &CommitmentPrefix,
        _proof: &CommitmentProofBytes,
        _root: &CommitmentRoot,
        _path: Path,
    ) -> Result<(), ClientError> {
        Ok(())
    }

    fn verify_client_message(
        &self,
        _ctx: &dyn ValidationContext,
        _client_id: &ClientId,
        client_message: Any,
        update_kind: &UpdateKind,
    ) -> Result<(), ClientError> {
        match update_kind {
            UpdateKind::UpdateClient => {
                let header = MockHeader::try_from(client_message)?;

                if self.latest_height() >= header.height() {
                    return Err(ClientError::LowHeaderHeight {
                        header_height: header.height(),
                        latest_height: self.latest_height(),
                    });
                }
            }
            UpdateKind::SubmitMisbehaviour => {
                let _misbehaviour = Misbehaviour::try_from(client_message)?;
            }
        }

        Ok(())
    }

    /// We consider the following to be misbehaviour:
    /// + UpdateHeader: no misbehaviour possible
    /// + Misbehaviour: headers are at same height and are in the future
    fn check_for_misbehaviour(
        &self,
        _ctx: &dyn ValidationContext,
        _client_id: &ClientId,
        client_message: Any,
        update_kind: &UpdateKind,
    ) -> Result<bool, ClientError> {
        match update_kind {
            UpdateKind::UpdateClient => Ok(false),
            UpdateKind::SubmitMisbehaviour => {
                let misbehaviour = Misbehaviour::try_from(client_message)?;
                let header_1 = misbehaviour.header1;
                let header_2 = misbehaviour.header2;

                let header_heights_equal = header_1.height() == header_2.height();
                let headers_are_in_future = self.latest_height() < header_1.height();

                Ok(header_heights_equal && headers_are_in_future)
            }
        }
    }

    fn update_state(
        &self,
        ctx: &mut dyn ExecutionContext,
        client_id: &ClientId,
        header: Any,
    ) -> Result<Vec<Height>, ClientError> {
        let header = MockHeader::try_from(header)?;
        let header_height = header.height;

        let new_client_state = MockClientState::new(header).into_box();
        let new_consensus_state = MockConsensusState::new(header).into_box();

        ctx.store_update_time(
            client_id.clone(),
            new_client_state.latest_height(),
            ctx.host_timestamp()?,
        )?;
        ctx.store_update_height(
            client_id.clone(),
            new_client_state.latest_height(),
            ctx.host_height()?,
        )?;
        ctx.store_consensus_state(
            ClientConsensusStatePath::new(client_id, &new_client_state.latest_height()),
            new_consensus_state,
        )?;
        ctx.store_client_state(ClientStatePath::new(client_id), new_client_state)?;

        Ok(vec![header_height])
    }

    fn update_state_on_misbehaviour(
        &self,
        ctx: &mut dyn ExecutionContext,
        client_id: &ClientId,
        _client_message: Any,
        _update_kind: &UpdateKind,
    ) -> Result<(), ClientError> {
        let frozen_client_state = self
            .with_frozen_height(Height::new(0, 1).unwrap())
            .into_box();

        ctx.store_client_state(ClientStatePath::new(client_id), frozen_client_state)?;

        Ok(())
    }
}

impl From<MockConsensusState> for MockClientState {
    fn from(cs: MockConsensusState) -> Self {
        Self::new(cs.header)
    }
}
