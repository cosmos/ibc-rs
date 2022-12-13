use core::str::FromStr;

use crate::prelude::*;

use derive_more::Display;
use serde::{Deserialize, Serialize};

#[derive(Debug, displaydoc::Display)]
pub enum SignerError {
    /// signer cannot be empty
    EmptySigner,
}

#[cfg(feature = "std")]
impl std::error::Error for SignerError {}

#[cfg_attr(feature = "parity-scale-codec", derive(scale_info::TypeInfo))]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Display)]
pub struct Signer(String);

#[cfg(feature = "parity-scale-codec")]
impl parity_scale_codec::Encode for Signer {
    fn encode_to<T: parity_scale_codec::Output + ?Sized>(&self, writer: &mut T) {
        let value = self.0.as_bytes().to_vec();
        value.encode_to(writer);
    }
}
#[cfg(feature = "parity-scale-codec")]
impl parity_scale_codec::Decode for Signer {
    fn decode<I: parity_scale_codec::Input>(
        input: &mut I,
    ) -> Result<Self, parity_scale_codec::Error> {
        let value = Vec::<u8>::decode(input)?;
        let value = String::from_utf8(value)
            .map_err(|_| parity_scale_codec::Error::from("Utf8 decode to string error"))?;
        Ok(Signer(value))
    }
}

impl FromStr for Signer {
    type Err = SignerError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_string();
        if s.trim().is_empty() {
            return Err(SignerError::EmptySigner);
        }
        Ok(Self(s))
    }
}

impl AsRef<str> for Signer {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}
