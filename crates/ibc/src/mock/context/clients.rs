//! Client context implementations for `MockContext`

use crate::core::ics02_client::ClientExecutionContext;
use crate::core::ics24_host::path::ClientStatePath;
use crate::prelude::*;
use crate::{
    core::{
        ics02_client::error::ClientError,
        ics24_host::{identifier::ClientId, path::ClientConsensusStatePath},
        timestamp::Timestamp,
        ContextError, ValidationContext,
    },
    Height,
};

use crate::clients::ics07_tendermint::{
    CommonContext as TmCommonContext, ValidationContext as TmValidationContext,
};

use super::{AnyClientState, AnyConsensusState, MockClientRecord, MockContext};

impl TmCommonContext for MockContext {
    type AnyConsensusState = AnyConsensusState;

    fn consensus_state(
        &self,
        client_cons_state_path: &ClientConsensusStatePath,
    ) -> Result<Self::AnyConsensusState, ContextError> {
        ValidationContext::consensus_state(self, client_cons_state_path)
    }
}

impl TmValidationContext for MockContext {
    fn host_timestamp(&self) -> Result<Timestamp, ContextError> {
        ValidationContext::host_timestamp(self)
    }

    fn next_consensus_state(
        &self,
        client_id: &ClientId,
        height: &Height,
    ) -> Result<Option<Self::AnyConsensusState>, ContextError> {
        let ibc_store = self.ibc_store.lock();
        let client_record =
            ibc_store
                .clients
                .get(client_id)
                .ok_or_else(|| ClientError::ClientStateNotFound {
                    client_id: client_id.clone(),
                })?;

        // Get the consensus state heights and sort them in ascending order.
        let mut heights: Vec<Height> = client_record.consensus_states.keys().cloned().collect();
        heights.sort();

        // Search for next state.
        for h in heights {
            if h > *height {
                // unwrap should never happen, as the consensus state for h must exist
                return Ok(Some(
                    client_record.consensus_states.get(&h).unwrap().clone(),
                ));
            }
        }
        Ok(None)
    }

    fn prev_consensus_state(
        &self,
        client_id: &ClientId,
        height: &Height,
    ) -> Result<Option<Self::AnyConsensusState>, ContextError> {
        let ibc_store = self.ibc_store.lock();
        let client_record =
            ibc_store
                .clients
                .get(client_id)
                .ok_or_else(|| ClientError::ClientStateNotFound {
                    client_id: client_id.clone(),
                })?;

        // Get the consensus state heights and sort them in descending order.
        let mut heights: Vec<Height> = client_record.consensus_states.keys().cloned().collect();
        heights.sort_by(|a, b| b.cmp(a));

        // Search for previous state.
        for h in heights {
            if h < *height {
                // unwrap should never happen, as the consensus state for h must exist
                return Ok(Some(
                    client_record.consensus_states.get(&h).unwrap().clone(),
                ));
            }
        }
        Ok(None)
    }
}

impl ClientExecutionContext for MockContext {
    type ClientValidationContext = Self;
    type AnyClientState = AnyClientState;
    type AnyConsensusState = AnyConsensusState;

    fn store_client_state(
        &mut self,
        client_state_path: ClientStatePath,
        client_state: Self::AnyClientState,
    ) -> Result<(), ContextError> {
        let mut ibc_store = self.ibc_store.lock();

        let client_id = client_state_path.0;
        let client_record = ibc_store
            .clients
            .entry(client_id)
            .or_insert(MockClientRecord {
                consensus_states: Default::default(),
                client_state: Default::default(),
            });

        client_record.client_state = Some(client_state);

        Ok(())
    }

    fn store_consensus_state(
        &mut self,
        consensus_state_path: ClientConsensusStatePath,
        consensus_state: Self::AnyConsensusState,
    ) -> Result<(), ContextError> {
        let mut ibc_store = self.ibc_store.lock();

        let client_record = ibc_store
            .clients
            .entry(consensus_state_path.client_id)
            .or_insert(MockClientRecord {
                consensus_states: Default::default(),
                client_state: Default::default(),
            });

        let height = Height::new(consensus_state_path.epoch, consensus_state_path.height).unwrap();
        client_record
            .consensus_states
            .insert(height, consensus_state);

        Ok(())
    }
}
