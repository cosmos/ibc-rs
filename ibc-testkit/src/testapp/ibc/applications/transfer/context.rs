use ibc::apps::transfer::context::{TokenTransferExecutionContext, TokenTransferValidationContext};
use ibc::apps::transfer::types::error::TokenTransferError;
use ibc::apps::transfer::types::PrefixedCoin;
use ibc::core::host::types::identifiers::{ChannelId, PortId};
use ibc::core::primitives::Signer;
use ibc::cosmos_host::utils::cosmos_adr028_escrow_address;
use subtle_encoding::bech32;

use super::types::DummyTransferModule;

impl TokenTransferValidationContext for DummyTransferModule {
    type AccountId = Signer;

    fn get_port(&self) -> Result<PortId, TokenTransferError> {
        Ok(PortId::transfer())
    }

    fn get_escrow_account(
        &self,
        port_id: &PortId,
        channel_id: &ChannelId,
        _coin: &PrefixedCoin,
    ) -> Result<Self::AccountId, TokenTransferError> {
        let addr = cosmos_adr028_escrow_address(port_id, channel_id);
        Ok(bech32::encode("cosmos", addr).into())
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
