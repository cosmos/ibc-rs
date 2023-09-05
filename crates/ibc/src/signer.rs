use derive_more::Display;

use crate::prelude::*;

/// Represents the address of the signer of the current transaction
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
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Display)]
pub struct Signer(String);

impl From<String> for Signer {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl AsRef<str> for Signer {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}
