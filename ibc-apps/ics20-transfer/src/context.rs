//! Defines the main context traits and IBC module callbacks

use ibc_app_transfer_types::error::TokenTransferError;
use ibc_app_transfer_types::{PrefixedCoin, PrefixedDenom};
use ibc_core::host::types::identifiers::{ChannelId, PortId};
use ibc_core::primitives::prelude::*;
use ibc_core::primitives::Signer;

/// Methods required in token transfer validation, to be implemented by the host
pub trait TokenTransferValidationContext {
    type AccountId: TryFrom<Signer>;

    /// get_port returns the portID for the transfer module.
    fn get_port(&self) -> Result<PortId, TokenTransferError>;

    /// Returns the escrow account id for a port and channel combination
    fn get_escrow_account(
        &self,
        port_id: &PortId,
        channel_id: &ChannelId,
        coin: &PrefixedCoin,
    ) -> Result<Self::AccountId, TokenTransferError>;

    /// Returns Ok() if the host chain supports sending coins.
    fn can_send_coins(&self) -> Result<(), TokenTransferError>;

    /// Returns Ok() if the host chain supports receiving coins.
    fn can_receive_coins(&self) -> Result<(), TokenTransferError>;

    /// Validates the sender and receiver accounts and the coin inputs
    fn send_coins_validate(
        &self,
        from_account: &Self::AccountId,
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
pub trait TokenTransferExecutionContext: TokenTransferValidationContext {
    /// This function should enable sending ibc fungible tokens from one account to another
    fn send_coins_execute(
        &mut self,
        from_account: &Self::AccountId,
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
