//! Defines the required context traits for ICS-721 to interact with host
//! machine.

/// Read-only methods required in NFT transfer validation context.
pub trait NftTransferValidationContext {}

/// Read-write methods required in NFT transfer execution context.
pub trait NftTransferExecutionContext: NftTransferValidationContext {}
