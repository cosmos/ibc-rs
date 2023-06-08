use alloc::sync::Arc;
use parking_lot::Mutex;
use subtle_encoding::bech32;
use tendermint::{block, consensus, evidence, public_key::Algorithm};

use crate::applications::transfer::context::{
    cosmos_adr028_escrow_address, TokenTransferExecutionContext, TokenTransferValidationContext,
};
use crate::applications::transfer::MODULE_ID_STR;
use crate::applications::transfer::{error::TokenTransferError, PrefixedCoin};
use crate::core::events::IbcEvent;
use crate::core::ics02_client::client_state::ClientState;
use crate::core::ics02_client::consensus_state::ConsensusState;
use crate::core::ics02_client::error::ClientError;
use crate::core::ics02_client::height::Height;
use crate::core::ics03_connection::connection::ConnectionEnd;
use crate::core::ics03_connection::error::ConnectionError;
use crate::core::ics04_channel::channel::{ChannelEnd, Counterparty, Order};
use crate::core::ics04_channel::commitment::PacketCommitment;
use crate::core::ics04_channel::context::{
    SendPacketExecutionContext, SendPacketValidationContext,
};
use crate::core::ics04_channel::error::{ChannelError, PacketError, PortError};
use crate::core::ics04_channel::packet::{Acknowledgement, Packet, Sequence};
use crate::core::ics04_channel::Version;
use crate::core::ics24_host::identifier::{ChannelId, ClientId, ConnectionId, PortId};
use crate::core::ics24_host::path::{
    ChannelEndPath, ClientConsensusStatePath, CommitmentPath, PortPath, SeqSendPath,
};
use crate::core::module::{
    ExecutionModule, ModuleContext, ModuleExtras, ModuleId, ValidationModule,
};
use crate::core::ContextError;
use crate::mock::context::MockIbcStore;
use crate::prelude::*;
use crate::signer::Signer;

// Needed in mocks.
pub fn default_consensus_params() -> consensus::Params {
    consensus::Params {
        block: block::Size {
            max_bytes: 22020096,
            max_gas: -1,
            time_iota_ms: 1000,
        },
        evidence: evidence::Params {
            max_age_num_blocks: 100000,
            max_age_duration: evidence::Duration(core::time::Duration::new(48 * 3600, 0)),
            max_bytes: 0,
        },
        validator: consensus::params::ValidatorParams {
            pub_key_types: vec![Algorithm::Ed25519],
        },
        version: Some(consensus::params::VersionParams::default()),
    }
}

pub fn get_dummy_proof() -> Vec<u8> {
    "Y29uc2Vuc3VzU3RhdGUvaWJjb25lY2xpZW50LzIy"
        .as_bytes()
        .to_vec()
}

pub fn get_dummy_account_id() -> Signer {
    "0CDA3F47EF3C4906693B170EF650EB968C5F4B2C"
        .to_string()
        .into()
}

pub fn get_dummy_bech32_account() -> String {
    "cosmos1wxeyh7zgn4tctjzs0vtqpc6p5cxq5t2muzl7ng".to_string()
}

#[derive(Debug, Clone)]
pub struct DummyTransferModule {
    pub module_id: ModuleId,
    pub ibc_store: Arc<Mutex<MockIbcStore>>,
}

impl DummyTransferModule {
    pub fn new(ibc_store: Arc<Mutex<MockIbcStore>>) -> Self {
        Self {
            module_id: MODULE_ID_STR.parse().unwrap(),
            ibc_store,
        }
    }
}

impl Default for DummyTransferModule {
    fn default() -> Self {
        Self::new(Arc::new(Mutex::new(MockIbcStore::default())))
    }
}

impl ModuleContext for DummyTransferModule {
    fn module_id(&self) -> ModuleId {
        self.module_id.clone()
    }

    fn get_owned_ports(&self) -> Vec<PortId> {
        self.ibc_store
            .lock()
            .port_to_module
            .keys()
            .cloned()
            .collect()
    }

    fn bind_port(&mut self, port_path: PortPath, port_owner: ModuleId) -> Result<(), PortError> {
        let mut ibc_store = self.ibc_store.lock();
        let fetched_owner = ibc_store.port_to_module.get(&port_path.0);
        if let Some(owner) = fetched_owner {
            return Err(PortError::PortAlreadyBound {
                port_id: port_path.clone().0,
                port_id_owner: owner.to_string(),
            });
        }
        ibc_store.port_to_module.insert(port_path.0, port_owner);
        Ok(())
    }

    fn release_port(&mut self, port_path: PortPath) -> Result<(), PortError> {
        self.ibc_store.lock().port_to_module.remove(&port_path.0);
        Ok(())
    }
}

impl ValidationModule for DummyTransferModule {
    fn on_chan_open_init_validate(
        &self,
        _order: Order,
        _connection_hops: &[ConnectionId],
        _port_id: &PortId,
        _channel_id: &ChannelId,
        _counterparty: &Counterparty,
        version: &Version,
    ) -> Result<Version, ChannelError> {
        Ok(version.clone())
    }

    fn on_chan_open_try_validate(
        &self,
        _order: Order,
        _connection_hops: &[ConnectionId],
        _port_id: &PortId,
        _channel_id: &ChannelId,
        _counterparty: &Counterparty,
        counterparty_version: &Version,
    ) -> Result<Version, ChannelError> {
        Ok(counterparty_version.clone())
    }

    fn on_timeout_packet_validate(
        &self,
        _packet: &Packet,
        _relayer: &Signer,
    ) -> Result<(), PacketError> {
        Ok(())
    }

    fn on_acknowledgement_packet_validate(
        &self,
        _packet: &Packet,
        _acknowledgement: &Acknowledgement,
        _relayer: &Signer,
    ) -> Result<(), PacketError> {
        Ok(())
    }
}

impl ExecutionModule for DummyTransferModule {
    fn on_chan_open_init_execute(
        &mut self,
        _order: Order,
        _connection_hops: &[ConnectionId],
        _port_id: &PortId,
        _channel_id: &ChannelId,
        _counterparty: &Counterparty,
        version: &Version,
    ) -> Result<(ModuleExtras, Version), ChannelError> {
        Ok((ModuleExtras::empty(), version.clone()))
    }

    fn on_chan_open_try_execute(
        &mut self,
        _order: Order,
        _connection_hops: &[ConnectionId],
        _port_id: &PortId,
        _channel_id: &ChannelId,
        _counterparty: &Counterparty,
        counterparty_version: &Version,
    ) -> Result<(ModuleExtras, Version), ChannelError> {
        Ok((ModuleExtras::empty(), counterparty_version.clone()))
    }

    fn on_recv_packet_execute(
        &mut self,
        _packet: &Packet,
        _relayer: &Signer,
    ) -> (ModuleExtras, Acknowledgement) {
        (
            ModuleExtras::empty(),
            Acknowledgement::try_from(vec![1u8]).unwrap(),
        )
    }

    fn on_timeout_packet_execute(
        &mut self,
        _packet: &Packet,
        _relayer: &Signer,
    ) -> (ModuleExtras, Result<(), PacketError>) {
        (ModuleExtras::empty(), Ok(()))
    }

    fn on_acknowledgement_packet_execute(
        &mut self,
        _packet: &Packet,
        _acknowledgement: &Acknowledgement,
        _relayer: &Signer,
    ) -> (ModuleExtras, Result<(), PacketError>) {
        (ModuleExtras::empty(), Ok(()))
    }
}

impl TokenTransferValidationContext for DummyTransferModule {
    type AccountId = Signer;

    fn get_escrow_account(
        &self,
        port_id: &PortId,
        channel_id: &ChannelId,
    ) -> Result<Self::AccountId, TokenTransferError> {
        let addr = cosmos_adr028_escrow_address(port_id, channel_id);
        Ok(bech32::encode("cosmos", addr).parse().unwrap())
    }

    fn can_send_coins(&self) -> Result<(), TokenTransferError> {
        Ok(())
    }

    fn can_receive_coins(&self) -> Result<(), TokenTransferError> {
        Ok(())
    }

    fn send_coins_validate(
        &self,
        _from_account: &Self::AccountId,
        _to_account: &Self::AccountId,
        _coin: &PrefixedCoin,
    ) -> Result<(), TokenTransferError> {
        Ok(())
    }

    fn mint_coins_validate(
        &self,
        _account: &Self::AccountId,
        _coin: &PrefixedCoin,
    ) -> Result<(), TokenTransferError> {
        Ok(())
    }

    fn burn_coins_validate(
        &self,
        _account: &Self::AccountId,
        _coin: &PrefixedCoin,
    ) -> Result<(), TokenTransferError> {
        Ok(())
    }
}

impl TokenTransferExecutionContext for DummyTransferModule {
    fn send_coins_execute(
        &mut self,
        _from_account: &Self::AccountId,
        _to_account: &Self::AccountId,
        _coin: &PrefixedCoin,
    ) -> Result<(), TokenTransferError> {
        Ok(())
    }

    fn mint_coins_execute(
        &mut self,
        _account: &Self::AccountId,
        _coin: &PrefixedCoin,
    ) -> Result<(), TokenTransferError> {
        Ok(())
    }

    fn burn_coins_execute(
        &mut self,
        _account: &Self::AccountId,
        _coin: &PrefixedCoin,
    ) -> Result<(), TokenTransferError> {
        Ok(())
    }
}

impl SendPacketValidationContext for DummyTransferModule {
    fn channel_end(&self, chan_end_path: &ChannelEndPath) -> Result<ChannelEnd, ContextError> {
        match self
            .ibc_store
            .lock()
            .channels
            .get(&chan_end_path.0)
            .and_then(|map| map.get(&chan_end_path.1))
        {
            Some(channel_end) => Ok(channel_end.clone()),
            None => Err(PacketError::ChannelNotFound {
                port_id: chan_end_path.0.clone(),
                channel_id: chan_end_path.1.clone(),
            }
            .into()),
        }
    }

    fn connection_end(&self, cid: &ConnectionId) -> Result<ConnectionEnd, ContextError> {
        match self.ibc_store.lock().connections.get(cid) {
            Some(connection_end) => Ok(connection_end.clone()),
            None => Err(ConnectionError::ConnectionNotFound {
                connection_id: cid.clone(),
            }),
        }
        .map_err(PacketError::Connection)
        .map_err(ContextError::PacketError)
    }

    fn client_state(&self, client_id: &ClientId) -> Result<Box<dyn ClientState>, ContextError> {
        match self.ibc_store.lock().clients.get(client_id) {
            Some(client_record) => {
                client_record
                    .client_state
                    .clone()
                    .ok_or_else(|| ClientError::ClientStateNotFound {
                        client_id: client_id.clone(),
                    })
            }
            None => Err(ClientError::ClientStateNotFound {
                client_id: client_id.clone(),
            }),
        }
        .map_err(|e| PacketError::Connection(ConnectionError::Client(e)))
        .map_err(ContextError::PacketError)
    }

    fn client_consensus_state(
        &self,
        client_cons_state_path: &ClientConsensusStatePath,
    ) -> Result<Box<dyn ConsensusState>, ContextError> {
        let height =
            Height::new(client_cons_state_path.epoch, client_cons_state_path.height).unwrap();
        match self
            .ibc_store
            .lock()
            .clients
            .get(&client_cons_state_path.client_id)
        {
            Some(client_record) => match client_record.consensus_states.get(&height) {
                Some(consensus_state) => Ok(consensus_state.clone()),
                None => Err(ClientError::ConsensusStateNotFound {
                    client_id: client_cons_state_path.client_id.clone(),
                    height,
                }),
            },
            None => Err(ClientError::ConsensusStateNotFound {
                client_id: client_cons_state_path.client_id.clone(),
                height,
            }),
        }
        .map_err(|e| PacketError::Connection(ConnectionError::Client(e)))
        .map_err(ContextError::PacketError)
    }

    fn get_next_sequence_send(
        &self,
        seq_send_path: &SeqSendPath,
    ) -> Result<Sequence, ContextError> {
        match self
            .ibc_store
            .lock()
            .next_sequence_send
            .get(&seq_send_path.0)
            .and_then(|map| map.get(&seq_send_path.1))
        {
            Some(sequence) => Ok(*sequence),
            None => Err(PacketError::MissingNextSendSeq {
                port_id: seq_send_path.0.clone(),
                channel_id: seq_send_path.1.clone(),
            }
            .into()),
        }
    }
}

impl SendPacketExecutionContext for DummyTransferModule {
    fn store_packet_commitment(
        &mut self,
        commitment_path: &CommitmentPath,
        commitment: PacketCommitment,
    ) -> Result<(), ContextError> {
        let port_id = commitment_path.port_id.clone();
        let channel_id = commitment_path.channel_id.clone();
        let seq = commitment_path.sequence;

        self.ibc_store
            .lock()
            .packet_commitment
            .entry(port_id)
            .or_default()
            .entry(channel_id)
            .or_default()
            .insert(seq, commitment);
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

    fn emit_ibc_event(&mut self, _event: IbcEvent) {}

    fn log_message(&mut self, _message: String) {}
}
