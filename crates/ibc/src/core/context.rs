use crate::prelude::*;

use super::{
    ics02_client::error::ClientError,
    ics03_connection::error::ConnectionError,
    ics04_channel::error::{ChannelError, PacketError},
};
use core::time::Duration;

use ibc_proto::google::protobuf::Any;

use crate::core::ics02_client::client_state::ClientState;
use crate::core::ics02_client::client_type::ClientType;
use crate::core::ics02_client::consensus_state::ConsensusState;
use crate::core::ics03_connection::connection::ConnectionEnd;
use crate::core::ics03_connection::version::{
    get_compatible_versions, pick_version, Version as ConnectionVersion,
};
use crate::core::ics04_channel::channel::{ChannelEnd, Counterparty, Order, State};
use crate::core::ics04_channel::commitment::{AcknowledgementCommitment, PacketCommitment};
use crate::core::ics04_channel::context::calculate_block_delay;
use crate::core::ics04_channel::events::{
    AcknowledgePacket, ChannelClosed, CloseConfirm, CloseInit, OpenAck, OpenConfirm, OpenInit,
    OpenTry, ReceivePacket, TimeoutPacket, WriteAcknowledgement,
};
use crate::core::ics04_channel::handler::{
    acknowledgement, chan_close_confirm, chan_close_init, chan_open_ack, chan_open_confirm,
    chan_open_init, chan_open_try, recv_packet, timeout, timeout_on_close,
};
use crate::core::ics04_channel::msgs::acknowledgement::{Acknowledgement, MsgAcknowledgement};
use crate::core::ics04_channel::msgs::chan_close_confirm::MsgChannelCloseConfirm;
use crate::core::ics04_channel::msgs::chan_close_init::MsgChannelCloseInit;
use crate::core::ics04_channel::msgs::chan_open_ack::MsgChannelOpenAck;
use crate::core::ics04_channel::msgs::chan_open_confirm::MsgChannelOpenConfirm;
use crate::core::ics04_channel::msgs::chan_open_init::MsgChannelOpenInit;
use crate::core::ics04_channel::msgs::chan_open_try::MsgChannelOpenTry;
use crate::core::ics04_channel::msgs::recv_packet::MsgRecvPacket;
use crate::core::ics04_channel::msgs::timeout::MsgTimeout;
use crate::core::ics04_channel::msgs::timeout_on_close::MsgTimeoutOnClose;
use crate::core::ics04_channel::msgs::{ChannelMsg, PacketMsg};
use crate::core::ics04_channel::packet::{Receipt, Sequence};
use crate::core::ics04_channel::timeout::TimeoutHeight;
use crate::core::ics05_port::error::PortError::UnknownPort;
use crate::core::ics23_commitment::commitment::CommitmentPrefix;
use crate::core::ics24_host::identifier::{ChannelId, ConnectionId, PortId};
use crate::core::ics24_host::path::{
    AckPath, ChannelEndPath, ClientConnectionPath, ClientConsensusStatePath, ClientStatePath,
    ClientTypePath, CommitmentPath, ConnectionPath, ReceiptPath, SeqAckPath, SeqRecvPath,
    SeqSendPath,
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

pub trait Router {
    /// Returns a reference to a `Module` registered against the specified `ModuleId`
    fn get_route(&self, module_id: &ModuleId) -> Option<&dyn Module>;

    /// Returns a mutable reference to a `Module` registered against the specified `ModuleId`
    fn get_route_mut(&mut self, module_id: &ModuleId) -> Option<&mut dyn Module>;

    /// Returns true if the `Router` has a `Module` registered against the specified `ModuleId`
    fn has_route(&self, module_id: &ModuleId) -> bool;

    /// Return the module_id associated with a given port_id
    fn lookup_module_by_port(&self, port_id: &PortId) -> Option<ModuleId>;

    fn lookup_module_channel(&self, msg: &ChannelMsg) -> Result<ModuleId, ChannelError> {
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

    fn lookup_module_packet(&self, msg: &PacketMsg) -> Result<ModuleId, ChannelError> {
        let port_id = match msg {
            PacketMsg::Recv(msg) => &msg.packet.port_on_b,
            PacketMsg::Ack(msg) => &msg.packet.port_on_a,
            PacketMsg::Timeout(msg) => &msg.packet.port_on_a,
            PacketMsg::TimeoutOnClose(msg) => &msg.packet.port_on_a,
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
    fn validate(&self, msg: MsgEnvelope) -> Result<(), RouterError>
    where
        Self: Sized,
    {
        match msg {
            MsgEnvelope::Client(msg) => match msg {
                ClientMsg::CreateClient(msg) => create_client::validate(self, msg),
                ClientMsg::UpdateClient(msg) => update_client::validate(self, msg),
                ClientMsg::Misbehaviour(msg) => misbehaviour::validate(self, msg),
                ClientMsg::UpgradeClient(msg) => upgrade_client::validate(self, msg),
            }
            .map_err(RouterError::ContextError),
            MsgEnvelope::Connection(msg) => match msg {
                ConnectionMsg::OpenInit(msg) => conn_open_init::validate(self, msg),
                ConnectionMsg::OpenTry(msg) => conn_open_try::validate(self, msg),
                ConnectionMsg::OpenAck(msg) => conn_open_ack::validate(self, msg),
                ConnectionMsg::OpenConfirm(ref msg) => conn_open_confirm::validate(self, msg),
            }
            .map_err(RouterError::ContextError),
            MsgEnvelope::Channel(msg) => {
                let module_id = self
                    .lookup_module_channel(&msg)
                    .map_err(ContextError::from)?;
                if !self.has_route(&module_id) {
                    return Err(ChannelError::RouteNotFound)
                        .map_err(ContextError::ChannelError)
                        .map_err(RouterError::ContextError);
                }

                match msg {
                    ChannelMsg::OpenInit(msg) => chan_open_init_validate(self, module_id, msg),
                    ChannelMsg::OpenTry(msg) => chan_open_try_validate(self, module_id, msg),
                    ChannelMsg::OpenAck(msg) => chan_open_ack_validate(self, module_id, msg),
                    ChannelMsg::OpenConfirm(msg) => {
                        chan_open_confirm_validate(self, module_id, msg)
                    }
                    ChannelMsg::CloseInit(msg) => chan_close_init_validate(self, module_id, msg),
                    ChannelMsg::CloseConfirm(msg) => {
                        chan_close_confirm_validate(self, module_id, msg)
                    }
                }
                .map_err(RouterError::ContextError)
            }
            MsgEnvelope::Packet(msg) => {
                let module_id = self
                    .lookup_module_packet(&msg)
                    .map_err(ContextError::from)?;
                if !self.has_route(&module_id) {
                    return Err(ChannelError::RouteNotFound)
                        .map_err(ContextError::ChannelError)
                        .map_err(RouterError::ContextError);
                }

                match msg {
                    PacketMsg::Recv(msg) => recv_packet_validate(self, msg),
                    PacketMsg::Ack(msg) => acknowledgement_packet_validate(self, module_id, msg),
                    PacketMsg::Timeout(msg) => {
                        timeout_packet_validate(self, module_id, TimeoutMsgType::Timeout(msg))
                    }
                    PacketMsg::TimeoutOnClose(msg) => timeout_packet_validate(
                        self,
                        module_id,
                        TimeoutMsgType::TimeoutOnClose(msg),
                    ),
                }
                .map_err(RouterError::ContextError)
            }
        }
    }

    /// Returns the ClientState for the given identifier `client_id`.
    fn client_state(&self, client_id: &ClientId) -> Result<Box<dyn ClientState>, ContextError>;

    /// Tries to decode the given `client_state` into a concrete light client state.
    fn decode_client_state(&self, client_state: Any) -> Result<Box<dyn ClientState>, ContextError>;

    /// Retrieve the consensus state for the given client ID at the specified
    /// height.
    ///
    /// Returns an error if no such state exists.
    fn consensus_state(
        &self,
        client_cons_state_path: &ClientConsensusStatePath,
    ) -> Result<Box<dyn ConsensusState>, ContextError>;

    /// Search for the lowest consensus state higher than `height`.
    fn next_consensus_state(
        &self,
        next_client_cons_state_path: &ClientConsensusStatePath,
    ) -> Result<Option<Box<dyn ConsensusState>>, ContextError>;

    /// Search for the highest consensus state lower than `height`.
    fn prev_consensus_state(
        &self,
        prev_client_cons_state_path: &ClientConsensusStatePath,
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
    fn client_counter(&self) -> Result<u64, ContextError>;

    /// Returns the ConnectionEnd for the given identifier `conn_id`.
    fn connection_end(&self, conn_id: &ConnectionId) -> Result<ConnectionEnd, ContextError>;

    /// Validates the `ClientState` of the client on the counterparty chain.
    fn validate_self_client(&self, counterparty_client_state: Any) -> Result<(), ConnectionError>;

    /// Returns the prefix that the local chain uses in the KV store.
    fn commitment_prefix(&self) -> CommitmentPrefix;

    /// Returns a counter on how many connections have been created thus far.
    fn connection_counter(&self) -> Result<u64, ContextError>;

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
    fn channel_end(&self, channel_end_path: &ChannelEndPath) -> Result<ChannelEnd, ContextError>;

    fn connection_channels(
        &self,
        cid: &ConnectionId,
    ) -> Result<Vec<(PortId, ChannelId)>, ContextError>;

    fn get_next_sequence_send(&self, seq_send_path: &SeqSendPath)
        -> Result<Sequence, ContextError>;

    fn get_next_sequence_recv(&self, seq_recv_path: &SeqRecvPath)
        -> Result<Sequence, ContextError>;

    fn get_next_sequence_ack(&self, seq_ack_path: &SeqAckPath) -> Result<Sequence, ContextError>;

    fn get_packet_commitment(
        &self,
        commitment_path: &CommitmentPath,
    ) -> Result<PacketCommitment, ContextError>;

    fn get_packet_receipt(&self, receipt_path: &ReceiptPath) -> Result<Receipt, ContextError>;

    fn get_packet_acknowledgement(
        &self,
        ack_path: &AckPath,
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
    fn channel_counter(&self) -> Result<u64, ContextError>;

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
    fn execute(&mut self, msg: MsgEnvelope) -> Result<(), RouterError>
    where
        Self: Sized,
    {
        match msg {
            MsgEnvelope::Client(msg) => match msg {
                ClientMsg::CreateClient(msg) => create_client::execute(self, msg),
                ClientMsg::UpdateClient(msg) => update_client::execute(self, msg),
                ClientMsg::Misbehaviour(msg) => misbehaviour::execute(self, msg),
                ClientMsg::UpgradeClient(msg) => upgrade_client::execute(self, msg),
            }
            .map_err(RouterError::ContextError),
            MsgEnvelope::Connection(msg) => match msg {
                ConnectionMsg::OpenInit(msg) => conn_open_init::execute(self, msg),
                ConnectionMsg::OpenTry(msg) => conn_open_try::execute(self, msg),
                ConnectionMsg::OpenAck(msg) => conn_open_ack::execute(self, msg),
                ConnectionMsg::OpenConfirm(ref msg) => conn_open_confirm::execute(self, msg),
            }
            .map_err(RouterError::ContextError),
            MsgEnvelope::Channel(msg) => {
                let module_id = self
                    .lookup_module_channel(&msg)
                    .map_err(ContextError::from)?;
                if !self.has_route(&module_id) {
                    return Err(ChannelError::RouteNotFound)
                        .map_err(ContextError::ChannelError)
                        .map_err(RouterError::ContextError);
                }

                match msg {
                    ChannelMsg::OpenInit(msg) => chan_open_init_execute(self, module_id, msg),
                    ChannelMsg::OpenTry(msg) => chan_open_try_execute(self, module_id, msg),
                    ChannelMsg::OpenAck(msg) => chan_open_ack_execute(self, module_id, msg),
                    ChannelMsg::OpenConfirm(msg) => chan_open_confirm_execute(self, module_id, msg),
                    ChannelMsg::CloseInit(msg) => chan_close_init_execute(self, module_id, msg),
                    ChannelMsg::CloseConfirm(msg) => {
                        chan_close_confirm_execute(self, module_id, msg)
                    }
                }
                .map_err(RouterError::ContextError)
            }
            MsgEnvelope::Packet(msg) => {
                let module_id = self
                    .lookup_module_packet(&msg)
                    .map_err(ContextError::from)?;
                if !self.has_route(&module_id) {
                    return Err(ChannelError::RouteNotFound)
                        .map_err(ContextError::ChannelError)
                        .map_err(RouterError::ContextError);
                }

                match msg {
                    PacketMsg::Recv(msg) => recv_packet_execute(self, module_id, msg),
                    PacketMsg::Ack(msg) => acknowledgement_packet_execute(self, module_id, msg),
                    PacketMsg::Timeout(msg) => {
                        timeout_packet_execute(self, module_id, TimeoutMsgType::Timeout(msg))
                    }
                    PacketMsg::TimeoutOnClose(msg) => {
                        timeout_packet_execute(self, module_id, TimeoutMsgType::TimeoutOnClose(msg))
                    }
                }
                .map_err(RouterError::ContextError)
            }
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
        connection_path: ConnectionPath,
        connection_end: ConnectionEnd,
    ) -> Result<(), ContextError>;

    /// Stores the given connection_id at a path associated with the client_id.
    fn store_connection_to_client(
        &mut self,
        client_connection_path: ClientConnectionPath,
        conn_id: ConnectionId,
    ) -> Result<(), ContextError>;

    /// Called upon connection identifier creation (Init or Try process).
    /// Increases the counter which keeps track of how many connections have been created.
    /// Should never fail.
    fn increase_connection_counter(&mut self);

    fn store_packet_commitment(
        &mut self,
        commitment_path: CommitmentPath,
        commitment: PacketCommitment,
    ) -> Result<(), ContextError>;

    fn delete_packet_commitment(&mut self, key: CommitmentPath) -> Result<(), ContextError>;

    fn store_packet_receipt(
        &mut self,
        receipt_path: ReceiptPath,
        receipt: Receipt,
    ) -> Result<(), ContextError>;

    fn store_packet_acknowledgement(
        &mut self,
        ack_path: AckPath,
        ack_commitment: AcknowledgementCommitment,
    ) -> Result<(), ContextError>;

    fn delete_packet_acknowledgement(&mut self, ack_path: AckPath) -> Result<(), ContextError>;

    /// Stores the given channel_end at a path associated with the port_id and channel_id.
    fn store_channel(
        &mut self,
        channel_end_path: ChannelEndPath,
        channel_end: ChannelEnd,
    ) -> Result<(), ContextError>;

    fn store_next_sequence_send(
        &mut self,
        seq_send_path: SeqSendPath,
        seq: Sequence,
    ) -> Result<(), ContextError>;

    fn store_next_sequence_recv(
        &mut self,
        seq_recv_path: SeqRecvPath,
        seq: Sequence,
    ) -> Result<(), ContextError>;

    fn store_next_sequence_ack(
        &mut self,
        seq_ack_path: SeqAckPath,
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

fn chan_open_init_validate<ValCtx>(
    ctx_a: &ValCtx,
    module_id: ModuleId,
    msg: MsgChannelOpenInit,
) -> Result<(), ContextError>
where
    ValCtx: ValidationContext,
{
    chan_open_init::validate(ctx_a, &msg)?;
    let chan_id_on_a = ChannelId::new(ctx_a.channel_counter()?);

    let module = ctx_a
        .get_route(&module_id)
        .ok_or(ChannelError::RouteNotFound)?;
    module.on_chan_open_init_validate(
        msg.ordering,
        &msg.connection_hops_on_a,
        &msg.port_id_on_a,
        &chan_id_on_a,
        &Counterparty::new(msg.port_id_on_b.clone(), None),
        &msg.version_proposal,
    )?;

    Ok(())
}

fn chan_open_init_execute<ExecCtx>(
    ctx_a: &mut ExecCtx,
    module_id: ModuleId,
    msg: MsgChannelOpenInit,
) -> Result<(), ContextError>
where
    ExecCtx: ExecutionContext,
{
    let chan_id_on_a = ChannelId::new(ctx_a.channel_counter()?);
    let module = ctx_a
        .get_route_mut(&module_id)
        .ok_or(ChannelError::RouteNotFound)?;
    let (extras, version) = module.on_chan_open_init_execute(
        msg.ordering,
        &msg.connection_hops_on_a,
        &msg.port_id_on_a,
        &chan_id_on_a,
        &Counterparty::new(msg.port_id_on_b.clone(), None),
        &msg.version_proposal,
    )?;

    let conn_id_on_a = msg.connection_hops_on_a[0].clone();

    // state changes
    {
        let chan_end_on_a = ChannelEnd::new(
            State::Init,
            msg.ordering,
            Counterparty::new(msg.port_id_on_b.clone(), None),
            msg.connection_hops_on_a.clone(),
            msg.version_proposal.clone(),
        );
        let chan_end_path_on_a = ChannelEndPath::new(&msg.port_id_on_a, &chan_id_on_a);
        ctx_a.store_channel(chan_end_path_on_a, chan_end_on_a)?;

        ctx_a.increase_channel_counter();

        // Initialize send, recv, and ack sequence numbers.
        let seq_send_path = SeqSendPath::new(&msg.port_id_on_a, &chan_id_on_a);
        ctx_a.store_next_sequence_send(seq_send_path, 1.into())?;

        let seq_recv_path = SeqRecvPath::new(&msg.port_id_on_a, &chan_id_on_a);
        ctx_a.store_next_sequence_recv(seq_recv_path, 1.into())?;

        let seq_ack_path = SeqAckPath::new(&msg.port_id_on_a, &chan_id_on_a);
        ctx_a.store_next_sequence_ack(seq_ack_path, 1.into())?;
    }

    // emit events and logs
    {
        ctx_a.log_message(format!(
            "success: channel open init with channel identifier: {chan_id_on_a}"
        ));
        let core_event = IbcEvent::OpenInitChannel(OpenInit::new(
            msg.port_id_on_a.clone(),
            chan_id_on_a.clone(),
            msg.port_id_on_b,
            conn_id_on_a,
            version,
        ));
        ctx_a.emit_ibc_event(core_event);

        for module_event in extras.events {
            ctx_a.emit_ibc_event(IbcEvent::AppModule(module_event));
        }

        for log_message in extras.log {
            ctx_a.log_message(log_message);
        }
    }

    Ok(())
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
    let chan_id_on_b = ChannelId::new(ctx_b.channel_counter()?);

    let module = ctx_b
        .get_route(&module_id)
        .ok_or(ChannelError::RouteNotFound)?;
    module.on_chan_open_try_validate(
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
    let chan_id_on_b = ChannelId::new(ctx_b.channel_counter()?);
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

    // state changes
    {
        let chan_end_on_b = ChannelEnd::new(
            State::TryOpen,
            msg.ordering,
            Counterparty::new(msg.port_id_on_a.clone(), Some(msg.chan_id_on_a.clone())),
            msg.connection_hops_on_b.clone(),
            version.clone(),
        );

        let chan_end_path_on_b = ChannelEndPath::new(&msg.port_id_on_b, &chan_id_on_b);
        ctx_b.store_channel(chan_end_path_on_b, chan_end_on_b)?;
        ctx_b.increase_channel_counter();

        // Initialize send, recv, and ack sequence numbers.
        let seq_send_path = SeqSendPath::new(&msg.port_id_on_b, &chan_id_on_b);
        ctx_b.store_next_sequence_send(seq_send_path, 1.into())?;

        let seq_recv_path = SeqRecvPath::new(&msg.port_id_on_b, &chan_id_on_b);
        ctx_b.store_next_sequence_recv(seq_recv_path, 1.into())?;

        let seq_ack_path = SeqAckPath::new(&msg.port_id_on_b, &chan_id_on_b);
        ctx_b.store_next_sequence_ack(seq_ack_path, 1.into())?;
    }

    // emit events and logs
    {
        ctx_b.log_message(format!(
            "success: channel open try with channel identifier: {chan_id_on_b}"
        ));

        let core_event = IbcEvent::OpenTryChannel(OpenTry::new(
            msg.port_id_on_b.clone(),
            chan_id_on_b.clone(),
            msg.port_id_on_a.clone(),
            msg.chan_id_on_a.clone(),
            conn_id_on_b,
            version,
        ));
        ctx_b.emit_ibc_event(core_event);

        for module_event in extras.events {
            ctx_b.emit_ibc_event(IbcEvent::AppModule(module_event));
        }

        for log_message in extras.log {
            ctx_b.log_message(log_message);
        }
    }

    Ok(())
}

fn chan_open_ack_validate<ValCtx>(
    ctx_a: &ValCtx,
    module_id: ModuleId,
    msg: MsgChannelOpenAck,
) -> Result<(), ContextError>
where
    ValCtx: ValidationContext,
{
    chan_open_ack::validate(ctx_a, &msg)?;

    let module = ctx_a
        .get_route(&module_id)
        .ok_or(ChannelError::RouteNotFound)?;
    module.on_chan_open_ack_validate(&msg.port_id_on_a, &msg.chan_id_on_a, &msg.version_on_b)?;

    Ok(())
}

fn chan_open_ack_execute<ExecCtx>(
    ctx_a: &mut ExecCtx,
    module_id: ModuleId,
    msg: MsgChannelOpenAck,
) -> Result<(), ContextError>
where
    ExecCtx: ExecutionContext,
{
    let module = ctx_a
        .get_route_mut(&module_id)
        .ok_or(ChannelError::RouteNotFound)?;
    let extras =
        module.on_chan_open_ack_execute(&msg.port_id_on_a, &msg.chan_id_on_a, &msg.version_on_b)?;
    let chan_end_path_on_a = ChannelEndPath::new(&msg.port_id_on_a, &msg.chan_id_on_a);
    let chan_end_on_a = ctx_a.channel_end(&chan_end_path_on_a)?;

    // state changes
    {
        let chan_end_on_a = {
            let mut chan_end_on_a = chan_end_on_a.clone();

            chan_end_on_a.set_state(State::Open);
            chan_end_on_a.set_version(msg.version_on_b.clone());
            chan_end_on_a.set_counterparty_channel_id(msg.chan_id_on_b.clone());

            chan_end_on_a
        };
        ctx_a.store_channel(chan_end_path_on_a, chan_end_on_a)?;
    }

    // emit events and logs
    {
        ctx_a.log_message("success: channel open ack".to_string());

        let core_event = {
            let port_id_on_b = chan_end_on_a.counterparty().port_id.clone();
            let conn_id_on_a = chan_end_on_a.connection_hops[0].clone();

            IbcEvent::OpenAckChannel(OpenAck::new(
                msg.port_id_on_a.clone(),
                msg.chan_id_on_a.clone(),
                port_id_on_b,
                msg.chan_id_on_b,
                conn_id_on_a,
            ))
        };
        ctx_a.emit_ibc_event(core_event);

        for module_event in extras.events {
            ctx_a.emit_ibc_event(IbcEvent::AppModule(module_event));
        }

        for log_message in extras.log {
            ctx_a.log_message(log_message);
        }
    }

    Ok(())
}

fn chan_open_confirm_validate<ValCtx>(
    ctx_b: &ValCtx,
    module_id: ModuleId,
    msg: MsgChannelOpenConfirm,
) -> Result<(), ContextError>
where
    ValCtx: ValidationContext,
{
    chan_open_confirm::validate(ctx_b, &msg)?;

    let module = ctx_b
        .get_route(&module_id)
        .ok_or(ChannelError::RouteNotFound)?;
    module.on_chan_open_confirm_validate(&msg.port_id_on_b, &msg.chan_id_on_b)?;

    Ok(())
}

fn chan_open_confirm_execute<ExecCtx>(
    ctx_b: &mut ExecCtx,
    module_id: ModuleId,
    msg: MsgChannelOpenConfirm,
) -> Result<(), ContextError>
where
    ExecCtx: ExecutionContext,
{
    let module = ctx_b
        .get_route_mut(&module_id)
        .ok_or(ChannelError::RouteNotFound)?;

    let extras = module.on_chan_open_confirm_execute(&msg.port_id_on_b, &msg.chan_id_on_b)?;
    let chan_end_path_on_b = ChannelEndPath::new(&msg.port_id_on_b, &msg.chan_id_on_b);
    let chan_end_on_b = ctx_b.channel_end(&chan_end_path_on_b)?;

    // state changes
    {
        let chan_end_on_b = {
            let mut chan_end_on_b = chan_end_on_b.clone();
            chan_end_on_b.set_state(State::Open);

            chan_end_on_b
        };
        ctx_b.store_channel(chan_end_path_on_b, chan_end_on_b)?;
    }

    // emit events and logs
    {
        ctx_b.log_message("success: channel open confirm".to_string());

        let conn_id_on_b = chan_end_on_b.connection_hops[0].clone();
        let port_id_on_a = chan_end_on_b.counterparty().port_id.clone();
        let chan_id_on_a = chan_end_on_b
            .counterparty()
            .channel_id
            .clone()
            .ok_or(ContextError::ChannelError(ChannelError::Other {
            description:
                "internal error: ChannelEnd doesn't have a counterparty channel id in OpenConfirm"
                    .to_string(),
        }))?;

        let core_event = IbcEvent::OpenConfirmChannel(OpenConfirm::new(
            msg.port_id_on_b.clone(),
            msg.chan_id_on_b.clone(),
            port_id_on_a,
            chan_id_on_a,
            conn_id_on_b,
        ));
        ctx_b.emit_ibc_event(core_event);

        for module_event in extras.events {
            ctx_b.emit_ibc_event(IbcEvent::AppModule(module_event));
        }

        for log_message in extras.log {
            ctx_b.log_message(log_message);
        }
    }

    Ok(())
}

fn chan_close_init_validate<ValCtx>(
    ctx_a: &ValCtx,
    module_id: ModuleId,
    msg: MsgChannelCloseInit,
) -> Result<(), ContextError>
where
    ValCtx: ValidationContext,
{
    chan_close_init::validate(ctx_a, &msg)?;

    let module = ctx_a
        .get_route(&module_id)
        .ok_or(ChannelError::RouteNotFound)?;
    module.on_chan_close_init_validate(&msg.port_id_on_a, &msg.chan_id_on_a)?;

    Ok(())
}

fn chan_close_init_execute<ExecCtx>(
    ctx_a: &mut ExecCtx,
    module_id: ModuleId,
    msg: MsgChannelCloseInit,
) -> Result<(), ContextError>
where
    ExecCtx: ExecutionContext,
{
    let module = ctx_a
        .get_route_mut(&module_id)
        .ok_or(ChannelError::RouteNotFound)?;
    let extras = module.on_chan_close_init_execute(&msg.port_id_on_a, &msg.chan_id_on_a)?;
    let chan_end_path_on_a = ChannelEndPath::new(&msg.port_id_on_a, &msg.chan_id_on_a);
    let chan_end_on_a = ctx_a.channel_end(&chan_end_path_on_a)?;

    // state changes
    {
        let chan_end_on_a = {
            let mut chan_end_on_a = chan_end_on_a.clone();
            chan_end_on_a.set_state(State::Closed);
            chan_end_on_a
        };

        ctx_a.store_channel(chan_end_path_on_a, chan_end_on_a)?;
    }

    // emit events and logs
    {
        ctx_a.log_message("success: channel close init".to_string());

        let core_event = {
            let port_id_on_b = chan_end_on_a.counterparty().port_id.clone();
            let chan_id_on_b = chan_end_on_a
                .counterparty()
                .channel_id
                .clone()
                .ok_or(ContextError::ChannelError(ChannelError::Other {
                description:
                    "internal error: ChannelEnd doesn't have a counterparty channel id in CloseInit"
                        .to_string(),
            }))?;
            let conn_id_on_a = chan_end_on_a.connection_hops[0].clone();

            IbcEvent::CloseInitChannel(CloseInit::new(
                msg.port_id_on_a.clone(),
                msg.chan_id_on_a.clone(),
                port_id_on_b,
                chan_id_on_b,
                conn_id_on_a,
            ))
        };
        ctx_a.emit_ibc_event(core_event);

        for module_event in extras.events {
            ctx_a.emit_ibc_event(IbcEvent::AppModule(module_event));
        }

        for log_message in extras.log {
            ctx_a.log_message(log_message);
        }
    }

    Ok(())
}

fn chan_close_confirm_validate<ValCtx>(
    ctx_b: &ValCtx,
    module_id: ModuleId,
    msg: MsgChannelCloseConfirm,
) -> Result<(), ContextError>
where
    ValCtx: ValidationContext,
{
    chan_close_confirm::validate(ctx_b, &msg)?;

    let module = ctx_b
        .get_route(&module_id)
        .ok_or(ChannelError::RouteNotFound)?;
    module.on_chan_close_confirm_validate(&msg.port_id_on_b, &msg.chan_id_on_b)?;

    Ok(())
}

fn chan_close_confirm_execute<ExecCtx>(
    ctx_b: &mut ExecCtx,
    module_id: ModuleId,
    msg: MsgChannelCloseConfirm,
) -> Result<(), ContextError>
where
    ExecCtx: ExecutionContext,
{
    let module = ctx_b
        .get_route_mut(&module_id)
        .ok_or(ChannelError::RouteNotFound)?;
    let extras = module.on_chan_close_confirm_execute(&msg.port_id_on_b, &msg.chan_id_on_b)?;
    let chan_end_path_on_b = ChannelEndPath::new(&msg.port_id_on_b, &msg.chan_id_on_b);
    let chan_end_on_b = ctx_b.channel_end(&chan_end_path_on_b)?;

    // state changes
    {
        let chan_end_on_b = {
            let mut chan_end_on_b = chan_end_on_b.clone();
            chan_end_on_b.set_state(State::Closed);
            chan_end_on_b
        };
        ctx_b.store_channel(chan_end_path_on_b, chan_end_on_b)?;
    }

    // emit events and logs
    {
        ctx_b.log_message("success: channel close confirm".to_string());

        let core_event = {
            let port_id_on_a = chan_end_on_b.counterparty().port_id.clone();
            let chan_id_on_a = chan_end_on_b
                .counterparty()
                .channel_id
                .clone()
                .ok_or(ContextError::ChannelError(ChannelError::Other {
                description:
                    "internal error: ChannelEnd doesn't have a counterparty channel id in CloseInit"
                        .to_string(),
            }))?;
            let conn_id_on_b = chan_end_on_b.connection_hops[0].clone();

            IbcEvent::CloseConfirmChannel(CloseConfirm::new(
                msg.port_id_on_b.clone(),
                msg.chan_id_on_b.clone(),
                port_id_on_a,
                chan_id_on_a,
                conn_id_on_b,
            ))
        };
        ctx_b.emit_ibc_event(core_event);

        for module_event in extras.events {
            ctx_b.emit_ibc_event(IbcEvent::AppModule(module_event));
        }

        for log_message in extras.log {
            ctx_b.log_message(log_message);
        }
    }

    Ok(())
}

fn recv_packet_validate<ValCtx>(ctx_b: &ValCtx, msg: MsgRecvPacket) -> Result<(), ContextError>
where
    ValCtx: ValidationContext,
{
    // Note: this contains the validation for `write_acknowledgement` as well.
    recv_packet::validate(ctx_b, &msg)

    // nothing to validate with the module, since `onRecvPacket` cannot fail.
}

fn recv_packet_execute<ExecCtx>(
    ctx_b: &mut ExecCtx,
    module_id: ModuleId,
    msg: MsgRecvPacket,
) -> Result<(), ContextError>
where
    ExecCtx: ExecutionContext,
{
    let chan_end_path_on_b = ChannelEndPath::new(&msg.packet.port_on_b, &msg.packet.chan_on_b);
    let chan_end_on_b = ctx_b.channel_end(&chan_end_path_on_b)?;

    // Check if another relayer already relayed the packet.
    // We don't want to fail the transaction in this case.
    {
        let packet_already_received = match chan_end_on_b.ordering {
            // Note: ibc-go doesn't make the check for `Order::None` channels
            Order::None => false,
            Order::Unordered => {
                let packet = msg.packet.clone();
                let receipt_path_on_b =
                    ReceiptPath::new(&packet.port_on_b, &packet.chan_on_b, packet.sequence);
                ctx_b.get_packet_receipt(&receipt_path_on_b).is_ok()
            }
            Order::Ordered => {
                let seq_recv_path_on_b =
                    SeqRecvPath::new(&msg.packet.port_on_b, &msg.packet.chan_on_b);
                let next_seq_recv = ctx_b.get_next_sequence_recv(&seq_recv_path_on_b)?;

                // the sequence number has already been incremented, so
                // another relayer already relayed the packet
                msg.packet.sequence < next_seq_recv
            }
        };

        if packet_already_received {
            return Ok(());
        }
    }

    let module = ctx_b
        .get_route_mut(&module_id)
        .ok_or(ChannelError::RouteNotFound)?;

    let (extras, acknowledgement) = module.on_recv_packet_execute(&msg.packet, &msg.signer);

    // state changes
    {
        // `recvPacket` core handler state changes
        match chan_end_on_b.ordering {
            Order::Unordered => {
                let path = ReceiptPath {
                    port_id: msg.packet.port_on_b.clone(),
                    channel_id: msg.packet.chan_on_b.clone(),
                    sequence: msg.packet.sequence,
                };

                ctx_b.store_packet_receipt(path, Receipt::Ok)?;
            }
            Order::Ordered => {
                let seq_recv_path = SeqRecvPath::new(&msg.packet.port_on_b, &msg.packet.chan_on_b);
                let next_seq_recv = ctx_b.get_next_sequence_recv(&seq_recv_path)?;
                ctx_b.store_next_sequence_recv(seq_recv_path, next_seq_recv.increment())?;
            }
            _ => {}
        }
        let ack_path = AckPath::new(
            &msg.packet.port_on_b,
            &msg.packet.chan_on_b,
            msg.packet.sequence,
        );
        // `writeAcknowledgement` handler state changes
        ctx_b.store_packet_acknowledgement(ack_path, ctx_b.ack_commitment(&acknowledgement))?;
    }

    // emit events and logs
    {
        ctx_b.log_message("success: packet receive".to_string());
        ctx_b.log_message("success: packet write acknowledgement".to_string());

        let conn_id_on_b = &chan_end_on_b.connection_hops()[0];
        ctx_b.emit_ibc_event(IbcEvent::ReceivePacket(ReceivePacket::new(
            msg.packet.clone(),
            chan_end_on_b.ordering,
            conn_id_on_b.clone(),
        )));
        ctx_b.emit_ibc_event(IbcEvent::WriteAcknowledgement(WriteAcknowledgement::new(
            msg.packet,
            acknowledgement,
            conn_id_on_b.clone(),
        )));

        for module_event in extras.events {
            ctx_b.emit_ibc_event(IbcEvent::AppModule(module_event));
        }

        for log_message in extras.log {
            ctx_b.log_message(log_message);
        }
    }

    Ok(())
}

fn acknowledgement_packet_validate<ValCtx>(
    ctx_a: &ValCtx,
    module_id: ModuleId,
    msg: MsgAcknowledgement,
) -> Result<(), ContextError>
where
    ValCtx: ValidationContext,
{
    acknowledgement::validate(ctx_a, &msg)?;

    let module = ctx_a
        .get_route(&module_id)
        .ok_or(ChannelError::RouteNotFound)?;

    module
        .on_acknowledgement_packet_validate(&msg.packet, &msg.acknowledgement, &msg.signer)
        .map_err(ContextError::PacketError)
}

fn acknowledgement_packet_execute<ExecCtx>(
    ctx_a: &mut ExecCtx,
    module_id: ModuleId,
    msg: MsgAcknowledgement,
) -> Result<(), ContextError>
where
    ExecCtx: ExecutionContext,
{
    let chan_end_path_on_a = ChannelEndPath::new(&msg.packet.port_on_a, &msg.packet.chan_on_a);
    let chan_end_on_a = ctx_a.channel_end(&chan_end_path_on_a)?;
    let conn_id_on_a = &chan_end_on_a.connection_hops()[0];

    // In all cases, this event is emitted
    ctx_a.emit_ibc_event(IbcEvent::AcknowledgePacket(AcknowledgePacket::new(
        msg.packet.clone(),
        chan_end_on_a.ordering,
        conn_id_on_a.clone(),
    )));

    let commitment_path = CommitmentPath::new(
        &msg.packet.port_on_a,
        &msg.packet.chan_on_a,
        msg.packet.sequence,
    );

    // check if we're in the NO-OP case
    if ctx_a.get_packet_commitment(&commitment_path).is_err() {
        // This error indicates that the timeout has already been relayed
        // or there is a misconfigured relayer attempting to prove a timeout
        // for a packet never sent. Core IBC will treat this error as a no-op in order to
        // prevent an entire relay transaction from failing and consuming unnecessary fees.
        return Ok(());
    };

    let module = ctx_a
        .get_route_mut(&module_id)
        .ok_or(ChannelError::RouteNotFound)?;

    let (extras, cb_result) =
        module.on_acknowledgement_packet_execute(&msg.packet, &msg.acknowledgement, &msg.signer);

    cb_result?;

    // apply state changes
    {
        let commitment_path = CommitmentPath {
            port_id: msg.packet.port_on_a.clone(),
            channel_id: msg.packet.chan_on_a.clone(),
            sequence: msg.packet.sequence,
        };
        ctx_a.delete_packet_commitment(commitment_path)?;

        if let Order::Ordered = chan_end_on_a.ordering {
            // Note: in validation, we verified that `msg.packet.sequence == nextSeqRecv`
            // (where `nextSeqRecv` is the value in the store)
            let seq_ack_path = SeqAckPath::new(&msg.packet.port_on_a, &msg.packet.chan_on_a);
            ctx_a.store_next_sequence_ack(seq_ack_path, msg.packet.sequence.increment())?;
        }
    }

    // emit events and logs
    {
        ctx_a.log_message("success: packet acknowledgement".to_string());

        // Note: Acknowledgement event was emitted at the beginning

        for module_event in extras.events {
            ctx_a.emit_ibc_event(IbcEvent::AppModule(module_event));
        }

        for log_message in extras.log {
            ctx_a.log_message(log_message);
        }
    }

    Ok(())
}

enum TimeoutMsgType {
    Timeout(MsgTimeout),
    TimeoutOnClose(MsgTimeoutOnClose),
}

fn timeout_packet_validate<ValCtx>(
    ctx_a: &ValCtx,
    module_id: ModuleId,
    timeout_msg_type: TimeoutMsgType,
) -> Result<(), ContextError>
where
    ValCtx: ValidationContext,
{
    match &timeout_msg_type {
        TimeoutMsgType::Timeout(msg) => timeout::validate(ctx_a, msg),
        TimeoutMsgType::TimeoutOnClose(msg) => timeout_on_close::validate(ctx_a, msg),
    }?;

    let module = ctx_a
        .get_route(&module_id)
        .ok_or(ChannelError::RouteNotFound)?;

    let (packet, signer) = match timeout_msg_type {
        TimeoutMsgType::Timeout(msg) => (msg.packet, msg.signer),
        TimeoutMsgType::TimeoutOnClose(msg) => (msg.packet, msg.signer),
    };

    module
        .on_timeout_packet_validate(&packet, &signer)
        .map_err(ContextError::PacketError)
}

fn timeout_packet_execute<ExecCtx>(
    ctx_a: &mut ExecCtx,
    module_id: ModuleId,
    timeout_msg_type: TimeoutMsgType,
) -> Result<(), ContextError>
where
    ExecCtx: ExecutionContext,
{
    let (packet, signer) = match timeout_msg_type {
        TimeoutMsgType::Timeout(msg) => (msg.packet, msg.signer),
        TimeoutMsgType::TimeoutOnClose(msg) => (msg.packet, msg.signer),
    };
    let chan_end_path_on_a = ChannelEndPath::new(&packet.port_on_a, &packet.chan_on_a);
    let chan_end_on_a = ctx_a.channel_end(&chan_end_path_on_a)?;

    // In all cases, this event is emitted
    ctx_a.emit_ibc_event(IbcEvent::TimeoutPacket(TimeoutPacket::new(
        packet.clone(),
        chan_end_on_a.ordering,
    )));

    let commitment_path_on_a =
        CommitmentPath::new(&packet.port_on_a, &packet.chan_on_a, packet.sequence);

    // check if we're in the NO-OP case
    if ctx_a.get_packet_commitment(&commitment_path_on_a).is_err() {
        // This error indicates that the timeout has already been relayed
        // or there is a misconfigured relayer attempting to prove a timeout
        // for a packet never sent. Core IBC will treat this error as a no-op in order to
        // prevent an entire relay transaction from failing and consuming unnecessary fees.
        return Ok(());
    };

    let module = ctx_a
        .get_route_mut(&module_id)
        .ok_or(ChannelError::RouteNotFound)?;

    let (extras, cb_result) = module.on_timeout_packet_execute(&packet, &signer);

    cb_result?;

    // apply state changes
    let chan_end_on_a = {
        let commitment_path = CommitmentPath {
            port_id: packet.port_on_a.clone(),
            channel_id: packet.chan_on_a.clone(),
            sequence: packet.sequence,
        };
        ctx_a.delete_packet_commitment(commitment_path)?;

        if let Order::Ordered = chan_end_on_a.ordering {
            let mut chan_end_on_a = chan_end_on_a;
            chan_end_on_a.state = State::Closed;
            ctx_a.store_channel(chan_end_path_on_a, chan_end_on_a.clone())?;

            chan_end_on_a
        } else {
            chan_end_on_a
        }
    };

    // emit events and logs
    {
        ctx_a.log_message("success: packet timeout".to_string());

        if let Order::Ordered = chan_end_on_a.ordering {
            let conn_id_on_a = chan_end_on_a.connection_hops()[0].clone();

            ctx_a.emit_ibc_event(IbcEvent::ChannelClosed(ChannelClosed::new(
                packet.port_on_a.clone(),
                packet.chan_on_a.clone(),
                chan_end_on_a.counterparty().port_id.clone(),
                chan_end_on_a.counterparty().channel_id.clone(),
                conn_id_on_a,
                chan_end_on_a.ordering,
            )));
        }

        for module_event in extras.events {
            ctx_a.emit_ibc_event(IbcEvent::AppModule(module_event));
        }

        for log_message in extras.log {
            ctx_a.log_message(log_message);
        }
    }

    Ok(())
}
