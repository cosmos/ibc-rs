use crate::prelude::*;
use core::fmt::{Display, Error as FmtError, Formatter};
use serde_derive::{Deserialize, Serialize};

#[cfg_attr(feature = "parity-scale-codec", derive(scale_info::TypeInfo))]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
/// Type of the client, depending on the specific consensus algorithm.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ClientType(String);

#[cfg(feature = "parity-scale-codec")]
impl parity_scale_codec::Encode for ClientType {
    fn encode_to<T: parity_scale_codec::Output + ?Sized>(&self, writer: &mut T) {
        let client_type = self.0.as_bytes().to_vec();
        client_type.encode_to(writer);
    }
}
#[cfg(feature = "parity-scale-codec")]
impl parity_scale_codec::Decode for ClientType {
    fn decode<I: parity_scale_codec::Input>(
        input: &mut I,
    ) -> Result<Self, parity_scale_codec::Error> {
        let client_type = Vec::<u8>::decode(input)?;
        let client_type = String::from_utf8(client_type)
            .map_err(|_| parity_scale_codec::Error::from("Utf8 decode to string error"))?;
        Ok(ClientType(client_type))
    }
}

impl ClientType {
    pub fn new(s: String) -> Self {
        Self(s)
    }

    /// Yields this identifier as a borrowed `&str`
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for ClientType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        write!(f, "ClientType({})", self.0)
    }
}
