use crate::prelude::*;

use alloc::collections::btree_map::BTreeMap as HashMap;
use core::time::Duration;
use ibc_proto::ibc::core::commitment::v1::MerkleProof;

use ibc_proto::google::protobuf::Any;
use ibc_proto::ibc::mock::ClientState as RawMockClientState;
use ibc_proto::protobuf::Protobuf;

use crate::core::ics02_client::client_state::{ClientState, UpdatedState};
use crate::core::ics02_client::client_type::ClientType;
use crate::core::ics02_client::consensus_state::ConsensusState;
use crate::core::ics02_client::error::ClientError;
use crate::core::ics03_connection::connection::ConnectionEnd;
use crate::core::ics04_channel::channel::ChannelEnd;
use crate::core::ics04_channel::commitment::{AcknowledgementCommitment, PacketCommitment};
use crate::core::ics04_channel::packet::Sequence;
use crate::core::ics23_commitment::commitment::{
    CommitmentPrefix, CommitmentProofBytes, CommitmentRoot,
};
use crate::core::ics23_commitment::merkle::apply_prefix;
use crate::core::ics24_host::identifier::{ChainId, ClientId};
use crate::core::ics24_host::path::{
    AckPath, ChannelEndPath, ClientConsensusStatePath, ClientStatePath, CommitmentPath,
    ConnectionPath, ReceiptPath, SeqRecvPath,
};
use crate::core::ics24_host::Path;
use crate::mock::client_state::client_type as mock_client_type;
use crate::mock::consensus_state::MockConsensusState;
use crate::mock::header::MockHeader;
use crate::mock::misbehaviour::Misbehaviour;

use crate::Height;

use crate::core::{ContextError, ValidationContext};

pub const MOCK_CLIENT_STATE_TYPE_URL: &str = "/ibc.mock.ClientState";

pub const MOCK_CLIENT_TYPE: &str = "9999-mock";

pub fn client_type() -> ClientType {
    ClientType::new(MOCK_CLIENT_TYPE.to_string())
}

/// A mock of an IBC client record as it is stored in a mock context.
/// For testing ICS02 handlers mostly, cf. `MockClientContext`.
#[derive(Clone, Debug)]
pub struct MockClientRecord {
    /// The type of this client.
    pub client_type: ClientType,

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

    fn frozen_height(&self) -> Option<Height> {
        self.frozen_height
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

    fn check_header_and_update_state(
        &self,
        _ctx: &dyn ValidationContext,
        _client_id: ClientId,
        header: Any,
    ) -> Result<UpdatedState, ClientError> {
        let header = MockHeader::try_from(header)?;

        if self.latest_height() >= header.height() {
            return Err(ClientError::LowHeaderHeight {
                header_height: header.height(),
                latest_height: self.latest_height(),
            });
        }

        Ok(UpdatedState {
            client_state: MockClientState::new(header).into_box(),
            consensus_state: MockConsensusState::new(header).into_box(),
        })
    }

    fn check_misbehaviour_and_update_state(
        &self,
        _ctx: &dyn ValidationContext,
        _client_id: ClientId,
        misbehaviour: Any,
    ) -> Result<Box<dyn ClientState>, ContextError> {
        let misbehaviour = Misbehaviour::try_from(misbehaviour)?;
        let header_1 = misbehaviour.header1;
        let header_2 = misbehaviour.header2;

        if header_1.height() != header_2.height() {
            return Err(ClientError::InvalidHeight.into());
        }

        if self.latest_height() >= header_1.height() {
            return Err(ClientError::LowHeaderHeight {
                header_height: header_1.height(),
                latest_height: self.latest_height(),
            }
            .into());
        }

        let new_state =
            MockClientState::new(header_1).with_frozen_height(Height::new(0, 1).unwrap());

        Ok(new_state.into_box())
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

    fn verify_client_consensus_state(
        &self,
        _height: Height,
        prefix: &CommitmentPrefix,
        _proof: &CommitmentProofBytes,
        _root: &CommitmentRoot,
        client_cons_state_path: &ClientConsensusStatePath,
        _expected_consensus_state: &dyn ConsensusState,
    ) -> Result<(), ClientError> {
        let client_prefixed_path =
            Path::ClientConsensusState(client_cons_state_path.clone()).to_string();

        let _path = apply_prefix(prefix, vec![client_prefixed_path]);

        Ok(())
    }

    fn verify_connection_state(
        &self,
        _height: Height,
        _prefix: &CommitmentPrefix,
        _proof: &CommitmentProofBytes,
        _root: &CommitmentRoot,
        _conn_path: &ConnectionPath,
        _expected_connection_end: &ConnectionEnd,
    ) -> Result<(), ClientError> {
        Ok(())
    }

    fn verify_channel_state(
        &self,
        _height: Height,
        _prefix: &CommitmentPrefix,
        _proof: &CommitmentProofBytes,
        _root: &CommitmentRoot,
        _chan_end_path: &ChannelEndPath,
        _expected_channel_end: &ChannelEnd,
    ) -> Result<(), ClientError> {
        Ok(())
    }

    fn verify_client_full_state(
        &self,
        _height: Height,
        _prefix: &CommitmentPrefix,
        _proof: &CommitmentProofBytes,
        _root: &CommitmentRoot,
        _client_state_path: &ClientStatePath,
        _expected_client_state: Any,
    ) -> Result<(), ClientError> {
        Ok(())
    }

    fn verify_packet_data(
        &self,
        _ctx: &dyn ValidationContext,
        _height: Height,
        _connection_end: &ConnectionEnd,
        _proof: &CommitmentProofBytes,
        _root: &CommitmentRoot,
        _commitment_path: &CommitmentPath,
        _commitment: PacketCommitment,
    ) -> Result<(), ClientError> {
        Ok(())
    }

    fn verify_next_sequence_recv(
        &self,
        _ctx: &dyn ValidationContext,
        _height: Height,
        _connection_end: &ConnectionEnd,
        _proof: &CommitmentProofBytes,
        _root: &CommitmentRoot,
        _seq_recv_path: &SeqRecvPath,
        _sequence: Sequence,
    ) -> Result<(), ClientError> {
        Ok(())
    }

    fn verify_packet_receipt_absence(
        &self,
        _ctx: &dyn ValidationContext,
        _height: Height,
        _connection_end: &ConnectionEnd,
        _proof: &CommitmentProofBytes,
        _root: &CommitmentRoot,
        _receipt_path: &ReceiptPath,
    ) -> Result<(), ClientError> {
        Ok(())
    }

    fn verify_packet_acknowledgement(
        &self,
        _ctx: &dyn ValidationContext,
        _height: Height,
        _connection_end: &ConnectionEnd,
        _proof: &CommitmentProofBytes,
        _root: &CommitmentRoot,
        _ack_path: &AckPath,
        _ack: AcknowledgementCommitment,
    ) -> Result<(), ClientError> {
        Ok(())
    }
}

impl From<MockConsensusState> for MockClientState {
    fn from(cs: MockConsensusState) -> Self {
        Self::new(cs.header)
    }
}
