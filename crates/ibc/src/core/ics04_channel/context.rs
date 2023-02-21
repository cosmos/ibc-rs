//! ICS4 (channel) context.

use crate::core::ics02_client::client_state::ClientState;
use crate::core::ics24_host::path::{ChannelEndPath, ClientConsensusStatePath, SeqSendPath};
use crate::core::{ContextError, ValidationContext};
use crate::prelude::*;
use core::time::Duration;
use num_traits::float::FloatCore;

use crate::core::ics02_client::consensus_state::ConsensusState;
use crate::core::ics03_connection::connection::ConnectionEnd;
use crate::core::ics04_channel::channel::ChannelEnd;
use crate::core::ics04_channel::commitment::PacketCommitment;
use crate::core::ics24_host::identifier::{ClientId, ConnectionId};
use crate::timestamp::Timestamp;

use super::packet::Sequence;
use super::timeout::TimeoutHeight;

pub trait SendPacketReader {
    /// Returns the ChannelEnd for the given `port_id` and `chan_id`.
    fn channel_end(&self, channel_end_path: &ChannelEndPath) -> Result<ChannelEnd, ContextError>;

    /// Returns the ConnectionState for the given identifier `connection_id`.
    fn connection_end(&self, connection_id: &ConnectionId) -> Result<ConnectionEnd, ContextError>;

    /// Returns the ClientState for the given identifier `client_id`. Necessary dependency towards
    /// proof verification.
    fn client_state(&self, client_id: &ClientId) -> Result<Box<dyn ClientState>, ContextError>;

    fn client_consensus_state(
        &self,
        client_cons_state_path: &ClientConsensusStatePath,
    ) -> Result<Box<dyn ConsensusState>, ContextError>;

    fn get_next_sequence_send(&self, seq_send_path: &SeqSendPath)
        -> Result<Sequence, ContextError>;

    fn hash(&self, value: &[u8]) -> Vec<u8>;

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
}

impl<T> SendPacketReader for T
where
    T: ValidationContext,
{
    fn channel_end(&self, channel_end_path: &ChannelEndPath) -> Result<ChannelEnd, ContextError> {
        self.channel_end(channel_end_path)
    }

    fn connection_end(&self, connection_id: &ConnectionId) -> Result<ConnectionEnd, ContextError> {
        self.connection_end(connection_id)
    }

    fn client_state(&self, client_id: &ClientId) -> Result<Box<dyn ClientState>, ContextError> {
        self.client_state(client_id)
    }

    fn client_consensus_state(
        &self,
        client_cons_state_path: &ClientConsensusStatePath,
    ) -> Result<Box<dyn ConsensusState>, ContextError> {
        self.consensus_state(client_cons_state_path)
    }

    fn get_next_sequence_send(
        &self,
        seq_send_path: &SeqSendPath,
    ) -> Result<Sequence, ContextError> {
        self.get_next_sequence_send(seq_send_path)
    }

    fn hash(&self, value: &[u8]) -> Vec<u8> {
        self.hash(value)
    }
}

pub fn calculate_block_delay(
    delay_period_time: &Duration,
    max_expected_time_per_block: &Duration,
) -> u64 {
    if max_expected_time_per_block.is_zero() {
        return 0;
    }

    FloatCore::ceil(delay_period_time.as_secs_f64() / max_expected_time_per_block.as_secs_f64())
        as u64
}
