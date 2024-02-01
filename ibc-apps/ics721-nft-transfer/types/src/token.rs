//! Defines Non-Fungible Token Transfer (ICS-721) token types.
use core::fmt::{self, Display};
use core::str::FromStr;

use http::Uri;
use ibc_core::primitives::prelude::*;
#[cfg(feature = "serde")]
use ibc_core::primitives::serializers;

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
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
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
        let ids: Result<Vec<TokenId>, _> = token_ids.iter().map(|t| t.parse()).collect();
        let mut ids = ids?;
        ids.sort();
        ids.dedup();
        if ids.len() != token_ids.len() {
            return Err(NftTransferError::DuplicatedTokenIds);
        }
        Ok(Self(ids))
    }
}

/// Token URI for an NFT
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TokenUri(
    #[cfg_attr(feature = "serde", serde(with = "serializers"))]
    #[cfg_attr(feature = "schema", schemars(with = "String"))]
    Uri,
);

#[cfg(feature = "borsh")]
impl borsh::BorshSerialize for TokenUri {
    fn serialize<W: borsh::maybestd::io::Write>(
        &self,
        writer: &mut W,
    ) -> borsh::maybestd::io::Result<()> {
        borsh::BorshSerialize::serialize(&self.to_string(), writer)
    }
}

#[cfg(feature = "borsh")]
impl borsh::BorshDeserialize for TokenUri {
    fn deserialize_reader<R: borsh::maybestd::io::Read>(
        reader: &mut R,
    ) -> borsh::maybestd::io::Result<Self> {
        let uri = String::deserialize_reader(reader)?;
        Ok(TokenUri::from_str(&uri).map_err(|_| borsh::maybestd::io::ErrorKind::Other)?)
    }
}

#[cfg(feature = "parity-scale-codec")]
impl parity_scale_codec::Encode for TokenUri {
    fn encode_to<T: parity_scale_codec::Output + ?Sized>(&self, writer: &mut T) {
        self.to_string().encode_to(writer);
    }
}

#[cfg(feature = "parity-scale-codec")]
impl parity_scale_codec::Decode for TokenUri {
    fn decode<I: parity_scale_codec::Input>(
        input: &mut I,
    ) -> Result<Self, parity_scale_codec::Error> {
        let uri = String::decode(input)?;
        TokenUri::from_str(&uri).map_err(|_| parity_scale_codec::Error::from("from str error"))
    }
}

#[cfg(feature = "parity-scale-codec")]
impl scale_info::TypeInfo for TokenUri {
    type Identity = Self;

    fn type_info() -> scale_info::Type {
        scale_info::Type::builder()
            .path(scale_info::Path::new("TokenUri", module_path!()))
            .composite(
                scale_info::build::Fields::unnamed()
                    .field(|f| f.ty::<String>().type_name("String")),
            )
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
            Ok(uri) => Ok(Self(uri)),
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
#[derive(Clone, Debug, PartialEq, Eq, derive_more::AsRef)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "serde")]
    #[test]
    fn test_serde_json_roundtrip() {
        fn serde_roundtrip(token_uri: TokenUri) {
            let serialized =
                serde_json::to_string(&token_uri).expect("failed to serialize TokenUri");
            let deserialized = serde_json::from_str::<TokenUri>(&serialized)
                .expect("failed to deserialize TokenUri");

            assert_eq!(deserialized, token_uri);
        }

        let uri = "/foo/bar?baz".parse::<Uri>().unwrap();
        serde_roundtrip(TokenUri(uri));

        let uri = "https://www.rust-lang.org/install.html"
            .parse::<Uri>()
            .unwrap();
        serde_roundtrip(TokenUri(uri));
    }

    #[cfg(feature = "borsh")]
    #[test]
    fn test_borsh_roundtrip() {
        fn borsh_roundtrip(token_uri: TokenUri) {
            use borsh::{BorshDeserialize, BorshSerialize};

            let token_uri_bytes = token_uri.try_to_vec().unwrap();
            let res = TokenUri::try_from_slice(&token_uri_bytes).unwrap();

            assert_eq!(token_uri, res);
        }

        let uri = "/foo/bar?baz".parse::<Uri>().unwrap();
        borsh_roundtrip(TokenUri(uri));

        let uri = "https://www.rust-lang.org/install.html"
            .parse::<Uri>()
            .unwrap();
        borsh_roundtrip(TokenUri(uri));
    }
}
