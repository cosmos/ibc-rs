//! ICS4 (channel) context.

use ibc_core_channel_types::channel::ChannelEnd;
use ibc_core_channel_types::commitment::PacketCommitment;
use ibc_core_client::context::prelude::*;
use ibc_core_connection::types::ConnectionEnd;
use ibc_core_handler_types::error::ContextError;
use ibc_core_handler_types::events::IbcEvent;
use ibc_core_host::types::identifiers::{ConnectionId, Sequence};
use ibc_core_host::types::path::{ChannelEndPath, CommitmentPath, SeqSendPath};
use ibc_core_host::{ExecutionContext, ValidationContext};
use ibc_primitives::prelude::*;

/// Methods required in send packet validation, to be implemented by the host
pub trait SendPacketValidationContext {
    type V: ClientValidationContext;

    /// Retrieve the context that implements all clients' `ValidationContext`.
    fn get_client_validation_context(&self) -> &Self::V;

    /// Returns the ChannelEnd for the given `port_id` and `chan_id`.
    fn channel_end(&self, channel_end_path: &ChannelEndPath) -> Result<ChannelEnd, ContextError>;

    /// Returns the ConnectionState for the given identifier `connection_id`.
    fn connection_end(&self, connection_id: &ConnectionId) -> Result<ConnectionEnd, ContextError>;

    fn get_next_sequence_send(&self, seq_send_path: &SeqSendPath)
        -> Result<Sequence, ContextError>;
}

impl<T> SendPacketValidationContext for T
where
    T: ValidationContext,
{
    type V = T::V;

    fn get_client_validation_context(&self) -> &Self::V {
        self.get_client_validation_context()
    }

    fn channel_end(&self, channel_end_path: &ChannelEndPath) -> Result<ChannelEnd, ContextError> {
        self.channel_end(channel_end_path)
    }

    fn connection_end(&self, connection_id: &ConnectionId) -> Result<ConnectionEnd, ContextError> {
        self.connection_end(connection_id)
    }

    fn get_next_sequence_send(
        &self,
        seq_send_path: &SeqSendPath,
    ) -> Result<Sequence, ContextError> {
        self.get_next_sequence_send(seq_send_path)
    }
}

/// Methods required in send packet execution, to be implemented by the host
pub trait SendPacketExecutionContext: SendPacketValidationContext {
    fn store_next_sequence_send(
        &mut self,
        seq_send_path: &SeqSendPath,
        seq: Sequence,
    ) -> Result<(), ContextError>;

    fn store_packet_commitment(
        &mut self,
        commitment_path: &CommitmentPath,
        commitment: PacketCommitment,
    ) -> Result<(), ContextError>;

    /// Ibc events
    fn emit_ibc_event(&mut self, event: IbcEvent) -> Result<(), ContextError>;

    /// Logging facility
    fn log_message(&mut self, message: String) -> Result<(), ContextError>;
}

impl<T> SendPacketExecutionContext for T
where
    T: ExecutionContext,
{
    fn store_next_sequence_send(
        &mut self,
        seq_send_path: &SeqSendPath,
        seq: Sequence,
    ) -> Result<(), ContextError> {
        self.store_next_sequence_send(seq_send_path, seq)
    }

    fn store_packet_commitment(
        &mut self,
        commitment_path: &CommitmentPath,
        commitment: PacketCommitment,
    ) -> Result<(), ContextError> {
        self.store_packet_commitment(commitment_path, commitment)
    }

    fn emit_ibc_event(&mut self, event: IbcEvent) -> Result<(), ContextError> {
        self.emit_ibc_event(event)
    }

    fn log_message(&mut self, message: String) -> Result<(), ContextError> {
        self.log_message(message)
    }
}
