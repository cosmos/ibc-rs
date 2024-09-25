use ibc::apps::transfer::context::{TokenTransferExecutionContext, TokenTransferValidationContext};
use ibc::apps::transfer::types::{Memo, PrefixedCoin};
use ibc::core::host::types::error::HostError;
use ibc::core::host::types::identifiers::{ChannelId, PortId};
use ibc::core::primitives::Signer;

use super::types::DummyTransferModule;

impl TokenTransferValidationContext for DummyTransferModule {
    type AccountId = Signer;

    fn get_port(&self) -> Result<PortId, HostError> {
        Ok(PortId::transfer())
    }

    fn can_send_coins(&self) -> Result<(), HostError> {
        Ok(())
    }

    fn can_receive_coins(&self) -> Result<(), HostError> {
        Ok(())
    }
    fn escrow_coins_validate(
        &self,
        _from_account: &Self::AccountId,
        _port_id: &PortId,
        _channel_id: &ChannelId,
        _coin: &PrefixedCoin,
        _memo: &Memo,
    ) -> Result<(), HostError> {
        Ok(())
    }

    fn unescrow_coins_validate(
        &self,
        _to_account: &Self::AccountId,
        _port_id: &PortId,
        _channel_id: &ChannelId,
        _coin: &PrefixedCoin,
    ) -> Result<(), HostError> {
        Ok(())
    }

    fn mint_coins_validate(
        &self,
        _account: &Self::AccountId,
        _coin: &PrefixedCoin,
    ) -> Result<(), HostError> {
        Ok(())
    }

    fn burn_coins_validate(
        &self,
        _account: &Self::AccountId,
        _coin: &PrefixedCoin,
        _memo: &Memo,
    ) -> Result<(), HostError> {
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
    ) -> Result<(), HostError> {
        Ok(())
    }

    fn unescrow_coins_execute(
        &mut self,
        _to_account: &Self::AccountId,
        _port_id: &PortId,
        _channel_id: &ChannelId,
        _coin: &PrefixedCoin,
    ) -> Result<(), HostError> {
        Ok(())
    }

    fn mint_coins_execute(
        &mut self,
        _account: &Self::AccountId,
        _coin: &PrefixedCoin,
    ) -> Result<(), HostError> {
        Ok(())
    }

    fn burn_coins_execute(
        &mut self,
        _account: &Self::AccountId,
        _coin: &PrefixedCoin,
        _memo: &Memo,
    ) -> Result<(), HostError> {
        Ok(())
    }
}
