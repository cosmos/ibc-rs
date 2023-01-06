use alloc::string::ToString;
use core::convert::TryFrom;
use core::str::FromStr;

use ibc_proto::ibc::applications::transfer::v2::FungibleTokenPacketData as RawPacketData;

use super::error::TokenTransferError;
use super::{Amount, PrefixedCoin, PrefixedDenom};
use crate::signer::Signer;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "serde",
    serde(try_from = "RawPacketData", into = "RawPacketData")
)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PacketData {
    pub token: PrefixedCoin,
    pub sender: Signer,
    pub receiver: Signer,
}

impl TryFrom<RawPacketData> for PacketData {
    type Error = TokenTransferError;

    fn try_from(raw_pkt_data: RawPacketData) -> Result<Self, Self::Error> {
        // This denom may be prefixed or unprefixed.
        let denom = PrefixedDenom::from_str(&raw_pkt_data.denom)?;
        let amount = Amount::from_str(&raw_pkt_data.amount)?;
        Ok(Self {
            token: PrefixedCoin { denom, amount },
            sender: raw_pkt_data
                .sender
                .parse()
                .map_err(TokenTransferError::Signer)?,
            receiver: raw_pkt_data
                .receiver
                .parse()
                .map_err(TokenTransferError::Signer)?,
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
        }
    }
}
