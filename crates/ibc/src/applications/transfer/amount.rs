//! Contains the `Amount` type, which represents amounts of tokens transferred.

use core::{ops::Deref, str::FromStr};
use derive_more::{Display, From, Into};

use super::error::TokenTransferError;
use primitive_types::U256;

#[cfg(feature = "schema")]
use crate::alloc::{borrow::ToOwned, string::String};

/// A type for representing token transfer amounts.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Display, From, Into)]
pub struct Amount(
    #[cfg_attr(feature = "schema", schemars(with = "String"))]
    #[serde(serialize_with = "serialize")]
    #[serde(deserialize_with = "deserialize")]
    U256,
);

#[cfg(feature = "parity-scale-codec")]
impl parity_scale_codec::WrapperTypeDecode for Amount {
    type Wrapped = [u64; 4];
}

#[cfg(feature = "parity-scale-codec")]
impl parity_scale_codec::WrapperTypeEncode for Amount {}

impl Deref for Amount {
    type Target = [u64; 4];

    fn deref(&self) -> &Self::Target {
        &self.0 .0
    }
}

impl From<[u64; 4]> for Amount {
    fn from(value: [u64; 4]) -> Self {
        Self(U256(value))
    }
}

impl Amount {
    pub fn checked_add(self, rhs: Self) -> Option<Self> {
        self.0.checked_add(rhs.0).map(Self)
    }

    pub fn checked_sub(self, rhs: Self) -> Option<Self> {
        self.0.checked_sub(rhs.0).map(Self)
    }
}

impl AsRef<U256> for Amount {
    fn as_ref(&self) -> &U256 {
        &self.0
    }
}

impl FromStr for Amount {
    type Err = TokenTransferError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let amount = U256::from_dec_str(s).map_err(TokenTransferError::InvalidAmount)?;
        Ok(Self(amount))
    }
}

impl From<u64> for Amount {
    fn from(v: u64) -> Self {
        Self(v.into())
    }
}

#[cfg(feature = "serde")]
pub fn serialize<S>(value: &U256, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    use crate::alloc::string::ToString;
    serializer.serialize_str(value.to_string().as_ref())
}

#[cfg(feature = "serde")]
fn deserialize<'de, D>(deserializer: D) -> Result<U256, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use crate::prelude::*;
    use serde::Deserialize;
    U256::from_dec_str(<String>::deserialize(deserializer)?.as_str())
        .map_err(serde::de::Error::custom)
}

#[cfg(test)]
#[cfg(feature = "serde")]
mod tests {
    use super::Amount;

    #[test]
    fn serde_amount() {
        let value = Amount::from(42);
        let string = serde_json::to_string(&value).expect("can serde string");
        assert_eq!(string, "\"42\"");
        let binary = serde_json::to_vec(&value).expect("can serde binary");
        let de: Amount = serde_json::from_slice(binary.as_ref()).expect("can deserialize");
        assert_eq!(de, value);
    }
}
