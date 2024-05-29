use crate::prelude::*;

use cosmrs::AccountId;
use derive_more::Display;

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

impl Signer {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    pub fn new_empty() -> Self {
        Self::new(String::new())
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

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

impl From<AccountId> for Signer {
    fn from(account_id: AccountId) -> Self {
        Self(account_id.to_string())
    }
}
