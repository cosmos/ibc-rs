use core::fmt::{Display, Error as FmtError, Formatter};
use core::str::FromStr;

use derive_more::Into;
use ibc_primitives::prelude::*;

use crate::error::IdentifierError;
use crate::validate::validate_port_identifier;

const TRANSFER_PORT_ID: &str = "transfer";

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
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Into)]
pub struct PortId(String);

impl PortId {
    pub fn new(id: String) -> Result<Self, IdentifierError> {
        Self::from_str(&id)
    }

    /// Infallible creation of the well-known transfer port
    pub fn transfer() -> Self {
        Self(TRANSFER_PORT_ID.to_string())
    }

    /// Get this identifier as a borrowed `&str`
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Get this identifier as a borrowed byte slice
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }

    pub fn validate(&self) -> Result<(), IdentifierError> {
        validate_port_identifier(self.as_str())
    }
}

/// This implementation provides a `to_string` method.
impl Display for PortId {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        write!(f, "{}", self.0)
    }
}

impl FromStr for PortId {
    type Err = IdentifierError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        validate_port_identifier(s).map(|_| Self(s.to_string()))
    }
}

impl AsRef<str> for PortId {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}
