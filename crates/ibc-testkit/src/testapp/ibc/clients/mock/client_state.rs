use core::str::FromStr;
use core::time::Duration;

use ibc::core::ics02_client::client_state::{
    ClientStateCommon, ClientStateExecution, ClientStateValidation, Status, UpdateKind,
};
use ibc::core::ics02_client::client_type::ClientType;
use ibc::core::ics02_client::error::{ClientError, UpgradeClientError};
use ibc::core::ics02_client::{ClientExecutionContext, ClientValidationContext};
use ibc::core::ics23_commitment::commitment::{
    CommitmentPrefix, CommitmentProofBytes, CommitmentRoot,
};
use ibc::core::ics24_host::identifier::ClientId;
use ibc::core::ics24_host::path::{ClientConsensusStatePath, ClientStatePath, Path};
use ibc::core::timestamp::Timestamp;
use ibc::core::ContextError;
use ibc::prelude::*;
use ibc::proto::mock::ClientState as RawMockClientState;
use ibc::proto::{Any, Protobuf};
use ibc::Height;

use crate::testapp::ibc::clients::mock::client_state::client_type as mock_client_type;
use crate::testapp::ibc::clients::mock::consensus_state::MockConsensusState;
use crate::testapp::ibc::clients::mock::header::MockHeader;
use crate::testapp::ibc::clients::mock::misbehaviour::Misbehaviour;

pub const MOCK_CLIENT_STATE_TYPE_URL: &str = "/ibc.mock.ClientState";
pub const MOCK_CLIENT_TYPE: &str = "9999-mock";

pub fn client_type() -> ClientType {
    ClientType::from_str(MOCK_CLIENT_TYPE).expect("never fails because it's valid client type")
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

    pub fn is_frozen(&self) -> bool {
        self.frozen_height.is_some()
    }

    fn expired(&self, _elapsed: Duration) -> bool {
        false
    }
}

impl Protobuf<RawMockClientState> for MockClientState {}

impl TryFrom<RawMockClientState> for MockClientState {
    type Error = ClientError;

    fn try_from(raw: RawMockClientState) -> Result<Self, Self::Error> {
        Ok(Self::new(raw.header.expect("Never fails").try_into()?))
    }
}

impl From<MockClientState> for RawMockClientState {
    fn from(value: MockClientState) -> Self {
        RawMockClientState {
            header: Some(ibc::proto::mock::Header {
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
        use core::ops::Deref;

        use bytes::Buf;
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
            value: Protobuf::<RawMockClientState>::encode_vec(client_state),
        }
    }
}

pub trait MockClientContext {
    type ConversionError: ToString;
    type AnyConsensusState: TryInto<MockConsensusState, Error = Self::ConversionError>;

    /// Returns the current timestamp of the local chain.
    fn host_timestamp(&self) -> Result<Timestamp, ContextError>;

    /// Returns the current height of the local chain.
    fn host_height(&self) -> Result<Height, ContextError>;

    /// Retrieve the consensus state for the given client ID at the specified
    /// height.
    ///
    /// Returns an error if no such state exists.
    fn consensus_state(
        &self,
        client_cons_state_path: &ClientConsensusStatePath,
    ) -> Result<Self::AnyConsensusState, ContextError>;
}

impl ClientStateCommon for MockClientState {
    fn verify_consensus_state(&self, consensus_state: Any) -> Result<(), ClientError> {
        let _mock_consensus_state = MockConsensusState::try_from(consensus_state)?;

        Ok(())
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

    fn verify_upgrade_client(
        &self,
        upgraded_client_state: Any,
        upgraded_consensus_state: Any,
        _proof_upgrade_client: CommitmentProofBytes,
        _proof_upgrade_consensus_state: CommitmentProofBytes,
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

impl<V> ClientStateValidation<V> for MockClientState
where
    V: ClientValidationContext + MockClientContext,
    V::AnyConsensusState: TryInto<MockConsensusState>,
    ClientError: From<<V::AnyConsensusState as TryInto<MockConsensusState>>::Error>,
{
    fn verify_client_message(
        &self,
        _ctx: &V,
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
        _ctx: &V,
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

    fn status(&self, ctx: &V, client_id: &ClientId) -> Result<Status, ClientError> {
        if self.is_frozen() {
            return Ok(Status::Frozen);
        }

        let latest_consensus_state: MockConsensusState = {
            let any_latest_consensus_state = match ctx.consensus_state(
                &ClientConsensusStatePath::new(client_id, &self.latest_height()),
            ) {
                Ok(cs) => cs,
                // if the client state does not have an associated consensus state for its latest height
                // then it must be expired
                Err(_) => return Ok(Status::Expired),
            };

            any_latest_consensus_state.try_into()?
        };

        let now = ctx.host_timestamp()?;
        let elapsed_since_latest_consensus_state = now
            .duration_since(&latest_consensus_state.timestamp())
            .ok_or(ClientError::Other {
                description: format!("latest consensus state is in the future. now: {now}, latest consensus state: {}", latest_consensus_state.timestamp()),
            })?;

        if self.expired(elapsed_since_latest_consensus_state) {
            return Ok(Status::Expired);
        }

        Ok(Status::Active)
    }
}

impl<E> ClientStateExecution<E> for MockClientState
where
    E: ClientExecutionContext + MockClientContext,
    <E as ClientExecutionContext>::AnyClientState: From<MockClientState>,
    <E as ClientExecutionContext>::AnyConsensusState: From<MockConsensusState>,
{
    fn initialise(
        &self,
        ctx: &mut E,
        client_id: &ClientId,
        consensus_state: Any,
    ) -> Result<(), ClientError> {
        let mock_consensus_state = MockConsensusState::try_from(consensus_state)?;

        ctx.store_client_state(ClientStatePath::new(client_id), (*self).into())?;
        ctx.store_consensus_state(
            ClientConsensusStatePath::new(client_id, &self.latest_height()),
            mock_consensus_state.into(),
        )?;

        Ok(())
    }

    fn update_state(
        &self,
        ctx: &mut E,
        client_id: &ClientId,
        header: Any,
    ) -> Result<Vec<Height>, ClientError> {
        let header = MockHeader::try_from(header)?;
        let header_height = header.height;

        let new_client_state = MockClientState::new(header);
        let new_consensus_state = MockConsensusState::new(header);

        ctx.store_consensus_state(
            ClientConsensusStatePath::new(client_id, &new_client_state.latest_height()),
            new_consensus_state.into(),
        )?;
        ctx.store_client_state(ClientStatePath::new(client_id), new_client_state.into())?;
        ctx.store_update_time(client_id.clone(), header_height, ctx.host_timestamp()?)?;
        ctx.store_update_height(client_id.clone(), header_height, ctx.host_height()?)?;

        Ok(vec![header_height])
    }

    fn update_state_on_misbehaviour(
        &self,
        ctx: &mut E,
        client_id: &ClientId,
        _client_message: Any,
        _update_kind: &UpdateKind,
    ) -> Result<(), ClientError> {
        let frozen_client_state = self.with_frozen_height(Height::min(0));

        ctx.store_client_state(ClientStatePath::new(client_id), frozen_client_state.into())?;

        Ok(())
    }

    fn update_state_on_upgrade(
        &self,
        ctx: &mut E,
        client_id: &ClientId,
        upgraded_client_state: Any,
        upgraded_consensus_state: Any,
    ) -> Result<Height, ClientError> {
        let new_client_state = MockClientState::try_from(upgraded_client_state)?;
        let new_consensus_state = MockConsensusState::try_from(upgraded_consensus_state)?;

        let latest_height = new_client_state.latest_height();

        ctx.store_consensus_state(
            ClientConsensusStatePath::new(client_id, &latest_height),
            new_consensus_state.into(),
        )?;
        ctx.store_client_state(ClientStatePath::new(client_id), new_client_state.into())?;

        let host_timestamp = ctx.host_timestamp()?;
        let host_height = ctx.host_height()?;

        ctx.store_update_time(client_id.clone(), latest_height, host_timestamp)?;
        ctx.store_update_height(client_id.clone(), latest_height, host_height)?;

        Ok(latest_height)
    }
}

impl From<MockConsensusState> for MockClientState {
    fn from(cs: MockConsensusState) -> Self {
        Self::new(cs.header)
    }
}
