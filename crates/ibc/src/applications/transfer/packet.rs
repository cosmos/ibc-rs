//! Contains the `PacketData` type that defines the structure of token transfers' packet bytes

use alloc::string::ToString;
use core::convert::TryFrom;
use core::str::FromStr;

#[cfg(feature = "schema")]
use crate::alloc::borrow::ToOwned;

use ibc_proto::ibc::applications::transfer::v2::FungibleTokenPacketData as RawPacketData;

use super::error::TokenTransferError;
use super::{Amount, Memo, PrefixedCoin, PrefixedDenom};
use crate::signer::Signer;

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
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PacketData {
    pub token: PrefixedCoin,
    pub sender: Signer,
    pub receiver: Signer,
    pub memo: Memo,
}

impl TryFrom<RawPacketData> for PacketData {
    type Error = TokenTransferError;

    fn try_from(raw_pkt_data: RawPacketData) -> Result<Self, Self::Error> {
        // This denom may be prefixed or unprefixed.
        let denom = PrefixedDenom::from_str(&raw_pkt_data.denom)?;
        let amount = Amount::from_str(&raw_pkt_data.amount)?;
        Ok(Self {
            token: PrefixedCoin { denom, amount },
            sender: raw_pkt_data.sender.into(),
            receiver: raw_pkt_data.receiver.into(),
            memo: raw_pkt_data.memo.into(),
        })
    }
}

impl From<PacketData> for RawPacketData {
    fn from(pkt_data: PacketData) -> Self {
        Self {
            denom: pkt_data.token.denom.to_string(),
            amount: pkt_data.token.amount.to_string(),
            sender: pkt_data.sender.to_string(),
            receiver: pkt_data.receiver.to_string(),
            memo: pkt_data.memo.to_string(),
        }
    }
}
