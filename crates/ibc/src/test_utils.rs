use crate::applications::transfer::context::{
    TokenTransferExecutionContext, TokenTransferValidationContext,
};
use crate::applications::transfer::error::TokenTransferError;
use crate::applications::transfer::PrefixedCoin;
use crate::core::ics04_channel::acknowledgement::Acknowledgement;
use crate::core::ics04_channel::channel::{Counterparty, Order};
use crate::core::ics04_channel::error::{ChannelError, PacketError};
use crate::core::ics04_channel::packet::Packet;
use crate::core::ics04_channel::Version;
use crate::core::ics24_host::identifier::{ChannelId, ConnectionId, PortId};
use crate::core::router::{Module, ModuleExtras};
use crate::prelude::*;
use crate::signer::Signer;

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

pub fn get_dummy_transfer_module() -> DummyTransferModule {
    DummyTransferModule
}
#[derive(Debug)]
pub struct DummyTransferModule;

impl DummyTransferModule {
    pub fn new() -> Self {
        Self
    }
}

impl Default for DummyTransferModule {
    fn default() -> Self {
        Self::new()
    }
}

impl Module for DummyTransferModule {
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
            Acknowledgement::try_from(vec![1u8]).expect("Never fails"),
        )
    }

    fn on_timeout_packet_validate(
        &self,
        _packet: &Packet,
        _relayer: &Signer,
    ) -> Result<(), PacketError> {
        Ok(())
    }

    fn on_timeout_packet_execute(
        &mut self,
        _packet: &Packet,
        _relayer: &Signer,
    ) -> (ModuleExtras, Result<(), PacketError>) {
        (ModuleExtras::empty(), Ok(()))
    }

    fn on_acknowledgement_packet_validate(
        &self,
        _packet: &Packet,
        _acknowledgement: &Acknowledgement,
        _relayer: &Signer,
    ) -> Result<(), PacketError> {
        Ok(())
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

impl<D> TokenTransferValidationContext<D> for DummyTransferModule {
    type AccountId = Signer;

    fn get_port(&self) -> Result<PortId, TokenTransferError> {
        Ok(PortId::transfer())
    }

    fn can_send_coins(&self) -> Result<(), TokenTransferError> {
        Ok(())
    }

    fn can_receive_coins(&self) -> Result<(), TokenTransferError> {
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

    fn escrow_coins_validate(
        &self,
        _port_id: &PortId,
        _channel_id: &ChannelId,
        _from_account: &Self::AccountId,
        _coin: &PrefixedCoin,
        _extra: &D,
    ) -> Result<(), TokenTransferError> {
        Ok(())
    }

    fn unescrow_coins_validate(
        &self,
        _port_id: &PortId,
        _channel_id: &ChannelId,
        _to_account: &Self::AccountId,
        _coin: &PrefixedCoin,
    ) -> Result<(), TokenTransferError> {
        Ok(())
    }
}

impl<D> TokenTransferExecutionContext<D> for DummyTransferModule {
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

    fn escrow_coins_execute(
        &mut self,
        _port_id: &PortId,
        _channel_id: &ChannelId,
        _from_account: &Self::AccountId,
        _coin: &PrefixedCoin,
        _extra: &D,
    ) -> Result<(), TokenTransferError> {
        Ok(())
    }

    fn unescrow_coins_execute(
        &mut self,
        _port_id: &PortId,
        _channel_id: &ChannelId,
        _to_account: &Self::AccountId,
        _coin: &PrefixedCoin,
    ) -> Result<(), TokenTransferError> {
        Ok(())
    }
}
