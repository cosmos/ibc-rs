use core::str::FromStr;
use core::time::Duration;

use ibc::core::client::context::prelude::*;
use ibc::core::client::types::error::{ClientError, UpgradeClientError};
use ibc::core::client::types::{Height, Status};
use ibc::core::commitment_types::commitment::{
    CommitmentPrefix, CommitmentProofBytes, CommitmentRoot,
};
use ibc::core::handler::types::error::ContextError;
use ibc::core::host::types::identifiers::{ClientId, ClientType};
use ibc::core::host::types::path::{ClientConsensusStatePath, ClientStatePath, Path};
use ibc::core::primitives::prelude::*;
use ibc::core::primitives::Timestamp;
use ibc::primitives::proto::{Any, Protobuf};

use crate::testapp::ibc::clients::mock::client_state::client_type as mock_client_type;
use crate::testapp::ibc::clients::mock::consensus_state::MockConsensusState;
use crate::testapp::ibc::clients::mock::header::{MockHeader, MOCK_HEADER_TYPE_URL};
use crate::testapp::ibc::clients::mock::misbehaviour::{Misbehaviour, MOCK_MISBEHAVIOUR_TYPE_URL};
use crate::testapp::ibc::clients::mock::proto::ClientState as RawMockClientState;

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
    pub trusting_period: Duration,
    pub frozen: bool,
}

impl MockClientState {
    /// Initializes a new `MockClientState` with the given `MockHeader` and a
    /// trusting period of 10 seconds as a default. If the trusting period
    /// needs to be changed, use the `with_trusting_period` method to override it.
    pub fn new(header: MockHeader) -> Self {
        Self {
            header,
            trusting_period: Duration::from_secs(10),
            frozen: false,
        }
    }

    pub fn latest_height(&self) -> Height {
        self.header.height()
    }

    pub fn refresh_time(&self) -> Option<Duration> {
        None
    }

    pub fn with_trusting_period(self, trusting_period: Duration) -> Self {
        Self {
            trusting_period,
            ..self
        }
    }

    pub fn frozen(self) -> Self {
        Self {
            frozen: true,
            ..self
        }
    }

    pub fn unfrozen(self) -> Self {
        Self {
            frozen: false,
            ..self
        }
    }

    pub fn is_frozen(&self) -> bool {
        self.frozen
    }

    fn expired(&self, elapsed: Duration) -> bool {
        elapsed > self.trusting_period
    }
}

impl Protobuf<RawMockClientState> for MockClientState {}

impl TryFrom<RawMockClientState> for MockClientState {
    type Error = ClientError;

    fn try_from(raw: RawMockClientState) -> Result<Self, Self::Error> {
        Ok(Self {
            header: raw
                .header
                .ok_or(ClientError::Other {
                    description: "header is not present".into(),
                })?
                .try_into()?,
            trusting_period: Duration::from_nanos(raw.trusting_period),
            frozen: raw.frozen,
        })
    }
}

impl From<MockClientState> for RawMockClientState {
    fn from(value: MockClientState) -> Self {
        RawMockClientState {
            header: Some(value.header.into()),
            trusting_period: value
                .trusting_period
                .as_nanos()
                .try_into()
                .expect("no error"),
            frozen: value.frozen,
        }
    }
}

impl Protobuf<Any> for MockClientState {}

impl TryFrom<Any> for MockClientState {
    type Error = ClientError;

    fn try_from(raw: Any) -> Result<Self, Self::Error> {
        fn decode_client_state(value: &[u8]) -> Result<MockClientState, ClientError> {
            let client_state =
                Protobuf::<RawMockClientState>::decode(value).map_err(|e| ClientError::Other {
                    description: e.to_string(),
                })?;
            Ok(client_state)
        }
        match raw.type_url.as_str() {
            MOCK_CLIENT_STATE_TYPE_URL => decode_client_state(&raw.value),
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

pub trait ConsensusStateConverter:
    TryInto<MockConsensusState, Error = ClientError> + From<MockConsensusState>
{
}

impl<C> ConsensusStateConverter for C where
    C: TryInto<MockConsensusState, Error = ClientError> + From<MockConsensusState>
{
}

pub trait MockClientContext {
    /// Returns the current timestamp of the local chain.
    fn host_timestamp(&self) -> Result<Timestamp, ContextError>;

    /// Returns the current height of the local chain.
    fn host_height(&self) -> Result<Height, ContextError>;
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
    V::ConsensusStateRef: ConsensusStateConverter,
{
    fn verify_client_message(
        &self,
        _ctx: &V,
        _client_id: &ClientId,
        client_message: Any,
    ) -> Result<(), ClientError> {
        match client_message.type_url.as_str() {
            MOCK_HEADER_TYPE_URL => {
                let _header = MockHeader::try_from(client_message)?;
            }
            MOCK_MISBEHAVIOUR_TYPE_URL => {
                let _misbehaviour = Misbehaviour::try_from(client_message)?;
            }
            _ => {}
        }

        Ok(())
    }

    fn check_for_misbehaviour(
        &self,
        _ctx: &V,
        _client_id: &ClientId,
        client_message: Any,
    ) -> Result<bool, ClientError> {
        match client_message.type_url.as_str() {
            MOCK_HEADER_TYPE_URL => Ok(false),
            MOCK_MISBEHAVIOUR_TYPE_URL => {
                let misbehaviour = Misbehaviour::try_from(client_message)?;
                let header_1 = misbehaviour.header1;
                let header_2 = misbehaviour.header2;

                let header_heights_equal = header_1.height() == header_2.height();
                let headers_are_in_future = self.latest_height() < header_1.height();

                Ok(header_heights_equal && headers_are_in_future)
            }
            header_type => Err(ClientError::UnknownHeaderType {
                header_type: header_type.to_owned(),
            }),
        }
    }

    fn status(&self, ctx: &V, client_id: &ClientId) -> Result<Status, ClientError> {
        if self.is_frozen() {
            return Ok(Status::Frozen);
        }

        let latest_consensus_state = {
            match ctx.consensus_state(&ClientConsensusStatePath::new(
                client_id.clone(),
                self.latest_height().revision_number(),
                self.latest_height().revision_height(),
            )) {
                Ok(cs) => cs.try_into()?,
                // if the client state does not have an associated consensus state for its latest height
                // then it must be expired
                Err(_) => return Ok(Status::Expired),
            }
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

    fn check_substitute(&self, _ctx: &V, _substitute_client_state: Any) -> Result<(), ClientError> {
        Ok(())
    }
}

impl<E> ClientStateExecution<E> for MockClientState
where
    E: ClientExecutionContext + MockClientContext,
    E::ClientStateRef: From<MockClientState>,
    E::ConsensusStateRef: ConsensusStateConverter,
{
    fn initialise(
        &self,
        ctx: &mut E,
        client_id: &ClientId,
        consensus_state: Any,
    ) -> Result<(), ClientError> {
        let mock_consensus_state = MockConsensusState::try_from(consensus_state)?;

        ctx.store_client_state(ClientStatePath::new(client_id.clone()), (*self).into())?;
        ctx.store_consensus_state(
            ClientConsensusStatePath::new(
                client_id.clone(),
                self.latest_height().revision_number(),
                self.latest_height().revision_height(),
            ),
            mock_consensus_state.into(),
        )?;
        ctx.store_update_meta(
            client_id.clone(),
            self.latest_height(),
            ctx.host_timestamp()?,
            ctx.host_height()?,
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
            ClientConsensusStatePath::new(
                client_id.clone(),
                new_client_state.latest_height().revision_number(),
                new_client_state.latest_height().revision_height(),
            ),
            new_consensus_state.into(),
        )?;
        ctx.store_client_state(
            ClientStatePath::new(client_id.clone()),
            new_client_state.into(),
        )?;
        ctx.store_update_meta(
            client_id.clone(),
            header_height,
            ctx.host_timestamp()?,
            ctx.host_height()?,
        )?;

        Ok(vec![header_height])
    }

    fn update_state_on_misbehaviour(
        &self,
        ctx: &mut E,
        client_id: &ClientId,
        _client_message: Any,
    ) -> Result<(), ClientError> {
        let frozen_client_state = self.frozen();

        ctx.store_client_state(
            ClientStatePath::new(client_id.clone()),
            frozen_client_state.into(),
        )?;

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
            ClientConsensusStatePath::new(
                client_id.clone(),
                latest_height.revision_number(),
                latest_height.revision_height(),
            ),
            new_consensus_state.into(),
        )?;
        ctx.store_client_state(
            ClientStatePath::new(client_id.clone()),
            new_client_state.into(),
        )?;

        let host_timestamp = ctx.host_timestamp()?;
        let host_height = ctx.host_height()?;

        ctx.store_update_meta(
            client_id.clone(),
            latest_height,
            host_timestamp,
            host_height,
        )?;

        Ok(latest_height)
    }

    fn update_on_recovery(
        &self,
        ctx: &mut E,
        subject_client_id: &ClientId,
        substitute_client_state: Any,
        substitute_consensus_state: Any,
    ) -> Result<(), ClientError> {
        let substitute_client_state = MockClientState::try_from(substitute_client_state)?;

        let latest_height = substitute_client_state.latest_height();

        let new_mock_client_state = MockClientState {
            frozen: false,
            ..substitute_client_state
        };

        let host_timestamp = ctx.host_timestamp()?;
        let host_height = ctx.host_height()?;

        let mock_consensus_state = MockConsensusState::try_from(substitute_consensus_state)?;

        ctx.store_consensus_state(
            ClientConsensusStatePath::new(
                subject_client_id.clone(),
                new_mock_client_state.latest_height().revision_number(),
                new_mock_client_state.latest_height().revision_height(),
            ),
            mock_consensus_state.into(),
        )?;

        ctx.store_client_state(
            ClientStatePath::new(subject_client_id.clone()),
            new_mock_client_state.into(),
        )?;

        ctx.store_update_meta(
            subject_client_id.clone(),
            latest_height,
            host_timestamp,
            host_height,
        )?;

        Ok(())
    }
}

impl From<MockConsensusState> for MockClientState {
    fn from(cs: MockConsensusState) -> Self {
        Self::new(cs.header)
    }
}

#[cfg(test)]
mod test {
    #[cfg(feature = "serde")]
    #[test]
    fn test_any_client_state_to_json() {
        use ibc::primitives::proto::Any;

        use super::{MockClientState, MockHeader};

        let client_state = MockClientState::new(MockHeader::default());
        let expected = r#"{"typeUrl":"/ibc.mock.ClientState","value":"CgQKAhABEIDIr6Al"}"#;
        let json = serde_json::to_string(&Any::from(client_state)).unwrap();
        assert_eq!(json, expected);

        let proto_any = serde_json::from_str::<Any>(expected).unwrap();
        assert_eq!(proto_any, Any::from(client_state));
    }
}
