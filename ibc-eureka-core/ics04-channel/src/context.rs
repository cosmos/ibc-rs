//! ICS4 (channel) context.

use ibc_eureka_core_channel_types::commitment::PacketCommitment;
use ibc_eureka_core_client::context::prelude::*;
use ibc_eureka_core_handler_types::events::IbcEvent;
use ibc_eureka_core_host::types::error::HostError;
use ibc_eureka_core_host::types::identifiers::Sequence;
use ibc_eureka_core_host::types::path::{CommitmentPath, SeqSendPath};
use ibc_eureka_core_host::{ExecutionContext, ValidationContext};
use ibc_primitives::prelude::*;

/// Methods required in send packet validation, to be implemented by the host
pub trait SendPacketValidationContext {
    type V: ClientValidationContext;

    /// Retrieve the context that implements all clients' `ValidationContext`.
    fn get_client_validation_context(&self) -> &Self::V;

    fn get_next_sequence_send(&self, seq_send_path: &SeqSendPath) -> Result<Sequence, HostError>;
}

impl<T> SendPacketValidationContext for T
where
    T: ValidationContext,
{
    type V = T::V;

    fn get_client_validation_context(&self) -> &Self::V {
        self.get_client_validation_context()
    }

    fn get_next_sequence_send(&self, seq_send_path: &SeqSendPath) -> Result<Sequence, HostError> {
        self.get_next_sequence_send(seq_send_path)
    }
}

/// Methods required in send packet execution, to be implemented by the host
pub trait SendPacketExecutionContext: SendPacketValidationContext {
    fn store_next_sequence_send(
        &mut self,
        seq_send_path: &SeqSendPath,
        seq: Sequence,
    ) -> Result<(), HostError>;

    fn store_packet_commitment(
        &mut self,
        commitment_path: &CommitmentPath,
        commitment: PacketCommitment,
    ) -> Result<(), HostError>;

    /// Ibc events
    fn emit_ibc_event(&mut self, event: IbcEvent) -> Result<(), HostError>;

    /// Logging facility
    fn log_message(&mut self, message: String) -> Result<(), HostError>;
}

impl<T> SendPacketExecutionContext for T
where
    T: ExecutionContext,
{
    fn store_next_sequence_send(
        &mut self,
        seq_send_path: &SeqSendPath,
        seq: Sequence,
    ) -> Result<(), HostError> {
        self.store_next_sequence_send(seq_send_path, seq)
    }

    fn store_packet_commitment(
        &mut self,
        commitment_path: &CommitmentPath,
        commitment: PacketCommitment,
    ) -> Result<(), HostError> {
        self.store_packet_commitment(commitment_path, commitment)
    }

    fn emit_ibc_event(&mut self, event: IbcEvent) -> Result<(), HostError> {
        self.emit_ibc_event(event)
    }

    fn log_message(&mut self, message: String) -> Result<(), HostError> {
        self.log_message(message)
    }
}
