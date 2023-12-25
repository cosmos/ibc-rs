//! Defines the required context traits for ICS-721 to interact with host
//! machine.

use crate::types::error::NftTransferError;
use crate::types::{ClassData, ClassId, ClassUri};
use crate::types::{TokenData, TokenId, TokenUri};
use ibc_core::host::types::identifiers::PortId;
use ibc_core::primitives::Signer;

/// Read-only methods required in NFT transfer validation context.
pub trait NftTransferValidationContext<N, C> {
    type AccountId: TryFrom<Signer>;

    /// get_port returns the portID for the transfer module.
    fn get_port(&self) -> Result<PortId, NftTransferError>;

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
    fn create_or_update_class(
        &self,
        class_id: &ClassId,
        class_uri: &ClassUri,
        class_data: &ClassData,
    ) -> Result<(), NftTransferError>;

    /// Creates a new NFT. The receiver becomes the owner.
    fn mint(
        &self,
        class_id: &ClassId,
        token_id: &TokenId,
        token_uri: &TokenUri,
        token_data: &TokenData,
        receiver: &Self::AccountId,
    ) -> Result<(), NftTransferError>;

    /// Transfers the NFT. If the token data is empty, it updates the token data.
    fn transfer(
        &self,
        class_id: &ClassId,
        token_id: &TokenId,
        receiver: &Self::AccountId,
        token_data: &TokenData,
    ) -> Result<(), NftTransferError>;

    /// Burns the NFT
    fn burn(&self, class_id: &ClassId, token_id: &TokenId) -> Result<(), NftTransferError>;
}
