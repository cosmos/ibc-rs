use crate::prelude::*;
use core::fmt::{Display, Error as FmtError, Formatter};
use serde_derive::{Deserialize, Serialize};

#[cfg_attr(
    feature = "scale-codec",
    derive(codec::Encode, codec::Decode, scale_info::TypeInfo)
)]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
/// Type of the client, depending on the specific consensus algorithm.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ClientType(String);

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
