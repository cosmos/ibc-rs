use ibc::apps::transfer::context::{TokenTransferExecutionContext, TokenTransferValidationContext};
use ibc::apps::transfer::types::error::TokenTransferError;
use ibc::apps::transfer::types::{Memo, PrefixedCoin};
use ibc::core::host::types::identifiers::{ChannelId, PortId};
use ibc::core::primitives::Signer;

use super::types::DummyTransferModule;

impl TokenTransferValidationContext for DummyTransferModule {
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
    fn escrow_coins_validate(
        &self,
        _from_account: &Self::AccountId,
        _port_id: &PortId,
        _channel_id: &ChannelId,
        _coin: &PrefixedCoin,
        _memo: &Memo,
    ) -> Result<(), TokenTransferError> {
        Ok(())
    }

    fn unescrow_coins_validate(
        &self,
        _to_account: &Self::AccountId,
        _port_id: &PortId,
        _channel_id: &ChannelId,
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
        _memo: &Memo,
    ) -> Result<(), TokenTransferError> {
        Ok(())
    }
}

impl TokenTransferExecutionContext for DummyTransferModule {
    fn escrow_coins_execute(
        &mut self,
        _from_account: &Self::AccountId,
        _port_id: &PortId,
        _channel_id: &ChannelId,
        _coin: &PrefixedCoin,
        _memo: &Memo,
    ) -> Result<(), TokenTransferError> {
        Ok(())
    }

    fn unescrow_coins_execute(
        &mut self,
        _to_account: &Self::AccountId,
        _port_id: &PortId,
        _channel_id: &ChannelId,
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
        _memo: &Memo,
    ) -> Result<(), TokenTransferError> {
        Ok(())
    }
}
