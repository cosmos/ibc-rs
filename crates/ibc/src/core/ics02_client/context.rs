//! ICS2 (client) context. The two traits `ClientReader` and `ClientKeeper` define the interface
//! that any host chain must implement to be able to process any `ClientMsg`. See
//! "ADR 003: IBC protocol implementation" for more details.

use alloc::boxed::Box;

use ibc_proto::google::protobuf::Any;

use crate::core::ics02_client::client_state::ClientState;
use crate::core::ics02_client::client_type::ClientType;
use crate::core::ics02_client::consensus_state::ConsensusState;
use crate::core::ics02_client::error::ClientError;
use crate::core::ics02_client::handler::ClientResult::{self, Create, Update, Upgrade};
use crate::core::ics24_host::identifier::ClientId;
use crate::core::ics24_host::path::ClientConsensusStatePath;
use crate::timestamp::Timestamp;
use crate::Height;

/// Defines the read-only part of ICS2 (client functions) context.
pub trait ClientReader {
    /// Returns the ClientType for the given identifier `client_id`.
    fn client_type(&self, client_id: &ClientId) -> Result<ClientType, ClientError>;

    /// Returns the ClientState for the given identifier `client_id`.
    fn client_state(&self, client_id: &ClientId) -> Result<Box<dyn ClientState>, ClientError>;

    /// Tries to decode the given `client_state` into a concrete light client state.
    fn decode_client_state(&self, client_state: Any) -> Result<Box<dyn ClientState>, ClientError>;

    /// Retrieve the consensus state for the given client ID at the specified
    /// height.
    ///
    /// Returns an error if no such state exists.
    fn consensus_state(
        &self,
        client_cons_state_path: &ClientConsensusStatePath,
    ) -> Result<Box<dyn ConsensusState>, ClientError>;

    /// Search for the lowest consensus state higher than `height`.
    fn next_consensus_state(
        &self,
        next_client_cons_state_path: &ClientConsensusStatePath,
    ) -> Result<Option<Box<dyn ConsensusState>>, ClientError>;

    /// Search for the highest consensus state lower than `height`.
    fn prev_consensus_state(
        &self,
        prev_client_cons_state_path: &ClientConsensusStatePath,
    ) -> Result<Option<Box<dyn ConsensusState>>, ClientError>;

    /// Returns the current height of the local chain.
    fn host_height(&self) -> Result<Height, ClientError>;

    /// Returns the current timestamp of the local chain.
    fn host_timestamp(&self) -> Result<Timestamp, ClientError> {
        let pending_consensus_state = self
            .pending_host_consensus_state()
            .expect("host must have pending consensus state");
        Ok(pending_consensus_state.timestamp())
    }

    /// Returns the `ConsensusState` of the host (local) chain at a specific height.
    fn host_consensus_state(&self, height: &Height)
        -> Result<Box<dyn ConsensusState>, ClientError>;

    /// Returns the pending `ConsensusState` of the host (local) chain.
    fn pending_host_consensus_state(&self) -> Result<Box<dyn ConsensusState>, ClientError>;

    /// Returns a natural number, counting how many clients have been created thus far.
    /// The value of this counter should increase only via method `ClientKeeper::increase_client_counter`.
    fn client_counter(&self) -> Result<u64, ClientError>;
}

/// Defines the write-only part of ICS2 (client functions) context.
pub trait ClientKeeper {
    fn store_client_result(&mut self, handler_res: ClientResult) -> Result<(), ClientError> {
        match handler_res {
            Create(res) => {
                self.store_client_type(res.client_id.clone(), res.client_type)?;
                self.store_client_state(res.client_id.clone(), res.client_state.clone())?;
                self.store_consensus_state(
                    res.client_id.clone(),
                    res.client_state.latest_height(),
                    res.consensus_state,
                )?;
                self.increase_client_counter();
                self.store_update_time(
                    res.client_id.clone(),
                    res.client_state.latest_height(),
                    res.processed_time,
                )?;
                self.store_update_height(
                    res.client_id,
                    res.client_state.latest_height(),
                    res.processed_height,
                )?;
                Ok(())
            }
            Update(res) => {
                self.store_client_state(res.client_id.clone(), res.client_state.clone())?;
                self.store_consensus_state(
                    res.client_id.clone(),
                    res.client_state.latest_height(),
                    res.consensus_state,
                )?;
                self.store_update_time(
                    res.client_id.clone(),
                    res.client_state.latest_height(),
                    res.processed_time,
                )?;
                self.store_update_height(
                    res.client_id,
                    res.client_state.latest_height(),
                    res.processed_height,
                )?;
                Ok(())
            }
            Upgrade(res) => {
                self.store_client_state(res.client_id.clone(), res.client_state.clone())?;
                self.store_consensus_state(
                    res.client_id.clone(),
                    res.client_state.latest_height(),
                    res.consensus_state,
                )?;
                Ok(())
            }
            ClientResult::Misbehaviour(res) => {
                self.store_client_state(res.client_id, res.client_state)
            }
        }
    }

    /// Called upon successful client creation
    fn store_client_type(
        &mut self,
        client_id: ClientId,
        client_type: ClientType,
    ) -> Result<(), ClientError>;

    /// Called upon successful client creation and update
    fn store_client_state(
        &mut self,
        client_id: ClientId,
        client_state: Box<dyn ClientState>,
    ) -> Result<(), ClientError>;

    /// Called upon successful client creation and update
    fn store_consensus_state(
        &mut self,
        client_id: ClientId,
        height: Height,
        consensus_state: Box<dyn ConsensusState>,
    ) -> Result<(), ClientError>;

    /// Called upon client creation.
    /// Increases the counter which keeps track of how many clients have been created.
    /// Should never fail.
    fn increase_client_counter(&mut self);

    /// Called upon successful client update.
    /// Implementations are expected to use this to record the specified time as the time at which
    /// this update (or header) was processed.
    fn store_update_time(
        &mut self,
        client_id: ClientId,
        height: Height,
        timestamp: Timestamp,
    ) -> Result<(), ClientError>;

    /// Called upon successful client update.
    /// Implementations are expected to use this to record the specified height as the height at
    /// at which this update (or header) was processed.
    fn store_update_height(
        &mut self,
        client_id: ClientId,
        height: Height,
        host_height: Height,
    ) -> Result<(), ClientError>;
}
