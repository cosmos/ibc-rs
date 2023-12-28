//! Defines Non-Fungible Token Transfer (ICS-721) data types.
use core::fmt::{self, Display, Formatter};
use core::str::FromStr;

use ibc_core::primitives::prelude::*;

use crate::error::NftTransferError;

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
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Data(BTreeMap<String, DataValue>);

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
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DataValue {
    value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    mime: Option<String>,
}

impl Display for Data {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", serde_json::to_string(&self.0).expect("infallible"))
    }
}

impl FromStr for Data {
    type Err = NftTransferError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let data: BTreeMap<String, DataValue> =
            serde_json::from_str(s).map_err(|_| NftTransferError::InvalidJsonData)?;

        Ok(Self(data))
    }
}
