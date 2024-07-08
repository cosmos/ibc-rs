//! Implementation of the `ClientValidationContext` and `ClientExecutionContext`
//! traits for the `Context` type.
use ibc_client_wasm_types::client_state::ClientState as WasmClientState;
use ibc_client_wasm_types::consensus_state::ConsensusState as WasmConsensusState;
use ibc_core::client::context::{ClientExecutionContext, ClientValidationContext};
use ibc_core::client::types::error::ClientError;
use ibc_core::client::types::Height;
use ibc_core::handler::types::error::ContextError;
use ibc_core::host::types::identifiers::ClientId;
use ibc_core::host::types::path::{ClientConsensusStatePath, ClientStatePath};
use ibc_core::primitives::proto::{Any, Protobuf};
use ibc_core::primitives::Timestamp;

use super::{Context, StorageMut};
use crate::api::ClientType;
use crate::context::CONSENSUS_STATE_HEIGHT_MAP;
use crate::utils::AnyCodec;

impl<'a, C: ClientType<'a>> ClientValidationContext for Context<'a, C>
where
    <C::ClientState as TryFrom<Any>>::Error: Into<ClientError>,
    <C::ConsensusState as TryFrom<Any>>::Error: Into<ClientError>,
{
    type ClientStateRef = C::ClientState;
    type ConsensusStateRef = C::ConsensusState;

    fn client_state(&self, _client_id: &ClientId) -> Result<Self::ClientStateRef, ContextError> {
        let client_state_value = self.retrieve(ClientStatePath::leaf())?;

        let any_wasm: WasmClientState = Protobuf::<Any>::decode(client_state_value.as_slice())
            .map_err(|e| ClientError::Other {
                description: e.to_string(),
            })?;

        let sov_client_state = C::ClientState::decode_any_vec(any_wasm.data)?;

        Ok(sov_client_state)
    }

    fn consensus_state(
        &self,
        client_cons_state_path: &ClientConsensusStatePath,
    ) -> Result<Self::ConsensusStateRef, ContextError> {
        let consensus_state_value = self.retrieve(client_cons_state_path.leaf())?;

        let any_wasm: WasmConsensusState =
            C::ConsensusState::decode_any_vec(consensus_state_value)?;

        let consensus_state = C::ConsensusState::decode_any_vec(any_wasm.data)?;

        Ok(consensus_state)
    }

    fn client_update_meta(
        &self,
        _client_id: &ClientId,
        height: &Height,
    ) -> Result<(Timestamp, Height), ContextError> {
        let time_key = self.client_update_time_key(height);

        let time_vec = self.retrieve(time_key)?;

        let time = u64::from_be_bytes(time_vec.try_into().map_err(|_| ClientError::Other {
            description: "time key cannot be converted to u64".to_string(),
        })?);

        let timestamp = Timestamp::from_nanoseconds(time).map_err(|e| ClientError::Other {
            description: e.to_string(),
        })?;

        let height_key = self.client_update_height_key(height);

        let revision_height_vec = self.retrieve(height_key)?;

        let revision_height =
            u64::from_be_bytes(
                revision_height_vec
                    .try_into()
                    .map_err(|_| ClientError::Other {
                        description: "revision height key cannot be converted to u64".to_string(),
                    })?,
            );

        let height = Height::new(0, revision_height)?;

        Ok((timestamp, height))
    }
}

impl<'a, C: ClientType<'a>> ClientExecutionContext for Context<'a, C>
where
    <C::ClientState as TryFrom<Any>>::Error: Into<ClientError>,
    <C::ConsensusState as TryFrom<Any>>::Error: Into<ClientError>,
{
    type ClientStateMut = C::ClientState;

    fn store_client_state(
        &mut self,
        _client_state_path: ClientStatePath,
        client_state: Self::ClientStateMut,
    ) -> Result<(), ContextError> {
        let prefixed_key = self.prefixed_key(ClientStatePath::leaf());

        let encoded_client_state = self.encode_client_state(client_state)?;

        self.insert(prefixed_key, encoded_client_state);

        Ok(())
    }

    fn store_consensus_state(
        &mut self,
        consensus_state_path: ClientConsensusStatePath,
        consensus_state: Self::ConsensusStateRef,
    ) -> Result<(), ContextError> {
        let prefixed_key = self.prefixed_key(consensus_state_path.leaf());

        let encoded_consensus_state = C::ConsensusState::encode_to_any_vec(consensus_state);

        let wasm_consensus_state = WasmConsensusState {
            data: encoded_consensus_state,
        };

        let encoded_wasm_consensus_state =
            C::ConsensusState::encode_to_any_vec(wasm_consensus_state);

        self.insert(prefixed_key, encoded_wasm_consensus_state);

        Ok(())
    }

    fn delete_consensus_state(
        &mut self,
        consensus_state_path: ClientConsensusStatePath,
    ) -> Result<(), ContextError> {
        let prefixed_key = self.prefixed_key(consensus_state_path.leaf());

        self.remove(prefixed_key);

        Ok(())
    }

    fn store_update_meta(
        &mut self,
        _client_id: ClientId,
        height: Height,
        host_timestamp: Timestamp,
        host_height: Height,
    ) -> Result<(), ContextError> {
        let time_key = self.client_update_time_key(&height);

        let prefixed_time_key = self.prefixed_key(time_key);

        let time_vec = host_timestamp.nanoseconds().to_be_bytes();

        self.insert(prefixed_time_key, time_vec);

        let height_key = self.client_update_height_key(&height);

        let prefixed_height_key = self.prefixed_key(height_key);

        let revision_height_vec = host_height.revision_height().to_be_bytes();

        self.insert(prefixed_height_key, revision_height_vec);

        CONSENSUS_STATE_HEIGHT_MAP
            .save(
                self.storage_mut(),
                (height.revision_number(), height.revision_height()),
                &Default::default(),
            )
            .map_err(|e| ClientError::Other {
                description: e.to_string(),
            })?;

        Ok(())
    }

    fn delete_update_meta(
        &mut self,
        _client_id: ClientId,
        height: Height,
    ) -> Result<(), ContextError> {
        let time_key = self.client_update_time_key(&height);

        let prefixed_time_key = self.prefixed_key(time_key);

        self.remove(prefixed_time_key);

        let height_key = self.client_update_height_key(&height);

        let prefixed_height_key = self.prefixed_key(height_key);

        self.remove(prefixed_height_key);

        CONSENSUS_STATE_HEIGHT_MAP.remove(
            self.storage_mut(),
            (height.revision_number(), height.revision_height()),
        );

        Ok(())
    }
}
