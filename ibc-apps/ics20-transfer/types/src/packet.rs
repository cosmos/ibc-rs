//! Contains the `PacketData` type that defines the structure of token transfers' packet bytes

use core::str::FromStr;

use ibc_core::primitives::prelude::*;
use ibc_core::primitives::Signer;
use ibc_proto::ibc::applications::transfer::v2::FungibleTokenPacketData as RawPacketData;

use super::error::TokenTransferError;
use super::{Amount, Memo, PrefixedCoin, PrefixedDenom};

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

#[cfg(test)]
mod tests {
    use primitive_types::U256;

    use super::*;
    use crate::BaseCoin;

    impl PacketData {
        pub fn new_dummy() -> Self {
            let address: Signer = "cosmos1wxeyh7zgn4tctjzs0vtqpc6p5cxq5t2muzl7ng"
                .to_string()
                .into();

            Self {
                token: BaseCoin {
                    denom: "uatom".parse().unwrap(),
                    amount: U256::from(10).into(),
                }
                .into(),
                sender: address.clone(),
                receiver: address,
                memo: "".to_string().into(),
            }
        }

        pub fn ser_json_assert_eq(&self, json: &str) {
            let ser = serde_json::to_string(&self).unwrap();
            assert_eq!(ser, json);
        }

        pub fn deser_json_assert_eq(&self, json: &str) {
            let deser: Self = serde_json::from_str(json).unwrap();
            assert_eq!(&deser, self);
        }
    }

    pub fn dummy_json_packet_data() -> &'static str {
        r#"{"denom":"uatom","amount":"10","sender":"cosmos1wxeyh7zgn4tctjzs0vtqpc6p5cxq5t2muzl7ng","receiver":"cosmos1wxeyh7zgn4tctjzs0vtqpc6p5cxq5t2muzl7ng","memo":""}"#
    }

    pub fn dummy_json_packet_data_without_memo() -> &'static str {
        r#"{"denom":"uatom","amount":"10","sender":"cosmos1wxeyh7zgn4tctjzs0vtqpc6p5cxq5t2muzl7ng","receiver":"cosmos1wxeyh7zgn4tctjzs0vtqpc6p5cxq5t2muzl7ng"}"#
    }

    /// Ensures `PacketData` properly encodes to JSON by first converting to a
    /// `RawPacketData` and then serializing that.
    #[test]
    fn test_packet_data_ser() {
        PacketData::new_dummy().ser_json_assert_eq(dummy_json_packet_data());
    }

    /// Ensures `PacketData` properly decodes from JSON by first deserializing to a
    /// `RawPacketData` and then converting from that.
    #[test]
    fn test_packet_data_deser() {
        PacketData::new_dummy().deser_json_assert_eq(dummy_json_packet_data());
        PacketData::new_dummy().deser_json_assert_eq(dummy_json_packet_data_without_memo());
    }
}
