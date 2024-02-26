//! Data type definition and utilities for the
//! version field of a channel end.
//!

use core::convert::Infallible;
use core::fmt::{Display, Error as FmtError, Formatter};
use core::str::FromStr;

use ibc_primitives::prelude::*;

use super::error::ChannelError;

/// The version field for a `ChannelEnd`.
///
/// This field is opaque to the core IBC protocol.
/// No explicit validation is necessary, and the
/// spec (v1) currently allows empty strings.
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
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Version(String);

impl Version {
    pub fn new(v: String) -> Self {
        Self(v)
    }

    pub fn empty() -> Self {
        Self::new("".to_string())
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn verify_is_expected(&self, expected: Version) -> Result<(), ChannelError> {
        if self != &expected {
            return Err(ChannelError::VersionNotSupported {
                expected,
                actual: self.clone(),
            });
        }
        Ok(())
    }
}

impl From<String> for Version {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl FromStr for Version {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::new(s.to_string()))
    }
}

impl Display for Version {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        write!(f, "{}", self.0)
    }
}
