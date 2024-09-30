use core::fmt::{Debug, Display, Error as FmtError, Formatter};
use core::str::FromStr;

use ibc_primitives::prelude::*;

use crate::error::IdentifierError;
use crate::identifiers::ClientId;
use crate::validate::validate_channel_identifier;

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
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ChannelId {
    V1(String),
    V2(ClientId),
}

impl ChannelId {
    /// Builds a new channel identifier. Like client and connection identifiers, channel ids are
    /// deterministically formed from two elements: a prefix `prefix`, and a monotonically
    /// increasing `counter`, separated by a dash "-".
    /// The prefix is currently determined statically (see `ChannelId::prefix()`) so this method
    /// accepts a single argument, the `counter`.
    ///
    /// ```
    /// # use ibc_core_host_types::identifiers::ChannelId;
    /// let chan_id = ChannelId::new(27);
    /// assert_eq!(chan_id.to_string(), "channel-27");
    /// ```
    pub fn new(identifier: u64) -> Self {
        let id = format!("{}-{}", Self::prefix(), identifier);
        Self::V1(id)
    }

    /// Returns the static prefix to be used across all channel identifiers.
    pub fn prefix() -> &'static str {
        CHANNEL_ID_PREFIX
    }

    /// Get this identifier as a borrowed `&str`
    pub fn as_str(&self) -> &str {
        match self {
            ChannelId::V1(id) => id.as_str(),
            ChannelId::V2(id) => id.as_str(),
        }
    }

    /// Get this identifier as a borrowed byte slice
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            ChannelId::V1(id) => id.as_bytes(),
            ChannelId::V2(id) => id.as_bytes(),
        }
    }

    pub fn zero() -> Self {
        Self::new(0)
    }
}

/// This implementation provides a `to_string` method.
impl Display for ChannelId {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        match self {
            ChannelId::V1(id) => write!(f, "{id}"),
            ChannelId::V2(id) => write!(f, "{id}"),
        }
    }
}

impl FromStr for ChannelId {
    type Err = IdentifierError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        validate_channel_identifier(s)
            .map(|_| Self::V1(s.to_string()))
            .or_else(|_| Ok(Self::V2(s.parse()?)))
    }
}

impl AsRef<str> for ChannelId {
    fn as_ref(&self) -> &str {
        match self {
            ChannelId::V1(id) => id,
            ChannelId::V2(id) => id.as_str(),
        }
    }
}

/// Equality check against string literal (satisfies &ChannelId == &str).
/// ```
/// use core::str::FromStr;
/// use ibc_core_host_types::identifiers::ChannelId;
/// let channel_id = ChannelId::from_str("channel-0");
/// assert!(channel_id.is_ok());
/// channel_id.map(|id| {assert_eq!(&id, "channel-0")});
/// ```
impl PartialEq<str> for ChannelId {
    fn eq(&self, other: &str) -> bool {
        self.as_str().eq(other)
    }
}
