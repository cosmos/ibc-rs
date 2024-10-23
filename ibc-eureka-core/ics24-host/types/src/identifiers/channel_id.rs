use core::fmt::{Debug, Display, Error as FmtError, Formatter};
use core::str::FromStr;

use derive_more::{From, Into};
use ibc_primitives::prelude::*;

use crate::error::IdentifierError;
use crate::identifiers::ClientId;

const CHANNEL_ID_PREFIX: &str = "channel";

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
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, From, Into)]
pub struct ChannelId(ClientId);

impl ChannelId {
    /// Returns the static prefix to be used across all channel identifiers.
    pub fn prefix() -> &'static str {
        CHANNEL_ID_PREFIX
    }

    /// Get this identifier as a borrowed `&str`
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    /// Get this identifier as a borrowed byte slice
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

/// This implementation provides a `to_string` method.
impl Display for ChannelId {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        write!(f, "{}", self.0)
    }
}

impl FromStr for ChannelId {
    type Err = IdentifierError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.parse()?))
    }
}

impl AsRef<str> for ChannelId {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

/// Equality check against string literal (satisfies &ChannelId == &str).
/// ```
/// use core::str::FromStr;
/// use ibc_eureka_core_host_types::identifiers::ChannelId;
/// let channel_id = ChannelId::from_str("channel-0");
/// assert!(channel_id.is_ok());
/// channel_id.map(|id| {assert_eq!(&id, "channel-0")});
/// ```
impl PartialEq<str> for ChannelId {
    fn eq(&self, other: &str) -> bool {
        self.as_str().eq(other)
    }
}
