//! Defines the main context traits and IBC module callbacks

use ibc_app_transfer_types::error::TokenTransferError;
use ibc_app_transfer_types::{Memo, PrefixedCoin, PrefixedDenom};
use ibc_core::host::types::identifiers::{ChannelId, PortId};
use ibc_core::primitives::prelude::*;
use ibc_core::primitives::Signer;

/// Methods required in token transfer validation, to be implemented by the host
pub trait TokenTransferValidationContext {
    type AccountId: TryFrom<Signer>;

    /// get_port returns the portID for the transfer module.
    fn get_port(&self) -> Result<PortId, TokenTransferError>;

    /// Returns Ok() if the host chain supports sending coins.
    fn can_send_coins(&self) -> Result<(), TokenTransferError>;

    /// Returns Ok() if the host chain supports receiving coins.
    fn can_receive_coins(&self) -> Result<(), TokenTransferError>;

    /// Validates that the tokens can be escrowed successfully.
    ///
    /// `memo` field allows to incorporate additional contextual details in the
    /// escrow validation.
    fn escrow_coins_validate(
        &self,
        from_account: &Self::AccountId,
        port_id: &PortId,
        channel_id: &ChannelId,
        coin: &PrefixedCoin,
        memo: &Memo,
    ) -> Result<(), TokenTransferError>;

    /// Validates that the tokens can be unescrowed successfully.
    fn unescrow_coins_validate(
        &self,
        to_account: &Self::AccountId,
        port_id: &PortId,
        channel_id: &ChannelId,
        coin: &PrefixedCoin,
    ) -> Result<(), TokenTransferError>;

    /// Validates the receiver account and the coin input
    fn mint_coins_validate(
        &self,
        account: &Self::AccountId,
        coin: &PrefixedCoin,
    ) -> Result<(), TokenTransferError>;

    /// Validates the sender account and the coin input before burning.
    ///
    /// `memo` field allows to incorporate additional contextual details in the
    /// burn validation.
    fn burn_coins_validate(
        &self,
        account: &Self::AccountId,
        coin: &PrefixedCoin,
        memo: &Memo,
    ) -> Result<(), TokenTransferError>;

    /// Returns a hash of the prefixed denom.
    /// Implement only if the host chain supports hashed denominations.
    fn denom_hash_string(&self, _denom: &PrefixedDenom) -> Option<String> {
        None
    }
}

/// Methods required in token transfer execution, to be implemented by the host.
pub trait TokenTransferExecutionContext: TokenTransferValidationContext {
    /// Executes the escrow of the tokens in a user account.
    ///
    /// `memo` field allows to incorporate additional contextual details in the
    /// escrow execution.
    fn escrow_coins_execute(
        &mut self,
        from_account: &Self::AccountId,
        port_id: &PortId,
        channel_id: &ChannelId,
        coin: &PrefixedCoin,
        memo: &Memo,
    ) -> Result<(), TokenTransferError>;

    /// Executes the unescrow of the tokens in a user account.
    fn unescrow_coins_execute(
        &mut self,
        to_account: &Self::AccountId,
        port_id: &PortId,
        channel_id: &ChannelId,
        coin: &PrefixedCoin,
    ) -> Result<(), TokenTransferError>;

    /// Executes minting of the tokens in a user account.
    fn mint_coins_execute(
        &mut self,
        account: &Self::AccountId,
        coin: &PrefixedCoin,
    ) -> Result<(), TokenTransferError>;

    /// Executes burning of the tokens in a user account.
    ///
    /// `memo` field allows to incorporate additional contextual details in the
    /// burn execution.
    fn burn_coins_execute(
        &mut self,
        account: &Self::AccountId,
        coin: &PrefixedCoin,
        memo: &Memo,
    ) -> Result<(), TokenTransferError>;
}
