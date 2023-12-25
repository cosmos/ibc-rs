//! Defines Non-Fungible Token Transfer (ICS-721) token types.
use core::convert::Infallible;
use core::fmt::{self, Display};
use core::str::FromStr;

use http::Uri;
use ibc_core::primitives::prelude::*;

use crate::error::NftTransferError;

/// Token ID for an NFT
#[cfg_attr(
    feature = "parity-scale-codec",
    derive(
        parity_scale_codec::Encode,
        parity_scale_codec::Decode,
        scale_info::TypeInfo
    )
)]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TokenId(String);

impl AsRef<str> for TokenId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Display for TokenId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for TokenId {
    type Err = Infallible;

    fn from_str(token_id: &str) -> Result<Self, Infallible> {
        Ok(Self(token_id.to_string()))
    }
}

/// Token URI for an NFT
#[cfg_attr(
    feature = "parity-scale-codec",
    derive(
        parity_scale_codec::Encode,
        parity_scale_codec::Decode,
        scale_info::TypeInfo
    )
)]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TokenUri(String);

impl AsRef<str> for TokenUri {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Display for TokenUri {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for TokenUri {
    type Err = NftTransferError;

    fn from_str(token_uri: &str) -> Result<Self, Self::Err> {
        match Uri::from_str(token_uri) {
            Ok(_) => Ok(Self(token_uri.to_string())),
            Err(e) => Err(NftTransferError::InvalidUri {
                uri: token_uri.to_string(),
                error: e,
            }),
        }
    }
}

/// Token data for an NFT
#[cfg_attr(
    feature = "parity-scale-codec",
    derive(
        parity_scale_codec::Encode,
        parity_scale_codec::Decode,
        scale_info::TypeInfo
    )
)]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TokenData(String);

impl AsRef<str> for TokenData {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Display for TokenData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for TokenData {
    type Err = NftTransferError;

    fn from_str(token_data: &str) -> Result<Self, Self::Err> {
        // TODO validation
        Ok(Self(token_data.to_string()))
    }
}
