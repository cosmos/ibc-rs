//! Implementation of a global context mock. Used in testing handlers of all IBC modules.

use core::ops::Add;
use core::time::Duration;

use ibc::core::channel::types::channel::ChannelEnd;
use ibc::core::channel::types::commitment::{AcknowledgementCommitment, PacketCommitment};
use ibc::core::channel::types::error::{ChannelError, PacketError};
use ibc::core::channel::types::packet::Receipt;
use ibc::core::client::types::error::ClientError;
use ibc::core::client::types::Height;
use ibc::core::commitment_types::commitment::CommitmentPrefix;
use ibc::core::connection::types::error::ConnectionError;
use ibc::core::connection::types::ConnectionEnd;
use ibc::core::handler::types::error::ContextError;
use ibc::core::handler::types::events::IbcEvent;
use ibc::core::host::types::identifiers::{ConnectionId, Sequence};
use ibc::core::host::types::path::{
    AckPath, ChannelEndPath, ClientConnectionPath, CommitmentPath, ConnectionPath, ReceiptPath,
    SeqAckPath, SeqRecvPath, SeqSendPath,
};
use ibc::core::host::{ExecutionContext, ValidationContext};
use ibc::core::primitives::prelude::*;
use ibc::core::primitives::{Signer, Timestamp};

use super::types::MockContext;
use crate::testapp::ibc::clients::mock::client_state::MockClientState;
use crate::testapp::ibc::clients::mock::consensus_state::MockConsensusState;
use crate::testapp::ibc::clients::AnyConsensusState;

impl ValidationContext for MockContext {
    type V = Self;
    type HostClientState = MockClientState;
    type HostConsensusState = MockConsensusState;

    fn host_height(&self) -> Result<Height, ContextError> {
        Ok(self.latest_height())
    }

    fn host_timestamp(&self) -> Result<Timestamp, ContextError> {
        Ok(self
            .history
            .last()
            .expect("history cannot be empty")
            .timestamp()
            .add(self.block_time)
            .expect("Never fails"))
    }

    fn client_counter(&self) -> Result<u64, ContextError> {
        Ok(self.ibc_store.lock().client_ids_counter)
    }

    fn host_consensus_state(&self, height: &Height) -> Result<MockConsensusState, ContextError> {
        let cs: AnyConsensusState = match self.host_block(height) {
            Some(block_ref) => Ok(block_ref.clone().into()),
            None => Err(ClientError::MissingLocalConsensusState { height: *height }),
        }
        .map_err(ContextError::ClientError)?;

        match cs {
            AnyConsensusState::Mock(cs) => Ok(cs),
            _ => Err(ClientError::Other {
                description: "unexpected consensus state type".to_string(),
            }
            .into()),
        }
    }

    fn validate_self_client(
        &self,
        client_state_of_host_on_counterparty: Self::HostClientState,
    ) -> Result<(), ContextError> {
        if client_state_of_host_on_counterparty.is_frozen() {
            return Err(ClientError::ClientFrozen {
                description: String::new(),
            }
            .into());
        }

        let self_chain_id = &self.host_chain_id;
        let self_revision_number = self_chain_id.revision_number();
        if self_revision_number
            != client_state_of_host_on_counterparty
                .latest_height()
                .revision_number()
        {
            return Err(ContextError::ConnectionError(
                ConnectionError::InvalidClientState {
                    reason: format!(
                        "client is not in the same revision as the chain. expected: {}, got: {}",
                        self_revision_number,
                        client_state_of_host_on_counterparty
                            .latest_height()
                            .revision_number()
                    ),
                },
            ));
        }

        let host_current_height = self.latest_height().increment();
        if client_state_of_host_on_counterparty.latest_height() >= host_current_height {
            return Err(ContextError::ConnectionError(
                ConnectionError::InvalidClientState {
                    reason: format!(
                        "client has latest height {} greater than or equal to chain height {}",
                        client_state_of_host_on_counterparty.latest_height(),
                        host_current_height
                    ),
                },
            ));
        }

        Ok(())
    }

    fn connection_end(&self, cid: &ConnectionId) -> Result<ConnectionEnd, ContextError> {
        match self.ibc_store.lock().connections.get(cid) {
            Some(connection_end) => Ok(connection_end.clone()),
            None => Err(ConnectionError::ConnectionNotFound {
                connection_id: cid.clone(),
            }),
        }
        .map_err(ContextError::ConnectionError)
    }

    fn commitment_prefix(&self) -> CommitmentPrefix {
        CommitmentPrefix::try_from(b"mock".to_vec()).expect("Never fails")
    }

    fn connection_counter(&self) -> Result<u64, ContextError> {
        Ok(self.ibc_store.lock().connection_ids_counter)
    }

    fn channel_end(&self, chan_end_path: &ChannelEndPath) -> Result<ChannelEnd, ContextError> {
        let port_id = &chan_end_path.0;
        let channel_id = &chan_end_path.1;

        match self
            .ibc_store
            .lock()
            .channels
            .get(port_id)
            .and_then(|map| map.get(channel_id))
        {
            Some(channel_end) => Ok(channel_end.clone()),
            None => Err(ChannelError::ChannelNotFound {
                port_id: port_id.clone(),
                channel_id: channel_id.clone(),
            }),
        }
        .map_err(ContextError::ChannelError)
    }

    fn get_next_sequence_send(
        &self,
        seq_send_path: &SeqSendPath,
    ) -> Result<Sequence, ContextError> {
        let port_id = &seq_send_path.0;
        let channel_id = &seq_send_path.1;

        match self
            .ibc_store
            .lock()
            .next_sequence_send
            .get(port_id)
            .and_then(|map| map.get(channel_id))
        {
            Some(sequence) => Ok(*sequence),
            None => Err(PacketError::MissingNextSendSeq {
                port_id: port_id.clone(),
                channel_id: channel_id.clone(),
            }),
        }
        .map_err(ContextError::PacketError)
    }

    fn get_next_sequence_recv(
        &self,
        seq_recv_path: &SeqRecvPath,
    ) -> Result<Sequence, ContextError> {
        let port_id = &seq_recv_path.0;
        let channel_id = &seq_recv_path.1;

        match self
            .ibc_store
            .lock()
            .next_sequence_recv
            .get(port_id)
            .and_then(|map| map.get(channel_id))
        {
            Some(sequence) => Ok(*sequence),
            None => Err(PacketError::MissingNextRecvSeq {
                port_id: port_id.clone(),
                channel_id: channel_id.clone(),
            }),
        }
        .map_err(ContextError::PacketError)
    }

    fn get_next_sequence_ack(&self, seq_ack_path: &SeqAckPath) -> Result<Sequence, ContextError> {
        let port_id = &seq_ack_path.0;
        let channel_id = &seq_ack_path.1;

        match self
            .ibc_store
            .lock()
            .next_sequence_ack
            .get(port_id)
            .and_then(|map| map.get(channel_id))
        {
            Some(sequence) => Ok(*sequence),
            None => Err(PacketError::MissingNextAckSeq {
                port_id: port_id.clone(),
                channel_id: channel_id.clone(),
            }),
        }
        .map_err(ContextError::PacketError)
    }

    fn get_packet_commitment(
        &self,
        commitment_path: &CommitmentPath,
    ) -> Result<PacketCommitment, ContextError> {
        let port_id = &commitment_path.port_id;
        let channel_id = &commitment_path.channel_id;
        let seq = &commitment_path.sequence;

        match self
            .ibc_store
            .lock()
            .packet_commitment
            .get(port_id)
            .and_then(|map| map.get(channel_id))
            .and_then(|map| map.get(seq))
        {
            Some(commitment) => Ok(commitment.clone()),
            None => Err(PacketError::PacketCommitmentNotFound { sequence: *seq }),
        }
        .map_err(ContextError::PacketError)
    }

    fn get_packet_receipt(&self, receipt_path: &ReceiptPath) -> Result<Receipt, ContextError> {
        let port_id = &receipt_path.port_id;
        let channel_id = &receipt_path.channel_id;
        let seq = &receipt_path.sequence;

        match self
            .ibc_store
            .lock()
            .packet_receipt
            .get(port_id)
            .and_then(|map| map.get(channel_id))
            .and_then(|map| map.get(seq))
        {
            Some(receipt) => Ok(receipt.clone()),
            None => Err(PacketError::PacketReceiptNotFound { sequence: *seq }),
        }
        .map_err(ContextError::PacketError)
    }

    fn get_packet_acknowledgement(
        &self,
        ack_path: &AckPath,
    ) -> Result<AcknowledgementCommitment, ContextError> {
        let port_id = &ack_path.port_id;
        let channel_id = &ack_path.channel_id;
        let seq = &ack_path.sequence;

        match self
            .ibc_store
            .lock()
            .packet_acknowledgement
            .get(port_id)
            .and_then(|map| map.get(channel_id))
            .and_then(|map| map.get(seq))
        {
            Some(ack) => Ok(ack.clone()),
            None => Err(PacketError::PacketAcknowledgementNotFound { sequence: *seq }),
        }
        .map_err(ContextError::PacketError)
    }

    fn channel_counter(&self) -> Result<u64, ContextError> {
        Ok(self.ibc_store.lock().channel_ids_counter)
    }

    fn max_expected_time_per_block(&self) -> Duration {
        self.block_time
    }

    fn validate_message_signer(&self, _signer: &Signer) -> Result<(), ContextError> {
        Ok(())
    }

    fn get_client_validation_context(&self) -> &Self::V {
        self
    }
}

impl ExecutionContext for MockContext {
    type E = Self;

    fn get_client_execution_context(&mut self) -> &mut Self::E {
        self
    }

    fn increase_client_counter(&mut self) -> Result<(), ContextError> {
        self.ibc_store.lock().client_ids_counter += 1;
        Ok(())
    }

    fn store_connection(
        &mut self,
        connection_path: &ConnectionPath,
        connection_end: ConnectionEnd,
    ) -> Result<(), ContextError> {
        let connection_id = connection_path.0.clone();
        self.ibc_store
            .lock()
            .connections
            .insert(connection_id, connection_end);
        Ok(())
    }

    fn store_connection_to_client(
        &mut self,
        client_connection_path: &ClientConnectionPath,
        conn_id: ConnectionId,
    ) -> Result<(), ContextError> {
        let client_id = client_connection_path.0.clone();
        self.ibc_store
            .lock()
            .client_connections
            .insert(client_id, conn_id);
        Ok(())
    }

    fn increase_connection_counter(&mut self) -> Result<(), ContextError> {
        self.ibc_store.lock().connection_ids_counter += 1;
        Ok(())
    }

    fn store_packet_commitment(
        &mut self,
        commitment_path: &CommitmentPath,
        commitment: PacketCommitment,
    ) -> Result<(), ContextError> {
        self.ibc_store
            .lock()
            .packet_commitment
            .entry(commitment_path.port_id.clone())
            .or_default()
            .entry(commitment_path.channel_id.clone())
            .or_default()
            .insert(commitment_path.sequence, commitment);
        Ok(())
    }

    fn delete_packet_commitment(
        &mut self,
        commitment_path: &CommitmentPath,
    ) -> Result<(), ContextError> {
        self.ibc_store
            .lock()
            .packet_commitment
            .get_mut(&commitment_path.port_id)
            .and_then(|map| map.get_mut(&commitment_path.channel_id))
            .and_then(|map| map.remove(&commitment_path.sequence));
        Ok(())
    }

    fn store_packet_receipt(
        &mut self,
        path: &ReceiptPath,
        receipt: Receipt,
    ) -> Result<(), ContextError> {
        self.ibc_store
            .lock()
            .packet_receipt
            .entry(path.port_id.clone())
            .or_default()
            .entry(path.channel_id.clone())
            .or_default()
            .insert(path.sequence, receipt);
        Ok(())
    }

    fn store_packet_acknowledgement(
        &mut self,
        ack_path: &AckPath,
        ack_commitment: AcknowledgementCommitment,
    ) -> Result<(), ContextError> {
        let port_id = ack_path.port_id.clone();
        let channel_id = ack_path.channel_id.clone();
        let seq = ack_path.sequence;

        self.ibc_store
            .lock()
            .packet_acknowledgement
            .entry(port_id)
            .or_default()
            .entry(channel_id)
            .or_default()
            .insert(seq, ack_commitment);
        Ok(())
    }

    fn delete_packet_acknowledgement(&mut self, ack_path: &AckPath) -> Result<(), ContextError> {
        let port_id = ack_path.port_id.clone();
        let channel_id = ack_path.channel_id.clone();
        let sequence = ack_path.sequence;

        self.ibc_store
            .lock()
            .packet_acknowledgement
            .get_mut(&port_id)
            .and_then(|map| map.get_mut(&channel_id))
            .and_then(|map| map.remove(&sequence));
        Ok(())
    }

    fn store_channel(
        &mut self,
        channel_end_path: &ChannelEndPath,
        channel_end: ChannelEnd,
    ) -> Result<(), ContextError> {
        let port_id = channel_end_path.0.clone();
        let channel_id = channel_end_path.1.clone();

        self.ibc_store
            .lock()
            .channels
            .entry(port_id)
            .or_default()
            .insert(channel_id, channel_end);
        Ok(())
    }

    fn store_next_sequence_send(
        &mut self,
        seq_send_path: &SeqSendPath,
        seq: Sequence,
    ) -> Result<(), ContextError> {
        let port_id = seq_send_path.0.clone();
        let channel_id = seq_send_path.1.clone();

        self.ibc_store
            .lock()
            .next_sequence_send
            .entry(port_id)
            .or_default()
            .insert(channel_id, seq);
        Ok(())
    }

    fn store_next_sequence_recv(
        &mut self,
        seq_recv_path: &SeqRecvPath,
        seq: Sequence,
    ) -> Result<(), ContextError> {
        let port_id = seq_recv_path.0.clone();
        let channel_id = seq_recv_path.1.clone();

        self.ibc_store
            .lock()
            .next_sequence_recv
            .entry(port_id)
            .or_default()
            .insert(channel_id, seq);
        Ok(())
    }

    fn store_next_sequence_ack(
        &mut self,
        seq_ack_path: &SeqAckPath,
        seq: Sequence,
    ) -> Result<(), ContextError> {
        let port_id = seq_ack_path.0.clone();
        let channel_id = seq_ack_path.1.clone();

        self.ibc_store
            .lock()
            .next_sequence_ack
            .entry(port_id)
            .or_default()
            .insert(channel_id, seq);
        Ok(())
    }

    fn increase_channel_counter(&mut self) -> Result<(), ContextError> {
        self.ibc_store.lock().channel_ids_counter += 1;
        Ok(())
    }

    fn emit_ibc_event(&mut self, event: IbcEvent) -> Result<(), ContextError> {
        self.ibc_store.lock().events.push(event);
        Ok(())
    }

    fn log_message(&mut self, message: String) -> Result<(), ContextError> {
        self.ibc_store.lock().logs.push(message);
        Ok(())
    }
}
