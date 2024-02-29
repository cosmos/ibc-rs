use core::fmt::Debug;

use basecoin_store::context::{ProvableStore, Store};
use basecoin_store::types::Height as StoreHeight;
use ibc::clients::tendermint::context::{
    CommonContext as TmCommonContext, ValidationContext as TmValidationContext,
};
use ibc::core::client::context::{ClientExecutionContext, ClientValidationContext};
use ibc::core::client::types::error::ClientError;
use ibc::core::client::types::Height;
use ibc::core::handler::types::error::ContextError;
use ibc::core::host::types::identifiers::{ChannelId, ClientId, PortId};
use ibc::core::host::types::path::{
    ClientConsensusStatePath, ClientStatePath, ClientUpdateHeightPath, ClientUpdateTimePath, Path,
};
use ibc::core::host::ValidationContext;
use ibc::core::primitives::Timestamp;
use ibc::primitives::prelude::*;

use crate::hosts::TestHost;
use crate::testapp::ibc::clients::mock::client_state::MockClientContext;
use crate::testapp::ibc::clients::{AnyClientState, AnyConsensusState};
use crate::testapp::ibc::core::types::MockGenericContext;

pub type PortChannelIdMap<V> = BTreeMap<PortId, BTreeMap<ChannelId, V>>;

/// A mock of an IBC client record as it is stored in a mock context.
/// For testing ICS02 handlers mostly, cf. `MockClientContext`.
#[derive(Clone, Debug)]
pub struct MockClientRecord {
    /// The client state (representing only the latest height at the moment).
    pub client_state: Option<AnyClientState>,

    /// Mapping of heights to consensus states for this client.
    pub consensus_states: BTreeMap<Height, AnyConsensusState>,
}

impl<S, H> MockClientContext for MockGenericContext<S, H>
where
    S: ProvableStore + Debug,
    H: TestHost,
{
    type ConversionError = &'static str;
    type AnyConsensusState = AnyConsensusState;

    fn host_timestamp(&self) -> Result<Timestamp, ContextError> {
        ValidationContext::host_timestamp(self)
    }

    fn host_height(&self) -> Result<Height, ContextError> {
        ValidationContext::host_height(self)
    }

    fn consensus_state(
        &self,
        client_cons_state_path: &ClientConsensusStatePath,
    ) -> Result<Self::AnyConsensusState, ContextError> {
        ValidationContext::consensus_state(self, client_cons_state_path)
    }
}
impl<S, H> ClientValidationContext for MockGenericContext<S, H>
where
    S: ProvableStore + Debug,
    H: TestHost,
{
    /// Returns the time and height when the client state for the given
    /// [`ClientId`] was updated with a header for the given [`Height`]
    fn update_meta(
        &self,
        client_id: &ClientId,
        height: &Height,
    ) -> Result<(Timestamp, Height), ContextError> {
        let client_update_time_path = ClientUpdateTimePath::new(
            client_id.clone(),
            height.revision_number(),
            height.revision_height(),
        );
        let processed_timestamp = self
            .ibc_store
            .client_processed_times
            .get(StoreHeight::Pending, &client_update_time_path)
            .ok_or(ClientError::UpdateMetaDataNotFound {
                client_id: client_id.clone(),
                height: *height,
            })?;
        let client_update_height_path = ClientUpdateHeightPath::new(
            client_id.clone(),
            height.revision_number(),
            height.revision_height(),
        );
        let processed_height = self
            .ibc_store
            .client_processed_heights
            .get(StoreHeight::Pending, &client_update_height_path)
            .ok_or(ClientError::UpdateMetaDataNotFound {
                client_id: client_id.clone(),
                height: *height,
            })?;

        Ok((processed_timestamp, processed_height))
    }
}

impl<S, H> ClientExecutionContext for MockGenericContext<S, H>
where
    S: ProvableStore + Debug,
    H: TestHost,
{
    type V = Self;

    type AnyClientState = AnyClientState;

    type AnyConsensusState = AnyConsensusState;

    /// Called upon successful client creation and update
    fn store_client_state(
        &mut self,
        client_state_path: ClientStatePath,
        client_state: Self::AnyClientState,
    ) -> Result<(), ContextError> {
        self.ibc_store
            .client_state_store
            .set(client_state_path.clone(), client_state)
            .map_err(|_| ClientError::Other {
                description: "Client state store error".to_string(),
            })?;

        Ok(())
    }

    /// Called upon successful client creation and update
    fn store_consensus_state(
        &mut self,
        consensus_state_path: ClientConsensusStatePath,
        consensus_state: Self::AnyConsensusState,
    ) -> Result<(), ContextError> {
        self.ibc_store
            .consensus_state_store
            .set(consensus_state_path, consensus_state)
            .map_err(|_| ClientError::Other {
                description: "Consensus state store error".to_string(),
            })?;
        Ok(())
    }

    /// Called upon successful client update. Implementations are expected to
    /// use this to record the time and height at which this update (or header)
    /// was processed.
    fn store_update_meta(
        &mut self,
        client_id: ClientId,
        height: Height,
        host_timestamp: Timestamp,
        host_height: Height,
    ) -> Result<(), ContextError> {
        let client_update_time_path = ClientUpdateTimePath::new(
            client_id.clone(),
            height.revision_number(),
            height.revision_height(),
        );
        self.ibc_store
            .client_processed_times
            .set(client_update_time_path, host_timestamp)
            .map_err(|_| ClientError::Other {
                description: "store update error".into(),
            })?;
        let client_update_height_path = ClientUpdateHeightPath::new(
            client_id.clone(),
            height.revision_number(),
            height.revision_height(),
        );
        self.ibc_store
            .client_processed_heights
            .set(client_update_height_path, host_height)
            .map_err(|_| ClientError::Other {
                description: "store update error".into(),
            })?;
        Ok(())
    }

    /// Delete the update metadata associated with the client at the specified
    /// height.
    fn delete_update_meta(
        &mut self,
        client_id: ClientId,
        height: Height,
    ) -> Result<(), ContextError> {
        let client_update_time_path = ClientUpdateTimePath::new(
            client_id.clone(),
            height.revision_number(),
            height.revision_height(),
        );
        self.ibc_store
            .client_processed_times
            .delete(client_update_time_path);
        let client_update_height_path = ClientUpdateHeightPath::new(
            client_id.clone(),
            height.revision_number(),
            height.revision_height(),
        );
        self.ibc_store
            .client_processed_heights
            .delete(client_update_height_path);
        Ok(())
    }

    fn delete_consensus_state(
        &mut self,
        consensus_state_path: ClientConsensusStatePath,
    ) -> Result<(), ContextError> {
        self.ibc_store
            .consensus_state_store
            .delete(consensus_state_path);
        Ok(())
    }
}

impl<S, H> TmCommonContext for MockGenericContext<S, H>
where
    S: ProvableStore + Debug,
    H: TestHost,
{
    type ConversionError = &'static str;
    type AnyConsensusState = AnyConsensusState;

    fn host_timestamp(&self) -> Result<Timestamp, ContextError> {
        ValidationContext::host_timestamp(self)
    }

    fn host_height(&self) -> Result<Height, ContextError> {
        ValidationContext::host_height(self)
    }

    fn consensus_state(
        &self,
        client_cons_state_path: &ClientConsensusStatePath,
    ) -> Result<Self::AnyConsensusState, ContextError> {
        ValidationContext::consensus_state(self, client_cons_state_path)
    }

    fn consensus_state_heights(&self, client_id: &ClientId) -> Result<Vec<Height>, ContextError> {
        let path = format!("clients/{}/consensusStates", client_id)
            .try_into()
            .map_err(|_| ClientError::Other {
                description: "Invalid consensus state path".into(),
            })?;

        self.ibc_store
            .consensus_state_store
            .get_keys(&path)
            .into_iter()
            .flat_map(|path| {
                if let Ok(Path::ClientConsensusState(consensus_path)) = path.try_into() {
                    Some(consensus_path)
                } else {
                    None
                }
            })
            .map(|consensus_path| {
                let height = Height::new(
                    consensus_path.revision_number,
                    consensus_path.revision_height,
                )?;
                Ok(height)
            })
            .collect()
    }
}

impl<S, H> TmValidationContext for MockGenericContext<S, H>
where
    S: ProvableStore + Debug,
    H: TestHost,
{
    fn next_consensus_state(
        &self,
        client_id: &ClientId,
        height: &Height,
    ) -> Result<Option<Self::AnyConsensusState>, ContextError> {
        let path = format!("clients/{client_id}/consensusStates")
            .try_into()
            .unwrap(); // safety - path must be valid since ClientId and height are valid Identifiers

        let keys = self.ibc_store.store.get_keys(&path);
        let found_path = keys.into_iter().find_map(|path| {
            if let Ok(Path::ClientConsensusState(path)) = path.try_into() {
                if height
                    < &Height::new(path.revision_number, path.revision_height).expect("no error")
                {
                    return Some(path);
                }
            }
            None
        });

        if let Some(path) = found_path {
            let consensus_state = self
                .ibc_store
                .consensus_state_store
                .get(StoreHeight::Pending, &path)
                .ok_or(ClientError::ConsensusStateNotFound {
                    client_id: client_id.clone(),
                    height: *height,
                })?;

            Ok(Some(consensus_state))
        } else {
            Ok(None)
        }
    }

    fn prev_consensus_state(
        &self,
        client_id: &ClientId,
        height: &Height,
    ) -> Result<Option<Self::AnyConsensusState>, ContextError> {
        let path = format!("clients/{client_id}/consensusStates")
            .try_into()
            .unwrap(); // safety - path must be valid since ClientId and height are valid Identifiers

        let keys = self.ibc_store.store.get_keys(&path);
        let found_path = keys.into_iter().rev().find_map(|path| {
            if let Ok(Path::ClientConsensusState(path)) = path.try_into() {
                if height
                    > &Height::new(path.revision_number, path.revision_height).expect("no error")
                {
                    return Some(path);
                }
            }
            None
        });

        if let Some(path) = found_path {
            let consensus_state = self
                .ibc_store
                .consensus_state_store
                .get(StoreHeight::Pending, &path)
                .ok_or(ClientError::ConsensusStateNotFound {
                    client_id: client_id.clone(),
                    height: *height,
                })?;

            Ok(Some(consensus_state))
        } else {
            Ok(None)
        }
    }
}
