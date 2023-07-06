//! Defines the `ClientType` format, typically used in chain IDs.

use crate::prelude::*;
use core::{
    fmt::{Display, Error as FmtError, Formatter},
    str::FromStr,
};

use crate::core::ics24_host::{
    identifier::validate::validate_client_type, identifier::IdentifierError,
};

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
/// Type of the client, depending on the specific consensus algorithm.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ClientType(String);

impl ClientType {
    /// Constructs a new `ClientType` from the given `String` if it ends with a valid client identifier.
    pub fn new(s: &str) -> Result<Self, IdentifierError> {
        let s_trim = s.trim();
        validate_client_type(s_trim)?;
        Ok(Self(s_trim.to_string()))
    }

    /// Yields this identifier as a borrowed `&str`
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for ClientType {
    type Err = IdentifierError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

impl Display for ClientType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        write!(f, "ClientType({})", self.0)
    }
}
