//! Contains the `Amount` type, which represents amounts of tokens transferred.
use core::ops::Deref;
use core::str::FromStr;

use derive_more::{Display, From, Into};
use ibc_core::primitives::prelude::*;
#[cfg(feature = "serde")]
use ibc_core::primitives::serializers;
use primitive_types::U256;

use super::error::TokenTransferError;

/// A type for representing token transfer amounts.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Display, From, Into)]
pub struct Amount(
    #[cfg_attr(feature = "schema", schemars(with = "String"))]
    #[cfg_attr(feature = "serde", serde(serialize_with = "serializers::serialize"))]
    #[cfg_attr(feature = "serde", serde(deserialize_with = "deserialize"))]
    U256,
);

#[cfg(feature = "parity-scale-codec")]
impl parity_scale_codec::WrapperTypeDecode for Amount {
    type Wrapped = [u64; 4];
}

#[cfg(feature = "parity-scale-codec")]
impl parity_scale_codec::WrapperTypeEncode for Amount {}

#[cfg(feature = "borsh")]
impl borsh::BorshSerialize for Amount {
    fn serialize<W: borsh::maybestd::io::Write>(
        &self,
        writer: &mut W,
    ) -> borsh::maybestd::io::Result<()> {
        // Note: a "word" is 8 bytes (i.e. a u64)
        let words = self.as_slice();
        let bytes: Vec<u8> = words.iter().flat_map(|word| word.to_be_bytes()).collect();

        writer.write_all(&bytes)
    }
}
#[cfg(feature = "borsh")]
impl borsh::BorshDeserialize for Amount {
    fn deserialize_reader<R: borsh::maybestd::io::Read>(
        reader: &mut R,
    ) -> borsh::maybestd::io::Result<Self> {
        const NUM_BYTES_IN_U64: usize = 8;
        const NUM_WORDS_IN_U256: usize = 4;

        let mut buf = [0; 32];
        let bytes_read = reader.read(&mut buf)?;
        if bytes_read != 32 {
            return Err(borsh::maybestd::io::Error::new(
                borsh::maybestd::io::ErrorKind::InvalidInput,
                format!("Expected to read 32 bytes, read {bytes_read}"),
            ));
        }

        let words: Vec<u64> = buf
            .chunks_exact(NUM_BYTES_IN_U64)
            .map(|word| {
                let word: [u8; NUM_BYTES_IN_U64] = word
                    .try_into()
                    .expect("exact chunks of 8 bytes are expected to be 8 bytes");
                u64::from_be_bytes(word)
            })
            .collect();

        let four_words: [u64; NUM_WORDS_IN_U256] = words
            .try_into()
            .expect("U256 is always 4 four words, and we confirmed that we read 32 bytes");

        Ok(four_words.into())
    }
}

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
fn deserialize<'de, D>(deserializer: D) -> Result<U256, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::Deserialize;
    U256::from_dec_str(<String>::deserialize(deserializer)?.as_str())
        .map_err(serde::de::Error::custom)
}

#[cfg(test)]
mod tests {
    use super::Amount;

    #[cfg(feature = "serde")]
    #[test]
    fn serde_amount() {
        let value = Amount::from(42);
        let string = serde_json::to_string(&value).expect("can serde string");
        assert_eq!(string, "\"42\"");
        let binary = serde_json::to_vec(&value).expect("can serde binary");
        let de: Amount = serde_json::from_slice(binary.as_ref()).expect("can deserialize");
        assert_eq!(de, value);
    }

    #[cfg(feature = "borsh")]
    #[test]
    fn borsh_amount() {
        use borsh::BorshDeserialize;

        let value = Amount::from(42);
        let serialized = borsh::to_vec(&value).unwrap();

        // Amount is supposed to be a U256 according to the spec, which is 32 bytes
        assert_eq!(serialized.len(), 32);

        let value_deserialized = Amount::try_from_slice(&serialized).unwrap();

        assert_eq!(value, value_deserialized);
    }
}
