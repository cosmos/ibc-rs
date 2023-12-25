//! Contains the `PacketData` type that defines the structure of NFT transfers' packet bytes

use core::convert::TryFrom;

use ibc_core::primitives::prelude::*;
use ibc_core::primitives::Signer;
use ibc_proto::ibc::applications::nft_transfer::v1::NonFungibleTokenPacketData as RawPacketData;

use crate::class::{ClassData, ClassId, ClassUri};
use crate::error::NftTransferError;
use crate::memo::Memo;
use crate::token::{TokenData, TokenId, TokenUri};

/// Defines the structure of token transfers' packet bytes
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "serde",
    serde(try_from = "RawPacketData", into = "RawPacketData")
)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "parity-scale-codec",
    derive(parity_scale_codec::Encode, parity_scale_codec::Decode,)
)]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PacketData {
    pub class_id: ClassId,
    pub class_uri: Option<ClassUri>,
    pub class_data: Option<ClassData>,
    pub token_ids: Vec<TokenId>,
    pub token_uris: Vec<TokenUri>,
    pub token_data: Vec<TokenData>,
    pub sender: Signer,
    pub receiver: Signer,
    pub memo: Memo,
}

impl TryFrom<RawPacketData> for PacketData {
    type Error = NftTransferError;

    fn try_from(raw_pkt_data: RawPacketData) -> Result<Self, Self::Error> {
        let token_ids = raw_pkt_data
            .token_ids
            .iter()
            .map(|t| t.parse().expect("infallible"))
            .collect();
        let token_uris: Result<Vec<TokenUri>, _> =
            raw_pkt_data.token_uris.iter().map(|t| t.parse()).collect();
        let token_data: Result<Vec<TokenData>, _> =
            raw_pkt_data.token_data.iter().map(|t| t.parse()).collect();
        let class_uri = if raw_pkt_data.class_uri.is_empty() {
            None
        } else {
            Some(raw_pkt_data.class_uri.parse()?)
        };
        let class_data = if raw_pkt_data.class_data.is_empty() {
            None
        } else {
            Some(raw_pkt_data.class_data.parse()?)
        };
        Ok(Self {
            class_id: raw_pkt_data.class_id.parse()?,
            class_uri,
            class_data,
            token_ids,
            token_uris: token_uris?,
            token_data: token_data?,
            sender: raw_pkt_data.sender.into(),
            receiver: raw_pkt_data.receiver.into(),
            memo: raw_pkt_data.memo.into(),
        })
    }
}

impl From<PacketData> for RawPacketData {
    fn from(pkt_data: PacketData) -> Self {
        Self {
            class_id: pkt_data.class_id.to_string(),
            class_uri: pkt_data
                .class_uri
                .map(|c| c.to_string())
                .unwrap_or_default(),
            class_data: pkt_data
                .class_data
                .map(|c| c.to_string())
                .unwrap_or_default(),
            token_ids: pkt_data.token_ids.iter().map(|t| t.to_string()).collect(),
            token_uris: pkt_data.token_uris.iter().map(|t| t.to_string()).collect(),
            token_data: pkt_data.token_data.iter().map(|t| t.to_string()).collect(),
            sender: pkt_data.sender.to_string(),
            receiver: pkt_data.receiver.to_string(),
            memo: pkt_data.memo.to_string(),
        }
    }
}
