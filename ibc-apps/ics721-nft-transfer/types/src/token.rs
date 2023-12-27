//! Defines Non-Furgible Token Transfer (ICS-721) token types.
use core::fmt::{self, Display};
use core::str::FromStr;

use http::Uri;
use ibc_core::primitives::prelude::*;

use crate::data::Data;
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
    type Err = NftTransferError;

    fn from_str(token_id: &str) -> Result<Self, Self::Err> {
        if token_id.trim().is_empty() {
            Err(NftTransferError::InvalidTokenId)
        } else {
            Ok(Self(token_id.to_string()))
        }
    }
}

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
pub struct TokenIds(pub Vec<TokenId>);

impl TokenIds {
    pub fn as_ref(&self) -> Vec<&TokenId> {
        self.0.iter().collect()
    }
}

impl Display for TokenIds {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            self.0
                .iter()
                .map(|t| t.to_string())
                .collect::<Vec<String>>()
                .join(",")
        )
    }
}

impl TryFrom<Vec<String>> for TokenIds {
    type Error = NftTransferError;

    fn try_from(token_ids: Vec<String>) -> Result<Self, Self::Error> {
        if token_ids.is_empty() {
            return Err(NftTransferError::NoTokenId);
        }
        let token_ids: Result<Vec<TokenId>, _> = token_ids.iter().map(|t| t.parse()).collect();
        Ok(Self(token_ids?))
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
            Err(err) => Err(NftTransferError::InvalidUri {
                uri: token_uri.to_string(),
                validation_error: err,
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
pub struct TokenData(Data);

impl Display for TokenData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for TokenData {
    type Err = NftTransferError;

    fn from_str(token_data: &str) -> Result<Self, Self::Err> {
        let data = Data::from_str(token_data)?;
        Ok(Self(data))
    }
}
