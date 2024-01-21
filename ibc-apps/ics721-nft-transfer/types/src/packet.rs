//! Contains the `PacketData` type that defines the structure of NFT transfers' packet bytes

use ibc_core::primitives::prelude::*;
use ibc_core::primitives::Signer;

use crate::class::{ClassData, ClassUri, PrefixedClassId};
use crate::error::NftTransferError;
use crate::memo::Memo;
use crate::serializers;
use crate::token::{TokenData, TokenIds, TokenUri};

/// Defines the structure of token transfers' packet bytes
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[serde(rename_all = "camelCase")]
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
    #[cfg_attr(feature = "serde", serde(with = "serializers"))]
    #[cfg_attr(feature = "schema", schemars(with = "String"))]
    pub class_id: PrefixedClassId,
    pub class_uri: Option<ClassUri>,
    pub class_data: Option<ClassData>,
    pub token_ids: TokenIds,
    // Need `Option` to decode `null` value
    pub token_uris: Option<Vec<TokenUri>>,
    // Need `Option` to decode `null` value
    pub token_data: Option<Vec<TokenData>>,
    pub sender: Signer,
    pub receiver: Signer,
    pub memo: Option<Memo>,
}

impl PacketData {
    /// Performs the basic validation of the packet data fields.
    pub fn validate_basic(&self) -> Result<(), NftTransferError> {
        if self.token_ids.0.is_empty() {
            return Err(NftTransferError::NoTokenId);
        }
        let num = self.token_ids.0.len();
        let num_uri = self
            .token_uris
            .as_ref()
            .map(|t| t.len())
            .unwrap_or_default();
        let num_data = self
            .token_data
            .as_ref()
            .map(|t| t.len())
            .unwrap_or_default();
        if (num_uri != 0 && num_uri != num) || (num_data != 0 && num_data != num) {
            return Err(NftTransferError::TokenMismatched);
        }
        Ok(())
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
        r#"{"image":{"value":"binary","mime":"image/png"},"name":{"value":"Crypto Creatures"}}"#;

    impl PacketData {
        pub fn new_dummy(memo: Option<&str>) -> Self {
            let address: Signer = DUMMY_ADDRESS.to_string().into();

            Self {
                class_id: PrefixedClassId::from_str(DUMMY_CLASS_ID).unwrap(),
                class_uri: Some(ClassUri::from_str(DUMMY_URI).unwrap()),
                class_data: Some(ClassData::from_str(DUMMY_DATA).unwrap()),
                token_ids: TokenIds::try_from(vec!["token_0".to_string(), "token_1".to_string()])
                    .unwrap(),
                token_uris: Some(vec![
                    TokenUri::from_str(DUMMY_URI).unwrap(),
                    TokenUri::from_str(DUMMY_URI).unwrap(),
                ]),
                token_data: Some(vec![
                    TokenData::from_str(DUMMY_DATA).unwrap(),
                    TokenData::from_str(DUMMY_DATA).unwrap(),
                ]),
                sender: address.clone(),
                receiver: address,
                memo: memo.map(|m| m.to_string().into()),
            }
        }

        pub fn new_min_dummy() -> Self {
            let address: Signer = DUMMY_ADDRESS.to_string().into();

            Self {
                class_id: PrefixedClassId::from_str(DUMMY_CLASS_ID).unwrap(),
                class_uri: None,
                class_data: None,
                token_ids: TokenIds::try_from(vec!["token_0".to_string()]).unwrap(),
                token_uris: None,
                token_data: None,
                sender: address.clone(),
                receiver: address,
                memo: None,
            }
        }

        pub fn ser_json_assert_eq(&self, json: &str) {
            let ser = serde_json::to_string(&self).unwrap();
            assert_eq!(ser, json);
        }

        pub fn deser_json_assert_eq(&self, json: &str) {
            let deser: Self = serde_json::from_str(json).unwrap();

            if let Some(data) = &deser.class_data {
                assert!(data.as_ref().parse_as_ics721_data().is_ok());
            };

            if let Some(token_data) = &deser.token_data {
                for data in token_data.iter() {
                    assert!(data.as_ref().parse_as_ics721_data().is_ok());
                }
            }

            assert_eq!(&deser, self);
        }
    }

    fn dummy_min_json_packet_data() -> &'static str {
        r#"{"classId":"class","tokenIds":["token_0"],"sender":"cosmos1wxeyh7zgn4tctjzs0vtqpc6p5cxq5t2muzl7ng","receiver":"cosmos1wxeyh7zgn4tctjzs0vtqpc6p5cxq5t2muzl7ng"}"#
    }

    fn dummy_json_packet_data() -> &'static str {
        r#"{"classId":"class","classUri":"http://example.com/","classData":"eyJpbWFnZSI6eyJ2YWx1ZSI6ImJpbmFyeSIsIm1pbWUiOiJpbWFnZS9wbmcifSwibmFtZSI6eyJ2YWx1ZSI6IkNyeXB0byBDcmVhdHVyZXMifX0=","tokenIds":["token_0","token_1"],"tokenUris":["http://example.com/","http://example.com/"],"tokenData":["eyJpbWFnZSI6eyJ2YWx1ZSI6ImJpbmFyeSIsIm1pbWUiOiJpbWFnZS9wbmcifSwibmFtZSI6eyJ2YWx1ZSI6IkNyeXB0byBDcmVhdHVyZXMifX0=","eyJpbWFnZSI6eyJ2YWx1ZSI6ImJpbmFyeSIsIm1pbWUiOiJpbWFnZS9wbmcifSwibmFtZSI6eyJ2YWx1ZSI6IkNyeXB0byBDcmVhdHVyZXMifX0="],"sender":"cosmos1wxeyh7zgn4tctjzs0vtqpc6p5cxq5t2muzl7ng","receiver":"cosmos1wxeyh7zgn4tctjzs0vtqpc6p5cxq5t2muzl7ng","memo":"memo"}"#
    }

    fn dummy_json_packet_data_without_memo() -> &'static str {
        r#"{"classId":"class","classUri":"http://example.com/","classData":"eyJpbWFnZSI6eyJ2YWx1ZSI6ImJpbmFyeSIsIm1pbWUiOiJpbWFnZS9wbmcifSwibmFtZSI6eyJ2YWx1ZSI6IkNyeXB0byBDcmVhdHVyZXMifX0=","tokenIds":["token_0","token_1"],"tokenUris":["http://example.com/","http://example.com/"],"tokenData":["eyJpbWFnZSI6eyJ2YWx1ZSI6ImJpbmFyeSIsIm1pbWUiOiJpbWFnZS9wbmcifSwibmFtZSI6eyJ2YWx1ZSI6IkNyeXB0byBDcmVhdHVyZXMifX0=","eyJpbWFnZSI6eyJ2YWx1ZSI6ImJpbmFyeSIsIm1pbWUiOiJpbWFnZS9wbmcifSwibmFtZSI6eyJ2YWx1ZSI6IkNyeXB0byBDcmVhdHVyZXMifX0="],"sender":"cosmos1wxeyh7zgn4tctjzs0vtqpc6p5cxq5t2muzl7ng","receiver":"cosmos1wxeyh7zgn4tctjzs0vtqpc6p5cxq5t2muzl7ng"}"#
    }

    /// Ensures `PacketData` properly encodes to JSON by first converting to a
    /// `RawPacketData` and then serializing that.
    #[test]
    fn test_packet_data_ser() {
        PacketData::new_dummy(Some("memo")).ser_json_assert_eq(dummy_json_packet_data());
    }

    /// Ensures `PacketData` properly decodes from JSON by first deserializing to a
    /// `RawPacketData` and then converting from that.
    #[test]
    fn test_packet_data_deser() {
        PacketData::new_dummy(Some("memo")).deser_json_assert_eq(dummy_json_packet_data());
        PacketData::new_dummy(None).deser_json_assert_eq(dummy_json_packet_data_without_memo());
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
