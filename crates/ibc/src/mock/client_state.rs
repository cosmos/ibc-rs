use crate::core::ics24_host::path::{ClientConsensusStatePath, ClientStatePath};
use crate::core::timestamp::Timestamp;
use crate::prelude::*;

use core::time::Duration;
use ibc_proto::ibc::core::commitment::v1::MerkleProof;

use ibc_proto::google::protobuf::Any;
use ibc_proto::ibc::mock::ClientState as RawMockClientState;
use ibc_proto::protobuf::Protobuf;

use crate::core::ics02_client::client_state::{
    StaticClientStateBase, StaticClientStateExecution, StaticClientStateInitializer,
    StaticClientStateValidation, UpdateKind,
};
use crate::core::ics02_client::client_type::ClientType;
use crate::core::ics02_client::error::{ClientError, UpgradeClientError};
use crate::core::ics23_commitment::commitment::{
    CommitmentPrefix, CommitmentProofBytes, CommitmentRoot,
};
use crate::core::ics24_host::identifier::ClientId;
use crate::core::ics24_host::path::Path;
use crate::mock::client_state::client_type as mock_client_type;
use crate::mock::consensus_state::MockConsensusState;
use crate::mock::header::MockHeader;
use crate::mock::misbehaviour::Misbehaviour;

use crate::Height;

use crate::core::ContextError;

pub const MOCK_CLIENT_STATE_TYPE_URL: &str = "/ibc.mock.ClientState";

pub const MOCK_CLIENT_TYPE: &str = "9999-mock";

pub fn client_type() -> ClientType {
    ClientType::from(MOCK_CLIENT_TYPE.to_string())
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
            value: Protobuf::<RawMockClientState>::encode_vec(&client_state),
        }
    }
}

impl StaticClientStateBase for MockClientState {
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

    fn expired(&self, _elapsed: Duration) -> bool {
        false
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
            return Err(UpgradeClientError::LowUpgradeHeight {
                upgraded_height: self.latest_height(),
                client_height: upgraded_mock_client_state.latest_height(),
            })?;
        }
        Ok(())
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
}

impl<ClientValidationContext> StaticClientStateValidation<ClientValidationContext>
    for MockClientState
{
    fn verify_client_message(
        &self,
        _ctx: &ClientValidationContext,
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

    fn check_for_misbehaviour(
        &self,
        _ctx: &ClientValidationContext,
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
}

pub trait MockClientExecutionContext {
    /// Returns the current height of the local chain.
    fn host_height(&self) -> Result<Height, ContextError>;

    /// Returns the current timestamp of the local chain.
    fn host_timestamp(&self) -> Result<Timestamp, ContextError>;

    fn store_update_time(
        &mut self,
        client_id: ClientId,
        height: Height,
        timestamp: Timestamp,
    ) -> Result<(), ContextError>;

    fn store_update_height(
        &mut self,
        client_id: ClientId,
        height: Height,
        host_height: Height,
    ) -> Result<(), ContextError>;

    fn store_client_state(
        &mut self,
        client_state_path: ClientStatePath,
        client_state: MockClientState,
    ) -> Result<(), ContextError>;

    fn store_consensus_state(
        &mut self,
        consensus_state_path: ClientConsensusStatePath,
        consensus_state: MockConsensusState,
    ) -> Result<(), ContextError>;
}

impl<ClientExecutionContext> StaticClientStateExecution<ClientExecutionContext> for MockClientState
where
    ClientExecutionContext: MockClientExecutionContext,
{
    fn update_state(
        &self,
        ctx: &mut ClientExecutionContext,
        client_id: &ClientId,
        header: Any,
    ) -> Result<Vec<Height>, ClientError> {
        let header = MockHeader::try_from(header)?;
        let header_height = header.height;

        let new_client_state = MockClientState::new(header);
        let new_consensus_state = MockConsensusState::new(header);

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
        ctx: &mut ClientExecutionContext,
        client_id: &ClientId,
        _client_message: Any,
        _update_kind: &UpdateKind,
    ) -> Result<(), ClientError> {
        let frozen_client_state = self.with_frozen_height(Height::new(0, 1).unwrap());

        ctx.store_client_state(ClientStatePath::new(client_id), frozen_client_state)?;

        Ok(())
    }

    fn update_state_with_upgrade_client(
        &self,
        ctx: &mut ClientExecutionContext,
        client_id: &ClientId,
        upgraded_client_state: Any,
        upgraded_consensus_state: Any,
    ) -> Result<Height, ClientError> {
        let new_client_state = MockClientState::try_from(upgraded_client_state)?;
        let new_consensus_state = MockConsensusState::try_from(upgraded_consensus_state)?;

        let latest_height = new_client_state.latest_height();

        ctx.store_consensus_state(
            ClientConsensusStatePath::new(client_id, &latest_height),
            new_consensus_state,
        )?;
        ctx.store_client_state(ClientStatePath::new(client_id), new_client_state)?;

        Ok(latest_height)
    }
}

impl<SupportedConsensusStates> StaticClientStateInitializer<SupportedConsensusStates>
    for MockClientState
where
    SupportedConsensusStates: From<MockConsensusState>,
{
    fn initialise(&self, consensus_state: Any) -> Result<SupportedConsensusStates, ClientError> {
        let consensus_state = MockConsensusState::try_from(consensus_state)?;

        Ok(consensus_state.into())
    }
}

impl From<MockConsensusState> for MockClientState {
    fn from(cs: MockConsensusState) -> Self {
        Self::new(cs.header)
    }
}
