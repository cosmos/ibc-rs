//! Defines the main context traits and IBC module callbacks

use ibc_app_transfer_types::error::TokenTransferError;
use ibc_app_transfer_types::{PrefixedCoin, PrefixedDenom};
use ibc_core::host::types::identifiers::{ChannelId, PortId};
use ibc_core::primitives::prelude::*;
use ibc_core::primitives::Signer;

/// Methods required in token transfer validation, to be implemented by the host
pub trait TokenTransferValidationContext<D> {
    type AccountId: TryFrom<Signer>;

    /// get_port returns the portID for the transfer module.
    fn get_port(&self) -> Result<PortId, TokenTransferError>;

    /// Returns Ok() if the host chain supports sending coins.
    fn can_send_coins(&self) -> Result<(), TokenTransferError>;

    /// Returns Ok() if the host chain supports receiving coins.
    fn can_receive_coins(&self) -> Result<(), TokenTransferError>;

    /// Validates that the tokens can be escrowed successfully
    fn escrow_coins_validate(
        &self,
        port_id: &PortId,
        channel_id: &ChannelId,
        from_account: &Self::AccountId,
        coin: &PrefixedCoin,
        extra: &D,
    ) -> Result<(), TokenTransferError>;

    /// Validates that the tokens can be unescrowed successfully
    fn unescrow_coins_validate(
        &self,
        port_id: &PortId,
        channel_id: &ChannelId,
        to_account: &Self::AccountId,
        coin: &PrefixedCoin,
    ) -> Result<(), TokenTransferError>;

    /// Validates the receiver account and the coin input
    fn mint_coins_validate(
        &self,
        account: &Self::AccountId,
        coin: &PrefixedCoin,
    ) -> Result<(), TokenTransferError>;

    /// Validates the sender account and the coin input
    fn burn_coins_validate(
        &self,
        account: &Self::AccountId,
        coin: &PrefixedCoin,
    ) -> Result<(), TokenTransferError>;

    /// Returns a hash of the prefixed denom.
    /// Implement only if the host chain supports hashed denominations.
    fn denom_hash_string(&self, _denom: &PrefixedDenom) -> Option<String> {
        None
    }
}

/// Methods required in token transfer execution, to be implemented by the host
pub trait TokenTransferExecutionContext<D>: TokenTransferValidationContext<D> {
    /// Escrows the tokens
    fn escrow_coins_execute(
        &mut self,
        port_id: &PortId,
        channel_id: &ChannelId,
        from_account: &Self::AccountId,
        coin: &PrefixedCoin,
        extra: &D,
    ) -> Result<(), TokenTransferError>;

    /// Unescrows the tokens
    fn unescrow_coins_execute(
        &mut self,
        port_id: &PortId,
        channel_id: &ChannelId,
        to_account: &Self::AccountId,
        coin: &PrefixedCoin,
    ) -> Result<(), TokenTransferError>;

    /// This function to enable minting ibc tokens to a user account
    fn mint_coins_execute(
        &mut self,
        account: &Self::AccountId,
        coin: &PrefixedCoin,
    ) -> Result<(), TokenTransferError>;

    /// This function should enable burning of minted tokens in a user account
    fn burn_coins_execute(
        &mut self,
        account: &Self::AccountId,
        coin: &PrefixedCoin,
    ) -> Result<(), TokenTransferError>;
}
