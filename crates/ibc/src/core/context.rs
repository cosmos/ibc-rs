use crate::prelude::*;

use super::{
    ics02_client::error::ClientError,
    ics03_connection::error::ConnectionError,
    ics04_channel::error::{ChannelError, PacketError},
};

use displaydoc::Display;

#[derive(Debug, Display)]
pub enum ContextError {
    /// ICS02 Client error
    ClientError(ClientError),
    /// ICS03 Connection error
    ConnectionError(ConnectionError),
    /// Ics04 Channel error
    ChannelError(ChannelError),
    /// ICS04 Packet error
    PacketError(PacketError),
}

impl From<ClientError> for ContextError {
    fn from(err: ClientError) -> ContextError {
        Self::ClientError(err)
    }
}

impl From<ConnectionError> for ContextError {
    fn from(err: ConnectionError) -> ContextError {
        Self::ConnectionError(err)
    }
}

impl From<ChannelError> for ContextError {
    fn from(err: ChannelError) -> ContextError {
        Self::ChannelError(err)
    }
}

impl From<PacketError> for ContextError {
    fn from(err: PacketError) -> ContextError {
        Self::PacketError(err)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for ContextError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self {
            Self::ClientError(e) => Some(e),
            Self::ConnectionError(e) => Some(e),
            Self::ChannelError(e) => Some(e),
            Self::PacketError(e) => Some(e),
        }
    }
}

#[cfg(feature = "val_exec_ctx")]
pub use val_exec_ctx::*;

#[cfg(feature = "val_exec_ctx")]
mod val_exec_ctx {
    use super::*;
    use core::time::Duration;

    use ibc_proto::google::protobuf::Any;

    use crate::core::ics02_client::client_state::ClientState;
    use crate::core::ics02_client::client_type::ClientType;
    use crate::core::ics02_client::consensus_state::ConsensusState;
    use crate::core::ics03_connection::connection::ConnectionEnd;
    use crate::core::ics03_connection::error::ConnectionError;
    use crate::core::ics03_connection::version::{
        get_compatible_versions, pick_version, Version as ConnectionVersion,
    };
    use crate::core::ics04_channel::channel::{ChannelEnd, Counterparty, State};
    use crate::core::ics04_channel::commitment::{AcknowledgementCommitment, PacketCommitment};
    use crate::core::ics04_channel::context::calculate_block_delay;
    use crate::core::ics04_channel::events::OpenTry;
    use crate::core::ics04_channel::handler::chan_open_try;
    use crate::core::ics04_channel::msgs::acknowledgement::Acknowledgement;
    use crate::core::ics04_channel::msgs::chan_open_try::MsgChannelOpenTry;
    use crate::core::ics04_channel::msgs::ChannelMsg;
    use crate::core::ics04_channel::packet::{Receipt, Sequence};
    use crate::core::ics04_channel::timeout::TimeoutHeight;
    use crate::core::ics05_port::error::PortError::UnknownPort;
    use crate::core::ics23_commitment::commitment::CommitmentPrefix;
    use crate::core::ics24_host::identifier::{ChannelId, ConnectionId, PortId};
    use crate::core::ics24_host::path::{
        ClientConnectionsPath, ClientConsensusStatePath, ClientStatePath, ClientTypePath,
        CommitmentsPath, ConnectionsPath, ReceiptsPath,
    };
    use crate::core::ics26_routing::context::{Module, ModuleId};
    use crate::core::{
        ics02_client::{
            handler::{create_client, misbehaviour, update_client, upgrade_client},
            msgs::ClientMsg,
        },
        ics03_connection::{
            handler::{conn_open_ack, conn_open_confirm, conn_open_init, conn_open_try},
            msgs::ConnectionMsg,
        },
        ics24_host::identifier::ClientId,
        ics26_routing::{error::RouterError, msgs::MsgEnvelope},
    };
    use crate::events::IbcEvent;
    use crate::timestamp::Timestamp;
    use crate::Height;

    use super::ContextError;

    pub trait Router {
        /// Returns a reference to a `Module` registered against the specified `ModuleId`
        fn get_route(&self, module_id: &ModuleId) -> Option<&dyn Module>;

        /// Returns a mutable reference to a `Module` registered against the specified `ModuleId`
        fn get_route_mut(&mut self, module_id: &ModuleId) -> Option<&mut dyn Module>;

        /// Returns true if the `Router` has a `Module` registered against the specified `ModuleId`
        fn has_route(&self, module_id: &ModuleId) -> bool;

        /// Return the module_id associated with a given port_id
        fn lookup_module_by_port(&self, port_id: &PortId) -> Option<ModuleId>;

        fn lookup_module(&self, msg: &ChannelMsg) -> Result<ModuleId, ChannelError> {
            let port_id = match msg {
                ChannelMsg::OpenInit(msg) => &msg.port_id_on_a,
                ChannelMsg::OpenTry(msg) => &msg.port_id_on_b,
                ChannelMsg::OpenAck(msg) => &msg.port_id_on_a,
                ChannelMsg::OpenConfirm(msg) => &msg.port_id_on_b,
                ChannelMsg::CloseInit(msg) => &msg.port_id_on_a,
                ChannelMsg::CloseConfirm(msg) => &msg.port_id_on_b,
            };
            let module_id = self
                .lookup_module_by_port(port_id)
                .ok_or(ChannelError::Port(UnknownPort {
                    port_id: port_id.clone(),
                }))?;
            Ok(module_id)
        }
    }

    pub trait ValidationContext: Router {
        /// Validation entrypoint.
        fn validate(&self, message: MsgEnvelope) -> Result<(), RouterError>
        where
            Self: Sized,
        {
            match message {
                MsgEnvelope::Client(message) => match message {
                    ClientMsg::CreateClient(message) => create_client::validate(self, message),
                    ClientMsg::UpdateClient(message) => update_client::validate(self, message),
                    ClientMsg::Misbehaviour(message) => misbehaviour::validate(self, message),
                    ClientMsg::UpgradeClient(message) => upgrade_client::validate(self, message),
                }
                .map_err(RouterError::ContextError),
                MsgEnvelope::Connection(message) => match message {
                    ConnectionMsg::OpenInit(message) => conn_open_init::validate(self, message),
                    ConnectionMsg::OpenTry(message) => conn_open_try::validate(self, message),
                    ConnectionMsg::OpenAck(message) => conn_open_ack::validate(self, message),
                    ConnectionMsg::OpenConfirm(ref message) => {
                        conn_open_confirm::validate(self, message)
                    }
                }
                .map_err(RouterError::ContextError),
                MsgEnvelope::Channel(message) => {
                    let module_id = self.lookup_module(&message).map_err(ContextError::from)?;
                    if !self.has_route(&module_id) {
                        return Err(ChannelError::RouteNotFound)
                            .map_err(ContextError::ChannelError)
                            .map_err(RouterError::ContextError);
                    }

                    match message {
                        ChannelMsg::OpenInit(_) => todo!(),
                        ChannelMsg::OpenTry(message) => {
                            chan_open_try_validate(self, module_id, message)
                        }
                        ChannelMsg::OpenAck(_) => todo!(),
                        ChannelMsg::OpenConfirm(_) => todo!(),
                        ChannelMsg::CloseInit(_) => todo!(),
                        ChannelMsg::CloseConfirm(_) => todo!(),
                    }
                    .map_err(RouterError::ContextError)
                }
                MsgEnvelope::Packet(_message) => todo!(),
            }
        }

        /// Returns the ClientState for the given identifier `client_id`.
        fn client_state(&self, client_id: &ClientId) -> Result<Box<dyn ClientState>, ContextError>;

        /// Tries to decode the given `client_state` into a concrete light client state.
        fn decode_client_state(
            &self,
            client_state: Any,
        ) -> Result<Box<dyn ClientState>, ContextError>;

        /// Retrieve the consensus state for the given client ID at the specified
        /// height.
        ///
        /// Returns an error if no such state exists.
        fn consensus_state(
            &self,
            client_id: &ClientId,
            height: &Height,
        ) -> Result<Box<dyn ConsensusState>, ContextError>;

        /// Search for the lowest consensus state higher than `height`.
        fn next_consensus_state(
            &self,
            client_id: &ClientId,
            height: &Height,
        ) -> Result<Option<Box<dyn ConsensusState>>, ContextError>;

        /// Search for the highest consensus state lower than `height`.
        fn prev_consensus_state(
            &self,
            client_id: &ClientId,
            height: &Height,
        ) -> Result<Option<Box<dyn ConsensusState>>, ContextError>;

        /// Returns the current height of the local chain.
        fn host_height(&self) -> Result<Height, ContextError>;

        /// Returns the current timestamp of the local chain.
        fn host_timestamp(&self) -> Result<Timestamp, ContextError> {
            let pending_consensus_state = self
                .pending_host_consensus_state()
                .expect("host must have pending consensus state");
            Ok(pending_consensus_state.timestamp())
        }

        /// Returns the pending `ConsensusState` of the host (local) chain.
        fn pending_host_consensus_state(&self) -> Result<Box<dyn ConsensusState>, ContextError>;

        /// Returns the `ConsensusState` of the host (local) chain at a specific height.
        fn host_consensus_state(
            &self,
            height: &Height,
        ) -> Result<Box<dyn ConsensusState>, ContextError>;

        /// Returns a natural number, counting how many clients have been created thus far.
        /// The value of this counter should increase only via method `ClientKeeper::increase_client_counter`.
        fn generate_client_identifier(&self) -> Result<u64, ContextError>;

        /// Returns the ConnectionEnd for the given identifier `conn_id`.
        fn connection_end(&self, conn_id: &ConnectionId) -> Result<ConnectionEnd, ContextError>;

        /// Validates the `ClientState` of the client on the counterparty chain.
        fn validate_self_client(
            &self,
            counterparty_client_state: Any,
        ) -> Result<(), ConnectionError>;

        /// Returns the prefix that the local chain uses in the KV store.
        fn commitment_prefix(&self) -> CommitmentPrefix;

        /// Returns a counter on how many connections have been created thus far.
        fn generate_connection_identifier(&self) -> Result<u64, ContextError>;

        /// Function required by ICS 03. Returns the list of all possible versions that the connection
        /// handshake protocol supports.
        fn get_compatible_versions(&self) -> Vec<ConnectionVersion> {
            get_compatible_versions()
        }

        /// Function required by ICS 03. Returns one version out of the supplied list of versions, which the
        /// connection handshake protocol prefers.
        fn pick_version(
            &self,
            supported_versions: &[ConnectionVersion],
            counterparty_candidate_versions: &[ConnectionVersion],
        ) -> Result<ConnectionVersion, ContextError> {
            pick_version(supported_versions, counterparty_candidate_versions)
                .map_err(ContextError::ConnectionError)
        }

        /// Returns the ChannelEnd for the given `port_id` and `chan_id`.
        fn channel_end(
            &self,
            port_channel_id: &(PortId, ChannelId),
        ) -> Result<ChannelEnd, ContextError>;

        fn connection_channels(
            &self,
            cid: &ConnectionId,
        ) -> Result<Vec<(PortId, ChannelId)>, ContextError>;

        fn get_next_sequence_send(
            &self,
            port_channel_id: &(PortId, ChannelId),
        ) -> Result<Sequence, ContextError>;

        fn get_next_sequence_recv(
            &self,
            port_channel_id: &(PortId, ChannelId),
        ) -> Result<Sequence, ContextError>;

        fn get_next_sequence_ack(
            &self,
            port_channel_id: &(PortId, ChannelId),
        ) -> Result<Sequence, ContextError>;

        fn get_packet_commitment(
            &self,
            key: &(PortId, ChannelId, Sequence),
        ) -> Result<PacketCommitment, ContextError>;

        fn get_packet_receipt(
            &self,
            key: &(PortId, ChannelId, Sequence),
        ) -> Result<Receipt, ContextError>;

        fn get_packet_acknowledgement(
            &self,
            key: &(PortId, ChannelId, Sequence),
        ) -> Result<AcknowledgementCommitment, ContextError>;

        /// Compute the commitment for a packet.
        /// Note that the absence of `timeout_height` is treated as
        /// `{revision_number: 0, revision_height: 0}` to be consistent with ibc-go,
        /// where this value is used to mean "no timeout height":
        /// <https://github.com/cosmos/ibc-go/blob/04791984b3d6c83f704c4f058e6ca0038d155d91/modules/core/04-channel/keeper/packet.go#L206>
        fn packet_commitment(
            &self,
            packet_data: &[u8],
            timeout_height: &TimeoutHeight,
            timeout_timestamp: &Timestamp,
        ) -> PacketCommitment {
            let mut hash_input = timeout_timestamp.nanoseconds().to_be_bytes().to_vec();

            let revision_number = timeout_height.commitment_revision_number().to_be_bytes();
            hash_input.append(&mut revision_number.to_vec());

            let revision_height = timeout_height.commitment_revision_height().to_be_bytes();
            hash_input.append(&mut revision_height.to_vec());

            let packet_data_hash = self.hash(packet_data);
            hash_input.append(&mut packet_data_hash.to_vec());

            self.hash(&hash_input).into()
        }

        fn ack_commitment(&self, ack: &Acknowledgement) -> AcknowledgementCommitment {
            self.hash(ack.as_ref()).into()
        }

        /// A hashing function for packet commitments
        fn hash(&self, value: &[u8]) -> Vec<u8>;

        /// Returns the time when the client state for the given [`ClientId`] was updated with a header for the given [`Height`]
        fn client_update_time(
            &self,
            client_id: &ClientId,
            height: &Height,
        ) -> Result<Timestamp, ContextError>;

        /// Returns the height when the client state for the given [`ClientId`] was updated with a header for the given [`Height`]
        fn client_update_height(
            &self,
            client_id: &ClientId,
            height: &Height,
        ) -> Result<Height, ContextError>;

        /// Returns a counter on the number of channel ids have been created thus far.
        /// The value of this counter should increase only via method
        /// `ChannelKeeper::increase_channel_counter`.
        fn generate_channel_identifier(&self) -> Result<u64, ContextError>;

        /// Returns the maximum expected time per block
        fn max_expected_time_per_block(&self) -> Duration;

        /// Calculates the block delay period using the connection's delay period and the maximum
        /// expected time per block.
        fn block_delay(&self, delay_period_time: &Duration) -> u64 {
            calculate_block_delay(delay_period_time, &self.max_expected_time_per_block())
        }
    }

    pub trait ExecutionContext: ValidationContext {
        /// Execution entrypoint
        fn execute(&mut self, message: MsgEnvelope) -> Result<(), RouterError>
        where
            Self: Sized,
        {
            match message {
                MsgEnvelope::Client(message) => match message {
                    ClientMsg::CreateClient(message) => create_client::execute(self, message),
                    ClientMsg::UpdateClient(message) => update_client::execute(self, message),
                    ClientMsg::Misbehaviour(message) => misbehaviour::execute(self, message),
                    ClientMsg::UpgradeClient(message) => upgrade_client::execute(self, message),
                }
                .map_err(RouterError::ContextError),
                MsgEnvelope::Connection(message) => match message {
                    ConnectionMsg::OpenInit(message) => conn_open_init::execute(self, message),
                    ConnectionMsg::OpenTry(message) => conn_open_try::execute(self, message),
                    ConnectionMsg::OpenAck(message) => conn_open_ack::execute(self, message),
                    ConnectionMsg::OpenConfirm(ref message) => {
                        conn_open_confirm::execute(self, message)
                    }
                }
                .map_err(RouterError::ContextError),
                MsgEnvelope::Channel(message) => {
                    let module_id = self.lookup_module(&message).map_err(ContextError::from)?;
                    if !self.has_route(&module_id) {
                        return Err(ChannelError::RouteNotFound)
                            .map_err(ContextError::ChannelError)
                            .map_err(RouterError::ContextError);
                    }

                    match message {
                        ChannelMsg::OpenInit(_) => todo!(),
                        ChannelMsg::OpenTry(message) => {
                            chan_open_try_execute(self, module_id, message)
                        }
                        ChannelMsg::OpenAck(_) => todo!(),
                        ChannelMsg::OpenConfirm(_) => todo!(),
                        ChannelMsg::CloseInit(_) => todo!(),
                        ChannelMsg::CloseConfirm(_) => todo!(),
                    }
                    .map_err(RouterError::ContextError)
                }
                MsgEnvelope::Packet(_message) => todo!(),
            }
        }

        /// Called upon successful client creation
        fn store_client_type(
            &mut self,
            client_type_path: ClientTypePath,
            client_type: ClientType,
        ) -> Result<(), ContextError>;

        /// Called upon successful client creation and update
        fn store_client_state(
            &mut self,
            client_state_path: ClientStatePath,
            client_state: Box<dyn ClientState>,
        ) -> Result<(), ContextError>;

        /// Called upon successful client creation and update
        fn store_consensus_state(
            &mut self,
            consensus_state_path: ClientConsensusStatePath,
            consensus_state: Box<dyn ConsensusState>,
        ) -> Result<(), ContextError>;

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
        ) -> Result<(), ContextError>;

        /// Called upon successful client update.
        /// Implementations are expected to use this to record the specified height as the height at
        /// at which this update (or header) was processed.
        fn store_update_height(
            &mut self,
            client_id: ClientId,
            height: Height,
            host_height: Height,
        ) -> Result<(), ContextError>;

        /// Stores the given connection_end at path
        fn store_connection(
            &mut self,
            connections_path: ConnectionsPath,
            connection_end: ConnectionEnd,
        ) -> Result<(), ContextError>;

        /// Stores the given connection_id at a path associated with the client_id.
        fn store_connection_to_client(
            &mut self,
            client_connections_path: ClientConnectionsPath,
            conn_id: ConnectionId,
        ) -> Result<(), ContextError>;

        /// Called upon connection identifier creation (Init or Try process).
        /// Increases the counter which keeps track of how many connections have been created.
        /// Should never fail.
        fn increase_connection_counter(&mut self);

        fn store_packet_commitment(
            &mut self,
            commitments_path: CommitmentsPath,
            commitment: PacketCommitment,
        ) -> Result<(), ContextError>;

        fn delete_packet_commitment(&mut self, key: CommitmentsPath) -> Result<(), ContextError>;

        fn store_packet_receipt(
            &mut self,
            path: ReceiptsPath,
            receipt: Receipt,
        ) -> Result<(), ContextError>;

        fn store_packet_acknowledgement(
            &mut self,
            key: (PortId, ChannelId, Sequence),
            ack_commitment: AcknowledgementCommitment,
        ) -> Result<(), ContextError>;

        fn delete_packet_acknowledgement(
            &mut self,
            key: (PortId, ChannelId, Sequence),
        ) -> Result<(), ContextError>;

        fn store_connection_channels(
            &mut self,
            conn_id: ConnectionId,
            port_channel_id: (PortId, ChannelId),
        ) -> Result<(), ContextError>;

        /// Stores the given channel_end at a path associated with the port_id and channel_id.
        fn store_channel(
            &mut self,
            port_channel_id: (PortId, ChannelId),
            channel_end: ChannelEnd,
        ) -> Result<(), ContextError>;

        fn store_next_sequence_send(
            &mut self,
            port_channel_id: (PortId, ChannelId),
            seq: Sequence,
        ) -> Result<(), ContextError>;

        fn store_next_sequence_recv(
            &mut self,
            port_channel_id: (PortId, ChannelId),
            seq: Sequence,
        ) -> Result<(), ContextError>;

        fn store_next_sequence_ack(
            &mut self,
            port_channel_id: (PortId, ChannelId),
            seq: Sequence,
        ) -> Result<(), ContextError>;

        /// Called upon channel identifier creation (Init or Try message processing).
        /// Increases the counter which keeps track of how many channels have been created.
        /// Should never fail.
        fn increase_channel_counter(&mut self);

        /// Ibc events
        fn emit_ibc_event(&mut self, event: IbcEvent);

        /// Logging facility
        fn log_message(&mut self, message: String);
    }

    fn chan_open_try_validate<ValCtx>(
        ctx_b: &ValCtx,
        module_id: ModuleId,
        msg: MsgChannelOpenTry,
    ) -> Result<(), ContextError>
    where
        ValCtx: ValidationContext,
    {
        chan_open_try::validate(ctx_b, &msg)?;
        let chan_id_on_b = ChannelId::new(ctx_b.generate_channel_identifier()?);

        let module = ctx_b
            .get_route(&module_id)
            .ok_or(ChannelError::RouteNotFound)?;
        let _ = module.on_chan_open_try_validate(
            msg.ordering,
            &msg.connection_hops_on_b,
            &msg.port_id_on_b,
            &chan_id_on_b,
            &Counterparty::new(msg.port_id_on_a.clone(), Some(msg.chan_id_on_a.clone())),
            &msg.version_supported_on_a,
        )?;

        Ok(())
    }

    fn chan_open_try_execute<ExecCtx>(
        ctx_b: &mut ExecCtx,
        module_id: ModuleId,
        msg: MsgChannelOpenTry,
    ) -> Result<(), ContextError>
    where
        ExecCtx: ExecutionContext,
    {
        let chan_id_on_b = ChannelId::new(ctx_b.generate_channel_identifier()?);
        ctx_b.log_message(format!(
            "success: channel open try with channel identifier: {chan_id_on_b}"
        ));

        let module = ctx_b
            .get_route_mut(&module_id)
            .ok_or(ChannelError::RouteNotFound)?;

        let (extras, version) = module.on_chan_open_try_execute(
            msg.ordering,
            &msg.connection_hops_on_b,
            &msg.port_id_on_b,
            &chan_id_on_b,
            &Counterparty::new(msg.port_id_on_a.clone(), Some(msg.chan_id_on_a.clone())),
            &msg.version_supported_on_a,
        )?;

        let conn_id_on_b = msg.connection_hops_on_b[0].clone();
        let port_channel_id_on_b = (msg.port_id_on_b.clone(), chan_id_on_b.clone());

        // emit events and logs
        {
            let core_event = IbcEvent::OpenTryChannel(OpenTry::new(
                msg.port_id_on_b.clone(),
                chan_id_on_b.clone(),
                msg.port_id_on_a.clone(),
                msg.chan_id_on_a.clone(),
                conn_id_on_b.clone(),
                version.clone(),
            ));
            ctx_b.emit_ibc_event(core_event);

            for module_event in extras.events {
                ctx_b.emit_ibc_event(IbcEvent::AppModule(module_event));
            }

            for log_message in extras.log {
                ctx_b.log_message(log_message);
            }
        }

        {
            let channel_end = ChannelEnd::new(
                State::TryOpen,
                msg.ordering,
                Counterparty::new(msg.port_id_on_a.clone(), Some(msg.chan_id_on_a.clone())),
                msg.connection_hops_on_b.clone(),
                version,
            );

            ctx_b.store_channel(port_channel_id_on_b.clone(), channel_end)?;

            ctx_b.increase_channel_counter();

            // Associate also the channel end to its connection.
            ctx_b.store_connection_channels(conn_id_on_b, port_channel_id_on_b.clone())?;

            // Initialize send, recv, and ack sequence numbers.
            ctx_b.store_next_sequence_send(port_channel_id_on_b.clone(), 1.into())?;
            ctx_b.store_next_sequence_recv(port_channel_id_on_b.clone(), 1.into())?;
            ctx_b.store_next_sequence_ack(port_channel_id_on_b, 1.into())?;
        }

        Ok(())
    }
}
