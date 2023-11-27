use ibc::apps::transfer::context::{TokenTransferExecutionContext, TokenTransferValidationContext};
use ibc::apps::transfer::types::error::TokenTransferError;
use ibc::apps::transfer::types::PrefixedCoin;
use ibc::core::host::types::identifiers::{ChannelId, PortId};
use ibc::core::primitives::Signer;

use super::types::DummyTransferModule;

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
