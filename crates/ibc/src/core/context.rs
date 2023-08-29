use crate::prelude::*;

use crate::signer::Signer;
use alloc::string::String;
use core::time::Duration;
use derive_more::From;
use displaydoc::Display;
use ibc_proto::google::protobuf::Any;

use crate::core::events::IbcEvent;
use crate::core::ics02_client::error::ClientError;
use crate::core::ics03_connection::connection::ConnectionEnd;
use crate::core::ics03_connection::error::ConnectionError;
use crate::core::ics03_connection::version::{
    get_compatible_versions, pick_version, Version as ConnectionVersion,
};
use crate::core::ics04_channel::channel::ChannelEnd;
use crate::core::ics04_channel::commitment::{AcknowledgementCommitment, PacketCommitment};
use crate::core::ics04_channel::context::calculate_block_delay;
use crate::core::ics04_channel::error::ChannelError;
use crate::core::ics04_channel::error::PacketError;
use crate::core::ics04_channel::packet::{Receipt, Sequence};
use crate::core::ics23_commitment::commitment::CommitmentPrefix;
use crate::core::ics24_host::identifier::{ClientId, ConnectionId};
use crate::core::ics24_host::path::{
    AckPath, ChannelEndPath, ClientConnectionPath, ClientConsensusStatePath, CommitmentPath,
    ConnectionPath, ReceiptPath, SeqAckPath, SeqRecvPath, SeqSendPath,
};
use crate::core::timestamp::Timestamp;
use crate::Height;

use super::ics02_client::client_state::ClientState;
use super::ics02_client::consensus_state::ConsensusState;
use super::ics02_client::ClientExecutionContext;
use super::ics24_host::identifier::PortId;

#[cfg(feature = "grpc")]
use crate::core::{
    ics02_client::{client_state::Status, client_type::ClientType},
    ics03_connection::connection::IdentifiedConnectionEnd,
    ics04_channel::channel::IdentifiedChannelEnd,
    ics24_host::path::Path,
};

/// Top-level error
#[derive(Debug, Display, From)]
pub enum ContextError {
    /// ICS02 Client error: {0}
    ClientError(ClientError),
    /// ICS03 Connection error: {0}
    ConnectionError(ConnectionError),
    /// ICS04 Channel error: {0}
    ChannelError(ChannelError),
    /// ICS04 Packet error: {0}
    PacketError(PacketError),
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

/// Error returned from entrypoint functions [`dispatch`][super::dispatch], [`validate`][super::validate] and
/// [`execute`][super::execute].
#[derive(Debug, Display)]
pub enum RouterError {
    /// context error: `{0}`
    ContextError(ContextError),
    /// unknown type URL `{url}`
    UnknownMessageTypeUrl { url: String },
    /// the message is malformed and cannot be decoded error: `{0}`
    MalformedMessageBytes(ibc_proto::protobuf::Error),
    /// port `{port_id}` is unknown
    UnknownPort { port_id: PortId },
    /// module not found
    ModuleNotFound,
}

impl From<ContextError> for RouterError {
    fn from(error: ContextError) -> Self {
        Self::ContextError(error)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for RouterError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self {
            Self::ContextError(e) => Some(e),
            Self::MalformedMessageBytes(e) => Some(e),
            _ => None,
        }
    }
}

/// Context to be implemented by the host that provides all "read-only" methods.
///
/// Trait used for the top-level [`validate`](crate::core::validate) and the [`gRPC query services`](crate::services).
pub trait ValidationContext {
    type ClientValidationContext;
    type E: ClientExecutionContext;
    type AnyConsensusState: ConsensusState;
    type AnyClientState: ClientState<Self::ClientValidationContext, Self::E>;

    /// Retrieve the context that implements all clients' `ValidationContext`.
    fn get_client_validation_context(&self) -> &Self::ClientValidationContext;

    /// Returns the ClientState for the given identifier `client_id`.
    ///
    /// Note: Clients have the responsibility to store client states on client creation and update.
    fn client_state(&self, client_id: &ClientId) -> Result<Self::AnyClientState, ContextError>;

    /// Tries to decode the given `client_state` into a concrete light client state.
    fn decode_client_state(&self, client_state: Any) -> Result<Self::AnyClientState, ContextError>;

    /// Retrieve the consensus state for the given client ID at the specified
    /// height.
    ///
    /// Returns an error if no such state exists.
    ///
    /// Note: Clients have the responsibility to store consensus states on client creation and update.
    fn consensus_state(
        &self,
        client_cons_state_path: &ClientConsensusStatePath,
    ) -> Result<Self::AnyConsensusState, ContextError>;

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

    /// Returns the current height of the local chain.
    fn host_height(&self) -> Result<Height, ContextError>;

    /// Returns the current timestamp of the local chain.
    fn host_timestamp(&self) -> Result<Timestamp, ContextError>;

    /// Returns the `ConsensusState` of the host (local) chain at a specific height.
    fn host_consensus_state(
        &self,
        height: &Height,
    ) -> Result<Self::AnyConsensusState, ContextError>;

    /// Returns a natural number, counting how many clients have been created
    /// thus far. The value of this counter should increase only via method
    /// `ExecutionContext::increase_client_counter`.
    fn client_counter(&self) -> Result<u64, ContextError>;

    /// Returns the ConnectionEnd for the given identifier `conn_id`.
    fn connection_end(&self, conn_id: &ConnectionId) -> Result<ConnectionEnd, ContextError>;

    /// Validates the `ClientState` of the client (a client referring to host) stored on the counterparty chain against the host's internal state.
    ///
    /// For more information on the specific requirements for validating the
    /// client state of a host chain, please refer to the [ICS24 host
    /// requirements](https://github.com/cosmos/ibc/tree/main/spec/core/ics-024-host-requirements#client-state-validation)
    ///
    /// Additionally, implementations specific to individual chains can be found
    /// in the [hosts](crate::hosts) module.
    fn validate_self_client(
        &self,
        client_state_of_host_on_counterparty: Any,
    ) -> Result<(), ContextError>;

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

/// Context to be implemented by the host to provide proofs in gRPC query responses
///
/// Trait used for the [`gRPC query services`](crate::services).
#[cfg(feature = "grpc")]
pub trait ProvableContext {
    fn get_proof(&self, height: Height, path: &Path) -> Option<Vec<u8>>;
}

/// Context to be implemented by the host that provides gRPC query services.
///
/// Trait used for the [`gRPC query services`](crate::services).
#[cfg(feature = "grpc")]
pub trait QueryContext: ProvableContext + ValidationContext {
    // Client queries
    fn client_states(
        &self,
    ) -> Result<Vec<(ClientId, <Self as ValidationContext>::AnyClientState)>, ContextError>;
    fn consensus_states(
        &self,
        client_id: &ClientId,
    ) -> Result<Vec<(Height, <Self as ValidationContext>::AnyConsensusState)>, ContextError>;
    fn consensus_state_heights(&self, client_id: &ClientId) -> Result<Vec<Height>, ContextError>;
    fn client_status(&self, client_id: &ClientId) -> Result<Status, ContextError>;
    fn allowed_clients(&self) -> Vec<ClientType>;

    // Connection queries
    fn connection_ends(&self) -> Result<Vec<IdentifiedConnectionEnd>, ContextError>;
    fn client_connection_ends(
        &self,
        client_id: &ClientId,
    ) -> Result<Vec<ConnectionId>, ContextError>;

    // Channel queries
    fn channel_ends(&self) -> Result<Vec<IdentifiedChannelEnd>, ContextError>;
    fn connection_channel_ends(
        &self,
        connection_id: &ConnectionId,
    ) -> Result<Vec<IdentifiedChannelEnd>, ContextError>;

    // Packet queries
    fn packet_commitments(
        &self,
        channel_end_path: &ChannelEndPath,
    ) -> Result<Vec<CommitmentPath>, ContextError>;
    fn packet_acknowledgements(
        &self,
        channel_end_path: &ChannelEndPath,
        sequences: impl IntoIterator<Item = Sequence>,
    ) -> Result<Vec<AckPath>, ContextError>;
    fn unreceived_packets(
        &self,
        channel_end_path: &ChannelEndPath,
        sequences: impl IntoIterator<Item = Sequence>,
    ) -> Result<Vec<Sequence>, ContextError>;
    fn unreceived_acks(
        &self,
        channel_end_path: &ChannelEndPath,
        sequences: impl IntoIterator<Item = Sequence>,
    ) -> Result<Vec<Sequence>, ContextError>;
}

/// Context to be implemented by the host that provides all "write-only" methods.
///
/// Trait used for the top-level [`execute`](crate::core::execute) and [`dispatch`](crate::core::dispatch)
pub trait ExecutionContext: ValidationContext {
    /// Retrieve the context that implements all clients' `ExecutionContext`.
    fn get_client_execution_context(&mut self) -> &mut Self::E;

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
    fn increase_connection_counter(&mut self);

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
    fn increase_channel_counter(&mut self);

    /// Emit the given IBC event
    fn emit_ibc_event(&mut self, event: IbcEvent);

    /// Log the given message.
    fn log_message(&mut self, message: String);
}
