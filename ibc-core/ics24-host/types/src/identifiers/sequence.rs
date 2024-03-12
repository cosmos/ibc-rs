use ibc_primitives::prelude::*;

use crate::error::IdentifierError;

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
/// The sequence number of a packet enforces ordering among packets from the same source.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Sequence(u64);

impl core::str::FromStr for Sequence {
    type Err = IdentifierError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from(s.parse::<u64>().map_err(|e| {
            IdentifierError::InvalidStringAsSequence {
                value: s.to_string(),
                reason: e.to_string(),
            }
        })?))
    }
}

impl Sequence {
    /// Gives the sequence number.
    pub fn value(&self) -> u64 {
        self.0
    }

    /// Returns `true` if the sequence number is zero.
    pub fn is_zero(&self) -> bool {
        self.0 == 0
    }

    /// Increments the sequence number by one.
    pub fn increment(&self) -> Sequence {
        Sequence(self.0 + 1)
    }

    /// Encodes the sequence number into a byte array in big endian.
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_be_bytes().to_vec()
    }
}

impl From<u64> for Sequence {
    fn from(seq: u64) -> Self {
        Sequence(seq)
    }
}

impl From<Sequence> for u64 {
    fn from(s: Sequence) -> u64 {
        s.0
    }
}

impl core::fmt::Display for Sequence {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        write!(f, "{}", self.0)
    }
}
