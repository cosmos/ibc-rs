use core::convert::Infallible;
use core::fmt::{self, Display};
use core::str::FromStr;

use crate::prelude::*;

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
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Memo(String);

impl AsRef<str> for Memo {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Display for Memo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for Memo {
    fn from(memo: String) -> Self {
        Self(memo)
    }
}

impl FromStr for Memo {
    type Err = Infallible;

    fn from_str(memo: &str) -> Result<Self, Infallible> {
        Ok(Self(memo.to_owned()))
    }
}
