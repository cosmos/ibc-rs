//! Contains the `PacketData` type that defines the structure of NFT transfers' packet bytes

use core::convert::TryFrom;

use ibc_core::primitives::prelude::*;
use ibc_core::primitives::Signer;
use ibc_proto::ibc::applications::nft_transfer::v1::NonFungibleTokenPacketData as RawPacketData;

use crate::class::{ClassData, ClassUri, PrefixedClassId};
use crate::error::NftTransferError;
use crate::memo::Memo;
use crate::token::{TokenData, TokenIds, TokenUri};

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
    pub class_id: PrefixedClassId,
    pub class_uri: Option<ClassUri>,
    pub class_data: Option<ClassData>,
    pub token_ids: TokenIds,
    pub token_uris: Vec<TokenUri>,
    pub token_data: Vec<TokenData>,
    pub sender: Signer,
    pub receiver: Signer,
    pub memo: Memo,
}

impl PacketData {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        class_id: PrefixedClassId,
        class_uri: Option<ClassUri>,
        class_data: Option<ClassData>,
        token_ids: TokenIds,
        token_uris: Vec<TokenUri>,
        token_data: Vec<TokenData>,
        sender: Signer,
        receiver: Signer,
        memo: Memo,
    ) -> Result<Self, NftTransferError> {
        if token_ids.0.is_empty() {
            return Err(NftTransferError::NoTokenId);
        }
        let num = token_ids.0.len();
        let num_uri = token_uris.len();
        let num_data = token_data.len();
        if (num_uri != 0 && num_uri != num) || (num_data != 0 && num_data != num) {
            return Err(NftTransferError::TokenMismatched);
        }
        Ok(Self {
            class_id,
            class_uri,
            class_data,
            token_ids,
            token_uris,
            token_data,
            sender,
            receiver,
            memo,
        })
    }
}

impl TryFrom<RawPacketData> for PacketData {
    type Error = NftTransferError;

    fn try_from(raw_pkt_data: RawPacketData) -> Result<Self, Self::Error> {
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

        let token_ids = raw_pkt_data.token_ids.try_into()?;
        let token_uris: Result<Vec<TokenUri>, _> =
            raw_pkt_data.token_uris.iter().map(|t| t.parse()).collect();
        let token_data: Result<Vec<TokenData>, _> =
            raw_pkt_data.token_data.iter().map(|t| t.parse()).collect();
        Self::new(
            raw_pkt_data.class_id.parse()?,
            class_uri,
            class_data,
            token_ids,
            token_uris?,
            token_data?,
            raw_pkt_data.sender.into(),
            raw_pkt_data.receiver.into(),
            raw_pkt_data.memo.into(),
        )
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
            token_ids: pkt_data
                .token_ids
                .as_ref()
                .iter()
                .map(|t| t.to_string())
                .collect(),
            token_uris: pkt_data.token_uris.iter().map(|t| t.to_string()).collect(),
            token_data: pkt_data.token_data.iter().map(|t| t.to_string()).collect(),
            sender: pkt_data.sender.to_string(),
            receiver: pkt_data.receiver.to_string(),
            memo: pkt_data.memo.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use core::str::FromStr;

    use super::*;

    const DUMMY_ADDRESS: &str = "cosmos1wxeyh7zgn4tctjzs0vtqpc6p5cxq5t2muzl7ng";
    const DUMMY_CLASS_ID: &str = "class";
    const DUMMY_URI: &str = "http://example.com";
    const DUMMY_DATA: &str =
        r#"{"name":{"value":"Crypto Creatures"},"image":{"value":"binary","mime":"image/png"}}"#;

    impl PacketData {
        pub fn new_dummy() -> Self {
            let address: Signer = DUMMY_ADDRESS.to_string().into();

            Self {
                class_id: PrefixedClassId::from_str(DUMMY_CLASS_ID).unwrap(),
                class_uri: Some(ClassUri::from_str(DUMMY_URI).unwrap()),
                class_data: Some(ClassData::from_str(DUMMY_DATA).unwrap()),
                token_ids: TokenIds::try_from(vec!["token_0".to_string(), "token_1".to_string()])
                    .unwrap(),
                token_uris: vec![
                    TokenUri::from_str(DUMMY_URI).unwrap(),
                    TokenUri::from_str(DUMMY_URI).unwrap(),
                ],
                token_data: vec![
                    TokenData::from_str(DUMMY_DATA).unwrap(),
                    TokenData::from_str(DUMMY_DATA).unwrap(),
                ],
                sender: address.clone(),
                receiver: address,
                memo: "".to_string().into(),
            }
        }

        pub fn new_min_dummy() -> Self {
            let address: Signer = DUMMY_ADDRESS.to_string().into();

            Self {
                class_id: PrefixedClassId::from_str(DUMMY_CLASS_ID).unwrap(),
                class_uri: None,
                class_data: None,
                token_ids: TokenIds::try_from(vec!["token_0".to_string()]).unwrap(),
                token_uris: vec![],
                token_data: vec![],
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

    fn dummy_min_json_packet_data() -> &'static str {
        r#"{"classId":"class","tokenIds":["token_0"],"sender":"cosmos1wxeyh7zgn4tctjzs0vtqpc6p5cxq5t2muzl7ng","receiver":"cosmos1wxeyh7zgn4tctjzs0vtqpc6p5cxq5t2muzl7ng"}"#
    }

    fn dummy_json_packet_data() -> &'static str {
        r#"{"classId":"class","classUri":"http://example.com/","classData":"{\"image\":{\"value\":\"binary\",\"mime\":\"image/png\"},\"name\":{\"value\":\"Crypto Creatures\"}}","tokenIds":["token_0","token_1"],"tokenUris":["http://example.com/","http://example.com/"],"tokenData":["{\"image\":{\"value\":\"binary\",\"mime\":\"image/png\"},\"name\":{\"value\":\"Crypto Creatures\"}}","{\"image\":{\"value\":\"binary\",\"mime\":\"image/png\"},\"name\":{\"value\":\"Crypto Creatures\"}}"],"sender":"cosmos1wxeyh7zgn4tctjzs0vtqpc6p5cxq5t2muzl7ng","receiver":"cosmos1wxeyh7zgn4tctjzs0vtqpc6p5cxq5t2muzl7ng","memo":""}"#
    }

    fn dummy_json_packet_data_without_memo() -> &'static str {
        r#"{"classId":"class","classUri":"http://example.com","classData":"{\"name\":{\"value\":\"Crypto Creatures\"},\"image\":{\"value\":\"binary\",\"mime\":\"image/png\"}}","tokenIds":["token_0","token_1"],"tokenUris":["http://example.com","http://example.com"],"tokenData":["{\"name\":{\"value\":\"Crypto Creatures\"},\"image\":{\"value\":\"binary\",\"mime\":\"image/png\"}}","{\"name\":{\"value\":\"Crypto Creatures\"},\"image\":{\"value\":\"binary\",\"mime\":\"image/png\"}}"],"sender":"cosmos1wxeyh7zgn4tctjzs0vtqpc6p5cxq5t2muzl7ng","receiver":"cosmos1wxeyh7zgn4tctjzs0vtqpc6p5cxq5t2muzl7ng"}"#
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
        PacketData::new_min_dummy().deser_json_assert_eq(dummy_min_json_packet_data());
    }

    #[test]
    fn test_invalid_packet_data() {
        // the number of tokens is mismatched
        let packet_data = r#"{"class_id":"class","token_ids":["token_0","token_1"],"token_uris":["http://example.com"],"token_data":["{\"image\":{\"value\":\"binary\",\"mime\":\"image/png\"},\"name\":{\"value\":\"Crypto Creatures\"}}"],"sender":"cosmos1wxeyh7zgn4tctjzs0vtqpc6p5cxq5t2muzl7ng","receiver":"cosmos1wxeyh7zgn4tctjzs0vtqpc6p5cxq5t2muzl7ng","memo":""}"#;
        assert!(
            serde_json::from_str::<PacketData>(packet_data).is_err(),
            "num of token data is unmatched"
        );

        // No token ID
        let packet_data = r#"{"class_id":"class","token_ids":[],"sender":"cosmos1wxeyh7zgn4tctjzs0vtqpc6p5cxq5t2muzl7ng","receiver":"cosmos1wxeyh7zgn4tctjzs0vtqpc6p5cxq5t2muzl7ng","memo":""}"#;
        assert!(
            serde_json::from_str::<PacketData>(packet_data).is_err(),
            "no token ID"
        );
    }
}
