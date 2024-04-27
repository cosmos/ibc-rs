use core::fmt::{Display, Error as FmtError, Formatter};
use core::str::FromStr;

use derive_more::Into;
use ibc_primitives::prelude::*;

use crate::error::IdentifierError;
use crate::validate::validate_connection_identifier;

const CONNECTION_ID_PREFIX: &str = "connection";

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
pub struct ConnectionId(String);

impl ConnectionId {
    /// Builds a new connection identifier. Connection identifiers are deterministically formed from
    /// two elements: a prefix `prefix`, and a monotonically increasing `counter`; these are
    /// separated by a dash "-". The prefix is currently determined statically (see
    /// `ConnectionId::prefix()`) so this method accepts a single argument, the `counter`.
    ///
    /// ```
    /// # use ibc_core_host_types::identifiers::ConnectionId;
    /// let conn_id = ConnectionId::new(11);
    /// assert_eq!(&conn_id, "connection-11");
    /// ```
    pub fn new(identifier: u64) -> Self {
        let id = format!("{}-{}", Self::prefix(), identifier);
        Self(id)
    }

    /// Returns the static prefix to be used across all connection identifiers.
    pub fn prefix() -> &'static str {
        CONNECTION_ID_PREFIX
    }

    /// Get this identifier as a borrowed `&str`
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Get this identifier as a borrowed byte slice
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }

    /// Return ConnectionId with identifier 0
    pub fn zero() -> Self {
        Self::new(0)
    }
}

/// This implementation provides a `to_string` method.
impl Display for ConnectionId {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        write!(f, "{}", self.0)
    }
}

impl FromStr for ConnectionId {
    type Err = IdentifierError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        validate_connection_identifier(s).map(|_| Self(s.to_string()))
    }
}

/// Equality check against string literal (satisfies &ConnectionId == &str).
/// ```
/// use core::str::FromStr;
/// use ibc_core_host_types::identifiers::ConnectionId;
/// let conn_id = ConnectionId::from_str("connection-0");
/// assert!(conn_id.is_ok());
/// conn_id.map(|id| {assert_eq!(&id, "connection-0")});
/// ```
impl PartialEq<str> for ConnectionId {
    fn eq(&self, other: &str) -> bool {
        self.as_str().eq(other)
    }
}
