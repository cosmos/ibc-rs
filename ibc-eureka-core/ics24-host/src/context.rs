use core::time::Duration;

use ibc_eureka_core_channel_types::commitment::{AcknowledgementCommitment, PacketCommitment};
use ibc_eureka_core_channel_types::packet::Receipt;
use ibc_eureka_core_client_context::prelude::*;
use ibc_eureka_core_client_types::Height;
use ibc_eureka_core_commitment_types::commitment::CommitmentPrefix;
use ibc_eureka_core_handler_types::events::IbcEvent;
use ibc_eureka_core_host_types::error::HostError;
use ibc_eureka_core_host_types::identifiers::Sequence;
use ibc_eureka_core_host_types::path::{
    AckPathV2 as AckPath, CommitmentPathV2 as CommitmentPath, ReceiptPathV2 as ReceiptPath,
    SeqAckPathV2 as SeqAckPath, SeqRecvPathV2 as SeqRecvPath, SeqSendPathV2 as SeqSendPath,
};
use ibc_primitives::prelude::*;
use ibc_primitives::{Signer, Timestamp};

use crate::utils::calculate_block_delay;

/// Context to be implemented by the host that provides all "read-only" methods.
///
/// Trait used for the top-level `validate` entrypoint in the `ibc-eureka-core` crate.
pub trait ValidationContext {
    type V: ClientValidationContext;
    /// The client state type for the host chain.
    type HostClientState: ClientStateValidation<Self::V>;
    /// The consensus state type for the host chain.
    type HostConsensusState: ConsensusState;

    /// Retrieve the context that implements all clients' `ValidationContext`.
    fn get_client_validation_context(&self) -> &Self::V;

    /// Returns the current height of the local chain.
    fn host_height(&self) -> Result<Height, HostError>;

    /// Returns the current timestamp of the local chain.
    fn host_timestamp(&self) -> Result<Timestamp, HostError>;

    /// Returns the `ConsensusState` of the host (local) chain at a specific height.
    fn host_consensus_state(&self, height: &Height) -> Result<Self::HostConsensusState, HostError>;

    /// Returns a natural number, counting how many clients have been created
    /// thus far. The value of this counter should increase only via method
    /// `ExecutionContext::increase_client_counter`.
    fn client_counter(&self) -> Result<u64, HostError>;

    /// Validates the `ClientState` of the host chain stored on the counterparty
    /// chain against the host's internal state.
    ///
    /// For more information on the specific requirements for validating the
    /// client state of a host chain, please refer to the [ICS24 host
    /// requirements](https://github.com/cosmos/ibc/tree/main/spec/core/ics-024-host-requirements#client-state-validation)
    ///
    /// Additionally, implementations specific to individual chains can be found
    /// in the `ibc-eureka-core/ics24-host` module.
    fn validate_self_client(
        &self,
        client_state_of_host_on_counterparty: Self::HostClientState,
    ) -> Result<(), HostError>;

    /// Returns the prefix that the local chain uses in the KV store.
    fn commitment_prefix(&self) -> CommitmentPrefix;

    /// Returns the sequence number for the next packet to be sent for the given store path
    fn get_next_sequence_send(&self, seq_send_path: &SeqSendPath) -> Result<Sequence, HostError>;

    /// Returns the sequence number for the next packet to be received for the given store path
    fn get_next_sequence_recv(&self, seq_recv_path: &SeqRecvPath) -> Result<Sequence, HostError>;

    /// Returns the sequence number for the next packet to be acknowledged for the given store path
    fn get_next_sequence_ack(&self, seq_ack_path: &SeqAckPath) -> Result<Sequence, HostError>;

    /// Returns the packet commitment for the given store path
    fn get_packet_commitment(
        &self,
        commitment_path: &CommitmentPath,
    ) -> Result<PacketCommitment, HostError>;

    /// Returns the packet receipt for the given store path. This receipt is
    /// used to acknowledge the successful processing of a received packet, and
    /// must not be pruned.
    ///
    /// If the receipt is present in the host's state, return `Receipt::Ok`,
    /// indicating the packet has already been processed. If the receipt is
    /// absent, return `Receipt::None`, indicating the packet has not been
    /// received.
    fn get_packet_receipt(&self, receipt_path: &ReceiptPath) -> Result<Receipt, HostError>;

    /// Returns the packet acknowledgement for the given store path
    fn get_packet_acknowledgement(
        &self,
        ack_path: &AckPath,
    ) -> Result<AcknowledgementCommitment, HostError>;

    /// Returns a counter on the number of channel ids have been created thus far.
    /// The value of this counter should increase only via method
    /// `ExecutionContext::increase_channel_counter`.
    fn channel_counter(&self) -> Result<u64, HostError>;

    /// Returns the maximum expected time per block
    fn max_expected_time_per_block(&self) -> Duration;

    /// Calculates the block delay period using the connection's delay period and the maximum
    /// expected time per block.
    fn block_delay(&self, delay_period_time: &Duration) -> u64 {
        calculate_block_delay(delay_period_time, &self.max_expected_time_per_block())
    }

    /// Validates the `signer` field of IBC messages, which represents the address
    /// of the user/relayer that signed the given message.
    fn validate_message_signer(&self, signer: &Signer) -> Result<(), HostError>;
}

/// Context to be implemented by the host that provides all "write-only" methods.
///
/// Trait used for the top-level `execute` and `dispatch` entrypoints in the `ibc-eureka-core` crate.
pub trait ExecutionContext: ValidationContext {
    type E: ClientExecutionContext;

    /// Retrieve the context that implements all clients' `ExecutionContext`.
    fn get_client_execution_context(&mut self) -> &mut Self::E;

    /// Called upon client creation.
    /// Increases the counter, that keeps track of how many clients have been created.
    fn increase_client_counter(&mut self) -> Result<(), HostError>;

    /// Called upon connection identifier creation (Init or Try process).
    /// Increases the counter which keeps track of how many connections have been created.
    fn increase_connection_counter(&mut self) -> Result<(), HostError>;

    /// Stores the given packet commitment at the given store path
    fn store_packet_commitment(
        &mut self,
        commitment_path: &CommitmentPath,
        commitment: PacketCommitment,
    ) -> Result<(), HostError>;

    /// Deletes the packet commitment at the given store path
    fn delete_packet_commitment(
        &mut self,
        commitment_path: &CommitmentPath,
    ) -> Result<(), HostError>;

    /// Stores the given packet receipt at the given store path
    fn store_packet_receipt(
        &mut self,
        receipt_path: &ReceiptPath,
        receipt: Receipt,
    ) -> Result<(), HostError>;

    /// Stores the given packet acknowledgement at the given store path
    fn store_packet_acknowledgement(
        &mut self,
        ack_path: &AckPath,
        ack_commitment: AcknowledgementCommitment,
    ) -> Result<(), HostError>;

    /// Deletes the packet acknowledgement at the given store path
    fn delete_packet_acknowledgement(&mut self, ack_path: &AckPath) -> Result<(), HostError>;

    /// Stores the given `nextSequenceSend` number at the given store path
    fn store_next_sequence_send(
        &mut self,
        seq_send_path: &SeqSendPath,
        seq: Sequence,
    ) -> Result<(), HostError>;

    /// Stores the given `nextSequenceRecv` number at the given store path
    fn store_next_sequence_recv(
        &mut self,
        seq_recv_path: &SeqRecvPath,
        seq: Sequence,
    ) -> Result<(), HostError>;

    /// Stores the given `nextSequenceAck` number at the given store path
    fn store_next_sequence_ack(
        &mut self,
        seq_ack_path: &SeqAckPath,
        seq: Sequence,
    ) -> Result<(), HostError>;

    /// Called upon channel identifier creation (Init or Try message processing).
    /// Increases the counter, that keeps track of how many channels have been created.
    fn increase_channel_counter(&mut self) -> Result<(), HostError>;

    /// Emit the given IBC event
    fn emit_ibc_event(&mut self, event: IbcEvent) -> Result<(), HostError>;

    /// Log the given message.
    fn log_message(&mut self, message: String) -> Result<(), HostError>;
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
