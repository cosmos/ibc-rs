//! Defines the required context traits for ICS-721 to interact with host
//! machine.
use ibc_core::host::types::identifiers::{ChannelId, PortId};
use ibc_core::primitives::prelude::*;
use ibc_core::primitives::Signer;

use crate::types::error::NftTransferError;
use crate::types::{
    ClassData, ClassId, ClassUri, Memo, PrefixedClassId, TokenData, TokenId, TokenUri,
};

/// Read-only methods required in NFT transfer validation context.
pub trait NftTransferValidationContext<N, C> {
    type AccountId: TryFrom<Signer>;

    /// get_port returns the portID for the transfer module.
    fn get_port(&self) -> Result<PortId, NftTransferError>;

    /// Returns Ok() if the host chain supports sending NFTs.
    fn can_send_nft(&self) -> Result<(), NftTransferError>;

    /// Returns Ok() if the host chain supports receiving NFTs.
    fn can_receive_nft(&self) -> Result<(), NftTransferError>;

    /// Validates that the NFT can be created or updated successfully.
    fn create_or_update_class_validate(
        &self,
        class_id: &ClassId,
        class_uri: &ClassUri,
        class_data: &ClassData,
    ) -> Result<(), NftTransferError>;

    /// Validates that the tokens can be escrowed successfully.
    ///
    /// `memo` field allows to incorporate additional contextual details in the
    /// escrow validation.
    fn escrow_nft_validate(
        &self,
        from_account: &Self::AccountId,
        port_id: &PortId,
        channel_id: &ChannelId,
        class_id: &PrefixedClassId,
        token_id: &TokenId,
        memo: &Memo,
    ) -> Result<(), NftTransferError>;

    /// Validates that the NFT can be unescrowed successfully.
    fn unescrow_nft_validate(
        &self,
        to_account: &Self::AccountId,
        port_id: &PortId,
        channel_id: &ChannelId,
        class_id: &PrefixedClassId,
        token_id: &TokenId,
    ) -> Result<(), NftTransferError>;

    /// Validates the receiver account and the NFT input
    fn mint_nft_validate(
        &self,
        account: &Self::AccountId,
        class_id: &PrefixedClassId,
        token_id: &TokenId,
        token_uri: &TokenUri,
        token_data: &TokenData,
    ) -> Result<(), NftTransferError>;

    /// Validates the sender account and the coin input before burning.
    ///
    /// `memo` field allows to incorporate additional contextual details in the
    /// burn validation.
    fn burn_nft_validate(
        &self,
        account: &Self::AccountId,
        class_id: &PrefixedClassId,
        token_id: &TokenId,
        memo: &Memo,
    ) -> Result<(), NftTransferError>;

    /// Returns a hash of the prefixed class ID.
    /// Implement only if the host chain supports hashed class ID.
    fn class_hash_string(&self, _class_id: &PrefixedClassId) -> Option<String> {
        None
    }

    /// Returns the current owner of the NFT
    fn get_owner(
        &self,
        class_id: &ClassId,
        token_id: &TokenId,
    ) -> Result<Self::AccountId, NftTransferError>;

    /// Returns the NFT
    fn get_nft(
        &self,
        class_id: &ClassId,
        token_id: &TokenId,
    ) -> Result<Option<N>, NftTransferError>;

    /// Returns the NFT class
    fn get_nft_class(class_id: &ClassId) -> Result<Option<C>, NftTransferError>;
}

/// Read-write methods required in NFT transfer execution context.
pub trait NftTransferExecutionContext<N, C>: NftTransferValidationContext<N, C> {
    /// Creates a new NFT Class identified by classId. If the class ID already exists, it updates the class metadata.
    fn create_or_update_class_execute(
        &self,
        class_id: &ClassId,
        class_uri: &ClassUri,
        class_data: &ClassData,
    ) -> Result<(), NftTransferError>;

    /// Executes the escrow of the NFT in a user account.
    ///
    /// `memo` field allows to incorporate additional contextual details in the
    /// escrow execution.
    fn escrow_nft_execute(
        &mut self,
        from_account: &Self::AccountId,
        port_id: &PortId,
        channel_id: &ChannelId,
        class_id: &PrefixedClassId,
        token_id: &TokenId,
        memo: &Memo,
    ) -> Result<(), NftTransferError>;

    /// Executes the unescrow of the NFT in a user account.
    fn unescrow_nft_execute(
        &mut self,
        to_account: &Self::AccountId,
        port_id: &PortId,
        channel_id: &ChannelId,
        class_id: &PrefixedClassId,
        token_id: &TokenId,
    ) -> Result<(), NftTransferError>;

    /// Executes minting of the NFT in a user account.
    fn mint_nft_execute(
        &mut self,
        account: &Self::AccountId,
        class_id: &PrefixedClassId,
        token_id: &TokenId,
        token_uri: &TokenUri,
        token_data: &TokenData,
    ) -> Result<(), NftTransferError>;

    /// Executes burning of the NFT in a user account.
    ///
    /// `memo` field allows to incorporate additional contextual details in the
    /// burn execution.
    fn burn_nft_execute(
        &mut self,
        account: &Self::AccountId,
        class_id: &PrefixedClassId,
        token_id: &TokenId,
        memo: &Memo,
    ) -> Result<(), NftTransferError>;
}
