use core::time::Duration;

use ibc_core_channel_types::channel::ChannelEnd;
use ibc_core_channel_types::commitment::{AcknowledgementCommitment, PacketCommitment};
use ibc_core_channel_types::packet::Receipt;
use ibc_core_client_context::prelude::*;
use ibc_core_client_types::Height;
use ibc_core_commitment_types::commitment::CommitmentPrefix;
use ibc_core_connection_types::version::{pick_version, Version as ConnectionVersion};
use ibc_core_connection_types::ConnectionEnd;
use ibc_core_handler_types::error::ContextError;
use ibc_core_handler_types::events::IbcEvent;
use ibc_core_host_types::identifiers::{ConnectionId, Sequence};
use ibc_core_host_types::path::{
    AckPath, ChannelEndPath, ClientConnectionPath, CommitmentPath, ConnectionPath, ReceiptPath,
    SeqAckPath, SeqRecvPath, SeqSendPath,
};
use ibc_primitives::prelude::*;
use ibc_primitives::{Signer, Timestamp};

use crate::utils::calculate_block_delay;

/// Context to be implemented by the host that provides all "read-only" methods.
///
/// Trait used for the top-level `validate` entrypoint in the `ibc-core` crate.
pub trait ValidationContext {
    type V: ClientValidationContext;
    /// The client state type for the host chain.
    type HostClientState: ClientStateValidation<Self::V>;
    /// The consensus state type for the host chain.
    type HostConsensusState: ConsensusState;

    /// Retrieve the context that implements all clients' `ValidationContext`.
    fn get_client_validation_context(&self) -> &Self::V;

    /// Returns the current height of the local chain.
    fn host_height(&self) -> Result<Height, ContextError>;

    /// Returns the current timestamp of the local chain.
    fn host_timestamp(&self) -> Result<Timestamp, ContextError>;

    /// Returns the `ConsensusState` of the host (local) chain at a specific height.
    fn host_consensus_state(
        &self,
        height: &Height,
    ) -> Result<Self::HostConsensusState, ContextError>;

    /// Returns a natural number, counting how many clients have been created
    /// thus far. The value of this counter should increase only via method
    /// `ExecutionContext::increase_client_counter`.
    fn client_counter(&self) -> Result<u64, ContextError>;

    /// Returns the ConnectionEnd for the given identifier `conn_id`.
    fn connection_end(&self, conn_id: &ConnectionId) -> Result<ConnectionEnd, ContextError>;

    /// Validates the `ClientState` of the host chain stored on the counterparty
    /// chain against the host's internal state.
    ///
    /// For more information on the specific requirements for validating the
    /// client state of a host chain, please refer to the [ICS24 host
    /// requirements](https://github.com/cosmos/ibc/tree/main/spec/core/ics-024-host-requirements#client-state-validation)
    ///
    /// Additionally, implementations specific to individual chains can be found
    /// in the `ibc-core/ics24-host` module.
    fn validate_self_client(
        &self,
        client_state_of_host_on_counterparty: Self::HostClientState,
    ) -> Result<(), ContextError>;

    /// Returns the prefix that the local chain uses in the KV store.
    fn commitment_prefix(&self) -> CommitmentPrefix;

    /// Returns a counter on how many connections have been created thus far.
    fn connection_counter(&self) -> Result<u64, ContextError>;

    /// Function required by ICS-03. Returns the list of all possible versions that the connection
    /// handshake protocol supports.
    fn get_compatible_versions(&self) -> Vec<ConnectionVersion> {
        ConnectionVersion::compatibles()
    }

    /// Function required by ICS-03. Returns one version out of the supplied list of versions, which the
    /// connection handshake protocol prefers.
    fn pick_version(
        &self,
        counterparty_candidate_versions: &[ConnectionVersion],
    ) -> Result<ConnectionVersion, ContextError> {
        let version = pick_version(
            &self.get_compatible_versions(),
            counterparty_candidate_versions,
        )?;
        Ok(version)
    }

    /// Returns the `ChannelEnd` for the given `port_id` and `chan_id`.
    fn channel_end(&self, channel_end_path: &ChannelEndPath) -> Result<ChannelEnd, ContextError>;

    /// Returns the sequence number for the next packet to be sent for the given store path
    fn get_next_sequence_send(&self, seq_send_path: &SeqSendPath)
        -> Result<Sequence, ContextError>;

    /// Returns the sequence number for the next packet to be received for the given store path
    fn get_next_sequence_recv(&self, seq_recv_path: &SeqRecvPath)
        -> Result<Sequence, ContextError>;

    /// Returns the sequence number for the next packet to be acknowledged for the given store path
    fn get_next_sequence_ack(&self, seq_ack_path: &SeqAckPath) -> Result<Sequence, ContextError>;

    /// Returns the packet commitment for the given store path
    fn get_packet_commitment(
        &self,
        commitment_path: &CommitmentPath,
    ) -> Result<PacketCommitment, ContextError>;

    /// Returns the packet receipt for the given store path
    fn get_packet_receipt(&self, receipt_path: &ReceiptPath) -> Result<Receipt, ContextError>;

    /// Returns the packet acknowledgement for the given store path
    fn get_packet_acknowledgement(
        &self,
        ack_path: &AckPath,
    ) -> Result<AcknowledgementCommitment, ContextError>;

    /// Returns a counter on the number of channel ids have been created thus far.
    /// The value of this counter should increase only via method
    /// `ExecutionContext::increase_channel_counter`.
    fn channel_counter(&self) -> Result<u64, ContextError>;

    /// Returns the maximum expected time per block
    fn max_expected_time_per_block(&self) -> Duration;

    /// Calculates the block delay period using the connection's delay period and the maximum
    /// expected time per block.
    fn block_delay(&self, delay_period_time: &Duration) -> u64 {
        calculate_block_delay(delay_period_time, &self.max_expected_time_per_block())
    }

    /// Validates the `signer` field of IBC messages, which represents the address
    /// of the user/relayer that signed the given message.
    fn validate_message_signer(&self, signer: &Signer) -> Result<(), ContextError>;
}

/// Context to be implemented by the host that provides all "write-only" methods.
///
/// Trait used for the top-level `execute` and `dispatch` entrypoints in the `ibc-core` crate.
pub trait ExecutionContext: ValidationContext {
    type E: ClientExecutionContext;

    /// Retrieve the context that implements all clients' `ExecutionContext`.
    fn get_client_execution_context(&mut self) -> &mut Self::E;

    /// Called upon client creation.
    /// Increases the counter which keeps track of how many clients have been created.
    /// Should never fail.
    fn increase_client_counter(&mut self) -> Result<(), ContextError>;

    /// Stores the given connection_end at path
    fn store_connection(
        &mut self,
        connection_path: &ConnectionPath,
        connection_end: ConnectionEnd,
    ) -> Result<(), ContextError>;

    /// Stores the given connection_id at a path associated with the client_id.
    fn store_connection_to_client(
        &mut self,
        client_connection_path: &ClientConnectionPath,
        conn_id: ConnectionId,
    ) -> Result<(), ContextError>;

    /// Called upon connection identifier creation (Init or Try process).
    /// Increases the counter which keeps track of how many connections have been created.
    /// Should never fail.
    fn increase_connection_counter(&mut self) -> Result<(), ContextError>;

    /// Stores the given packet commitment at the given store path
    fn store_packet_commitment(
        &mut self,
        commitment_path: &CommitmentPath,
        commitment: PacketCommitment,
    ) -> Result<(), ContextError>;

    /// Deletes the packet commitment at the given store path
    fn delete_packet_commitment(
        &mut self,
        commitment_path: &CommitmentPath,
    ) -> Result<(), ContextError>;

    /// Stores the given packet receipt at the given store path
    fn store_packet_receipt(
        &mut self,
        receipt_path: &ReceiptPath,
        receipt: Receipt,
    ) -> Result<(), ContextError>;

    /// Stores the given packet acknowledgement at the given store path
    fn store_packet_acknowledgement(
        &mut self,
        ack_path: &AckPath,
        ack_commitment: AcknowledgementCommitment,
    ) -> Result<(), ContextError>;

    /// Deletes the packet acknowledgement at the given store path
    fn delete_packet_acknowledgement(&mut self, ack_path: &AckPath) -> Result<(), ContextError>;

    /// Stores the given channel_end at a path associated with the port_id and channel_id.
    fn store_channel(
        &mut self,
        channel_end_path: &ChannelEndPath,
        channel_end: ChannelEnd,
    ) -> Result<(), ContextError>;

    /// Stores the given `nextSequenceSend` number at the given store path
    fn store_next_sequence_send(
        &mut self,
        seq_send_path: &SeqSendPath,
        seq: Sequence,
    ) -> Result<(), ContextError>;

    /// Stores the given `nextSequenceRecv` number at the given store path
    fn store_next_sequence_recv(
        &mut self,
        seq_recv_path: &SeqRecvPath,
        seq: Sequence,
    ) -> Result<(), ContextError>;

    /// Stores the given `nextSequenceAck` number at the given store path
    fn store_next_sequence_ack(
        &mut self,
        seq_ack_path: &SeqAckPath,
        seq: Sequence,
    ) -> Result<(), ContextError>;

    /// Called upon channel identifier creation (Init or Try message processing).
    /// Increases the counter which keeps track of how many channels have been created.
    /// Should never fail.
    fn increase_channel_counter(&mut self) -> Result<(), ContextError>;

    /// Emit the given IBC event
    fn emit_ibc_event(&mut self, event: IbcEvent) -> Result<(), ContextError>;

    /// Log the given message.
    fn log_message(&mut self, message: String) -> Result<(), ContextError>;
}

/// Convenient type alias for `ClientStateRef`, providing access to client
/// validation methods within the context.
pub type ClientStateRef<Ctx> =
    <<Ctx as ValidationContext>::V as ClientValidationContext>::ClientStateRef;

/// Convenient type alias for `ClientStateMut`, providing access to client
/// execution methods within the context.
pub type ClientStateMut<Ctx> =
    <<Ctx as ExecutionContext>::E as ClientExecutionContext>::ClientStateMut;

/// Convenient type alias for `ConsensusStateRef`, providing access to client
/// validation methods within the context.
pub type ConsensusStateRef<Ctx> =
    <<Ctx as ValidationContext>::V as ClientValidationContext>::ConsensusStateRef;
