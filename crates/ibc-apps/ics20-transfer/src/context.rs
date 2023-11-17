//! Defines the main context traits and IBC module callbacks

use ibc_app_transfer_types::error::TokenTransferError;
use ibc_app_transfer_types::{PrefixedCoin, PrefixedDenom, VERSION};
use ibc_core::host::identifiers::{ChannelId, PortId};
use ibc_core::primitives::prelude::*;
use ibc_core::primitives::Signer;
use sha2::{Digest, Sha256};

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

/// Helper function to generate an escrow address for a given port and channel
/// ids according to the format specified in the Cosmos SDK
/// [`ADR-028`](https://github.com/cosmos/cosmos-sdk/blob/master/docs/architecture/adr-028-public-key-addresses.md)
pub fn cosmos_adr028_escrow_address(port_id: &PortId, channel_id: &ChannelId) -> Vec<u8> {
    let contents = format!("{port_id}/{channel_id}");

    let mut hasher = Sha256::new();
    hasher.update(VERSION.as_bytes());
    hasher.update([0]);
    hasher.update(contents.as_bytes());

    let mut hash = hasher.finalize().to_vec();
    hash.truncate(20);
    hash
}

#[cfg(test)]
mod tests {
    use subtle_encoding::bech32;

    use super::*;
    use crate::context::cosmos_adr028_escrow_address;

    #[test]
    fn test_cosmos_escrow_address() {
        fn assert_eq_escrow_address(port_id: &str, channel_id: &str, address: &str) {
            let port_id = port_id.parse().unwrap();
            let channel_id = channel_id.parse().unwrap();
            let gen_address = {
                let addr = cosmos_adr028_escrow_address(&port_id, &channel_id);
                bech32::encode("cosmos", addr)
            };
            assert_eq!(gen_address, address.to_owned())
        }

        // addresses obtained using `gaiad query ibc-transfer escrow-address [port-id] [channel-id]`
        assert_eq_escrow_address(
            "transfer",
            "channel-141",
            "cosmos1x54ltnyg88k0ejmk8ytwrhd3ltm84xehrnlslf",
        );
        assert_eq_escrow_address(
            "transfer",
            "channel-207",
            "cosmos1ju6tlfclulxumtt2kglvnxduj5d93a64r5czge",
        );
        assert_eq_escrow_address(
            "transfer",
            "channel-187",
            "cosmos177x69sver58mcfs74x6dg0tv6ls4s3xmmcaw53",
        );
    }
}
