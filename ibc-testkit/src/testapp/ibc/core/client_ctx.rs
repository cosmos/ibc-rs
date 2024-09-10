use core::fmt::Debug;

use basecoin_store::context::{ProvableStore, Store};
use basecoin_store::types::Height as StoreHeight;
use ibc::core::client::context::{
    ClientExecutionContext, ClientValidationContext, ExtClientValidationContext,
};
use ibc::core::client::types::Height;
use ibc::core::host::types::error::HostError;
use ibc::core::host::types::identifiers::{ChannelId, ClientId, PortId};
use ibc::core::host::types::path::{
    ClientConsensusStatePath, ClientStatePath, ClientUpdateHeightPath, ClientUpdateTimePath, Path,
};
use ibc::core::host::ValidationContext;
use ibc::core::primitives::Timestamp;
use ibc::primitives::prelude::*;

use super::types::MockIbcStore;
use crate::testapp::ibc::clients::mock::client_state::MockClientContext;
use crate::testapp::ibc::clients::{AnyClientState, AnyConsensusState};

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

impl<S> MockClientContext for MockIbcStore<S>
where
    S: ProvableStore + Debug,
{
    fn host_timestamp(&self) -> Result<Timestamp, HostError> {
        ValidationContext::host_timestamp(self)
    }

    fn host_height(&self) -> Result<Height, HostError> {
        ValidationContext::host_height(self)
    }
}

impl<S> ExtClientValidationContext for MockIbcStore<S>
where
    S: ProvableStore + Debug,
{
    fn host_timestamp(&self) -> Result<Timestamp, HostError> {
        ValidationContext::host_timestamp(self)
    }

    fn host_height(&self) -> Result<Height, HostError> {
        ValidationContext::host_height(self)
    }

    /// Returns the list of heights at which the consensus state of the given client was updated.
    fn consensus_state_heights(&self, client_id: &ClientId) -> Result<Vec<Height>, HostError> {
        let path = format!("clients/{}/consensusStates", client_id).into();

        self.consensus_state_store
            .get_keys(&path)
            .into_iter()
            .filter_map(|path| {
                if let Ok(Path::ClientConsensusState(consensus_path)) = path.try_into() {
                    Some(consensus_path)
                } else {
                    None
                }
            })
            .map(|consensus_path| {
                Height::new(
                    consensus_path.revision_number,
                    consensus_path.revision_height,
                )
                .map_err(|e| HostError::InvalidData {
                    description: e.to_string(),
                })
            })
            .collect::<Result<Vec<_>, _>>()
    }

    fn next_consensus_state(
        &self,
        client_id: &ClientId,
        height: &Height,
    ) -> Result<Option<Self::ConsensusStateRef>, HostError> {
        let path = format!("clients/{client_id}/consensusStates").into();

        let keys = self.store.get_keys(&path);
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

        let consensus_state = found_path
            .map(|path| {
                self.consensus_state_store
                    .get(StoreHeight::Pending, &path)
                    .ok_or_else(|| HostError::FailedToRetrieveFromStore {
                        description: format!(
                            "missing consensus state for client {0} at height {1}",
                            client_id.clone(),
                            *height
                        ),
                    })
            })
            .transpose()
            .map_err(|e| HostError::MissingData {
                description: e.to_string(),
            })?;

        Ok(consensus_state)
    }

    fn prev_consensus_state(
        &self,
        client_id: &ClientId,
        height: &Height,
    ) -> Result<Option<Self::ConsensusStateRef>, HostError> {
        let path = format!("clients/{client_id}/consensusStates").into();

        let keys = self.store.get_keys(&path);
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

        let consensus_state = found_path
            .map(|path| {
                self.consensus_state_store
                    .get(StoreHeight::Pending, &path)
                    .ok_or_else(|| HostError::FailedToRetrieveFromStore {
                        description: format!(
                            "missing consensus state for client {0} at height {1}",
                            client_id.clone(),
                            *height
                        ),
                    })
            })
            .transpose()
            .map_err(|e| HostError::MissingData {
                description: e.to_string(),
            })?;

        Ok(consensus_state)
    }
}

impl<S> ClientValidationContext for MockIbcStore<S>
where
    S: ProvableStore + Debug,
{
    type ClientStateRef = AnyClientState;
    type ConsensusStateRef = AnyConsensusState;

    fn client_state(&self, client_id: &ClientId) -> Result<Self::ClientStateRef, HostError> {
        self.client_state_store
            .get(StoreHeight::Pending, &ClientStatePath(client_id.clone()))
            .ok_or(HostError::FailedToRetrieveFromStore {
                description: format!("missing client state for client {0}", client_id.clone()),
            })
    }

    fn consensus_state(
        &self,
        client_cons_state_path: &ClientConsensusStatePath,
    ) -> Result<AnyConsensusState, HostError> {
        let height = Height::new(
            client_cons_state_path.revision_number,
            client_cons_state_path.revision_height,
        )
        .map_err(|e| HostError::InvalidData {
            description: e.to_string(),
        })?;
        let consensus_state = self
            .consensus_state_store
            .get(StoreHeight::Pending, client_cons_state_path)
            .ok_or(HostError::FailedToRetrieveFromStore {
                description: format!(
                    "missing consensus state for client {0} at height {1}",
                    client_cons_state_path.client_id.clone(),
                    height
                ),
            })?;

        Ok(consensus_state)
    }

    /// Returns the time and height when the client state for the given
    /// [`ClientId`] was updated with a header for the given [`Height`]
    fn client_update_meta(
        &self,
        client_id: &ClientId,
        height: &Height,
    ) -> Result<(Timestamp, Height), HostError> {
        let client_update_time_path = ClientUpdateTimePath::new(
            client_id.clone(),
            height.revision_number(),
            height.revision_height(),
        );
        let processed_timestamp = self
            .client_processed_times
            .get(StoreHeight::Pending, &client_update_time_path)
            .ok_or(HostError::FailedToRetrieveFromStore {
                description: format!(
                    "missing client update metadata for client {0} at height {1}",
                    client_id.clone(),
                    *height,
                ),
            })?;
        let client_update_height_path = ClientUpdateHeightPath::new(
            client_id.clone(),
            height.revision_number(),
            height.revision_height(),
        );
        let processed_height = self
            .client_processed_heights
            .get(StoreHeight::Pending, &client_update_height_path)
            .ok_or(HostError::FailedToRetrieveFromStore {
                description: format!(
                    "missing client update metadata for client {0} at height {1}",
                    client_id.clone(),
                    *height,
                ),
            })?;

        Ok((processed_timestamp, processed_height))
    }
}

impl<S> ClientExecutionContext for MockIbcStore<S>
where
    S: ProvableStore + Debug,
{
    type ClientStateMut = AnyClientState;

    /// Called upon successful client creation and update
    fn store_client_state(
        &mut self,
        client_state_path: ClientStatePath,
        client_state: Self::ClientStateRef,
    ) -> Result<(), HostError> {
        self.client_state_store
            .set(client_state_path, client_state)
            .map_err(|e| HostError::FailedToStoreData {
                description: format!("{e:?}"),
            })?;

        Ok(())
    }

    /// Called upon successful client creation and update
    fn store_consensus_state(
        &mut self,
        consensus_state_path: ClientConsensusStatePath,
        consensus_state: Self::ConsensusStateRef,
    ) -> Result<(), HostError> {
        self.consensus_state_store
            .set(consensus_state_path, consensus_state)
            .map_err(|e| HostError::FailedToStoreData {
                description: format!("{e:?}"),
            })?;
        Ok(())
    }

    fn delete_consensus_state(
        &mut self,
        consensus_state_path: ClientConsensusStatePath,
    ) -> Result<(), HostError> {
        self.consensus_state_store.delete(consensus_state_path);
        Ok(())
    }

    /// Delete the update metadata associated with the client at the specified
    /// height.
    fn delete_update_meta(&mut self, client_id: ClientId, height: Height) -> Result<(), HostError> {
        let client_update_time_path = ClientUpdateTimePath::new(
            client_id.clone(),
            height.revision_number(),
            height.revision_height(),
        );
        self.client_processed_times.delete(client_update_time_path);
        let client_update_height_path = ClientUpdateHeightPath::new(
            client_id,
            height.revision_number(),
            height.revision_height(),
        );
        self.client_processed_heights
            .delete(client_update_height_path);
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
    ) -> Result<(), HostError> {
        let client_update_time_path = ClientUpdateTimePath::new(
            client_id.clone(),
            height.revision_number(),
            height.revision_height(),
        );
        self.client_processed_times
            .set(client_update_time_path, host_timestamp)
            .map_err(|e| HostError::FailedToStoreData {
                description: format!("{e:?}"),
            })?;
        let client_update_height_path = ClientUpdateHeightPath::new(
            client_id,
            height.revision_number(),
            height.revision_height(),
        );
        self.client_processed_heights
            .set(client_update_height_path, host_height)
            .map_err(|e| HostError::FailedToStoreData {
                description: format!("{e:?}"),
            })?;
        Ok(())
    }
}
