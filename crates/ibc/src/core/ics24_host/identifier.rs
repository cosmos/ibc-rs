//! Defines identifier types

pub(crate) mod validate;
use validate::*;

use core::fmt::{Debug, Display, Error as FmtError, Formatter};
use core::str::FromStr;

use derive_more::Into;
use displaydoc::Display;

use crate::clients::ics07_tendermint::client_type as tm_client_type;
use crate::core::ics02_client::client_type::ClientType;

use crate::prelude::*;

const CONNECTION_ID_PREFIX: &str = "connection";
const CHANNEL_ID_PREFIX: &str = "channel";

const DEFAULT_PORT_ID: &str = "defaultPort";
const TRANSFER_PORT_ID: &str = "transfer";

/// Defines the domain type for chain identifiers.
///
/// A valid `ChainId` follows the format {chain name}-{revision number} where
/// the revision number indicates how many times the chain has been upgraded.
/// Creating `ChainId`s not in this format will result in an error.
///
/// It should be noted this format is not standardized yet, though it is widely
/// accepted and compatible with Cosmos SDK driven chains.
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
pub struct ChainId {
    id: String,
    revision_number: u64,
}

impl ChainId {
    /// Creates a new `ChainId` with the given chain name and revision number.
    ///
    /// It checks the chain name for valid characters according to `ICS-24`
    /// specification and returns a `ChainId` in the the format of {chain
    /// name}-{revision number}. Stricter checks beyond `ICS-24` rests with
    /// the users, based on their requirements.
    ///
    /// ```
    /// use ibc::core::ics24_host::identifier::ChainId;
    ///
    /// let revision_number = 10;
    /// let id = ChainId::new("chainA", revision_number).unwrap();
    /// assert_eq!(id.revision_number(), revision_number);
    /// ```
    pub fn new(name: &str, revision_number: u64) -> Result<Self, IdentifierError> {
        let prefix = name.trim();
        validate_identifier_chars(prefix)?;
        validate_identifier_length(prefix, 1, 43)?;
        let id = format!("{prefix}-{revision_number}");
        Ok(Self {
            id,
            revision_number,
        })
    }

    /// Get a reference to the underlying string.
    pub fn as_str(&self) -> &str {
        &self.id
    }

    pub fn split_chain_id(&self) -> (&str, u64) {
        parse_chain_id_string(self.as_str())
            .expect("never fails because a valid chain identifier is parsed")
    }

    /// Extract the chain name from the chain identifier
    pub fn chain_name(&self) -> &str {
        self.split_chain_id().0
    }

    /// Extract the revision number from the chain identifier
    pub fn revision_number(&self) -> u64 {
        self.revision_number
    }

    /// Swaps `ChainId`s revision number with the new specified revision number
    /// ```
    /// use ibc::core::ics24_host::identifier::ChainId;
    /// let mut chain_id = ChainId::new("chainA", 1).unwrap();
    /// chain_id.set_revision_number(2);
    /// assert_eq!(chain_id.revision_number(), 2);
    /// ```
    pub fn set_revision_number(&mut self, revision_number: u64) {
        let chain_name = self.chain_name();
        self.id = format!("{}-{}", chain_name, revision_number);
        self.revision_number = revision_number;
    }

    /// A convenient method to check if the `ChainId` forms a valid identifier
    /// with the desired min/max length. However, ICS-24 does not specify a
    /// certain min or max lengths for chain identifiers.
    pub fn validate_length(&self, min_length: u64, max_length: u64) -> Result<(), IdentifierError> {
        validate_prefix_length(self.chain_name(), min_length, max_length)
    }
}

/// Construct a `ChainId` from a string literal only if it forms a valid
/// identifier.
impl FromStr for ChainId {
    type Err = IdentifierError;

    fn from_str(id: &str) -> Result<Self, Self::Err> {
        let (_, revision_number) = parse_chain_id_string(id)?;
        Ok(Self {
            id: id.to_string(),
            revision_number,
        })
    }
}

impl Display for ChainId {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        write!(f, "{}", self.id)
    }
}

/// Parses a string intended to represent a `ChainId` and, if successful,
/// returns a tuple containing the chain name and revision number.
fn parse_chain_id_string(chain_id_str: &str) -> Result<(&str, u64), IdentifierError> {
    let (name, rev_number_str) = match chain_id_str.rsplit_once('-') {
        Some((name, rev_number_str)) => (name, rev_number_str),
        None => {
            return Err(IdentifierError::InvalidCharacter {
                id: chain_id_str.to_string(),
            })
        }
    };

    // Validates the chain name for allowed characters according to ICS-24.
    validate_identifier_chars(name)?;

    // Validates the revision number not to start with leading zeros, like "01".
    if rev_number_str.as_bytes().first() == Some(&b'0') && rev_number_str.len() > 1 {
        return Err(IdentifierError::InvalidCharacter {
            id: chain_id_str.to_string(),
        });
    }

    // Parses the revision number string into a `u64` and checks its validity.
    let revision_number =
        rev_number_str
            .parse()
            .map_err(|_| IdentifierError::InvalidCharacter {
                id: chain_id_str.to_string(),
            })?;

    Ok((name, revision_number))
}

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
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Into)]
pub struct ClientId(String);

impl ClientId {
    /// Builds a new client identifier. Client identifiers are deterministically formed from two
    /// elements: a prefix derived from the client type `ctype`, and a monotonically increasing
    /// `counter`; these are separated by a dash "-".
    ///
    /// ```
    /// # use ibc::core::ics24_host::identifier::ClientId;
    /// # use ibc::core::ics02_client::client_type::ClientType;
    /// # use std::str::FromStr;
    /// let tm_client_id = ClientId::new(ClientType::from_str("07-tendermint").unwrap(), 0);
    /// assert!(tm_client_id.is_ok());
    /// tm_client_id.map(|id| { assert_eq!(&id, "07-tendermint-0") });
    /// ```
    pub fn new(client_type: ClientType, counter: u64) -> Result<Self, IdentifierError> {
        let prefix = client_type.as_str().trim();
        validate_client_type(prefix)?;
        let id = format!("{prefix}-{counter}");
        Self::from_str(id.as_str())
    }

    /// Get this identifier as a borrowed `&str`
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Get this identifier as a borrowed byte slice
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

/// This implementation provides a `to_string` method.
impl Display for ClientId {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        write!(f, "{}", self.0)
    }
}

impl FromStr for ClientId {
    type Err = IdentifierError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        validate_client_identifier(s).map(|_| Self(s.to_string()))
    }
}

impl Default for ClientId {
    fn default() -> Self {
        Self::new(tm_client_type(), 0).expect("Never fails because we use a valid client type")
    }
}

/// Equality check against string literal (satisfies &ClientId == &str).
/// ```
/// use core::str::FromStr;
/// use ibc::core::ics24_host::identifier::ClientId;
/// let client_id = ClientId::from_str("clientidtwo");
/// assert!(client_id.is_ok());
/// client_id.map(|id| {assert_eq!(&id, "clientidtwo")});
/// ```
impl PartialEq<str> for ClientId {
    fn eq(&self, other: &str) -> bool {
        self.as_str().eq(other)
    }
}

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
pub struct ConnectionId(String);

impl ConnectionId {
    /// Builds a new connection identifier. Connection identifiers are deterministically formed from
    /// two elements: a prefix `prefix`, and a monotonically increasing `counter`; these are
    /// separated by a dash "-". The prefix is currently determined statically (see
    /// `ConnectionId::prefix()`) so this method accepts a single argument, the `counter`.
    ///
    /// ```
    /// # use ibc::core::ics24_host::identifier::ConnectionId;
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

impl Default for ConnectionId {
    fn default() -> Self {
        Self::new(0)
    }
}

/// Equality check against string literal (satisfies &ConnectionId == &str).
/// ```
/// use core::str::FromStr;
/// use ibc::core::ics24_host::identifier::ConnectionId;
/// let conn_id = ConnectionId::from_str("connectionId-0");
/// assert!(conn_id.is_ok());
/// conn_id.map(|id| {assert_eq!(&id, "connectionId-0")});
/// ```
impl PartialEq<str> for ConnectionId {
    fn eq(&self, other: &str) -> bool {
        self.as_str().eq(other)
    }
}

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

impl Default for PortId {
    fn default() -> Self {
        Self(DEFAULT_PORT_ID.to_string())
    }
}

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
pub struct ChannelId(String);

impl ChannelId {
    /// Builds a new channel identifier. Like client and connection identifiers, channel ids are
    /// deterministically formed from two elements: a prefix `prefix`, and a monotonically
    /// increasing `counter`, separated by a dash "-".
    /// The prefix is currently determined statically (see `ChannelId::prefix()`) so this method
    /// accepts a single argument, the `counter`.
    ///
    /// ```
    /// # use ibc::core::ics24_host::identifier::ChannelId;
    /// let chan_id = ChannelId::new(27);
    /// assert_eq!(chan_id.to_string(), "channel-27");
    /// ```
    pub fn new(identifier: u64) -> Self {
        let id = format!("{}-{}", Self::prefix(), identifier);
        Self(id)
    }

    /// Returns the static prefix to be used across all channel identifiers.
    pub fn prefix() -> &'static str {
        CHANNEL_ID_PREFIX
    }

    /// Get this identifier as a borrowed `&str`
    pub fn as_str(&self) -> &str {
        &self.0
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
        validate_channel_identifier(s).map(|_| Self(s.to_string()))
    }
}

impl AsRef<str> for ChannelId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Default for ChannelId {
    fn default() -> Self {
        Self::new(0)
    }
}

/// Equality check against string literal (satisfies &ChannelId == &str).
/// ```
/// use core::str::FromStr;
/// use ibc::core::ics24_host::identifier::ChannelId;
/// let channel_id = ChannelId::from_str("channelId-0");
/// assert!(channel_id.is_ok());
/// channel_id.map(|id| {assert_eq!(&id, "channelId-0")});
/// ```
impl PartialEq<str> for ChannelId {
    fn eq(&self, other: &str) -> bool {
        self.as_str().eq(other)
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[derive(Debug, Display)]
pub enum IdentifierError {
    /// identifier `{id}` cannot contain separator '/'
    ContainSeparator { id: String },
    /// identifier `{id}` has invalid length `{length}` must be between `{min}`-`{max}` characters
    InvalidLength {
        id: String,
        length: u64,
        min: u64,
        max: u64,
    },
    /// identifier `{id}` must only contain alphanumeric characters or `.`, `_`, `+`, `-`, `#`, - `[`, `]`, `<`, `>`
    InvalidCharacter { id: String },
    /// identifier prefix `{prefix}` is invalid
    InvalidPrefix { prefix: String },
    /// identifier cannot be empty
    Empty,
}

#[cfg(feature = "std")]
impl std::error::Error for IdentifierError {}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_valid_chain_id() {
        assert!(ChainId::from_str("chainA-0").is_ok());
        assert!(ChainId::from_str("chainA-1").is_ok());
        assert!(ChainId::from_str("chainA--1").is_ok());
        assert!(ChainId::from_str("chainA-1-2").is_ok());
    }

    #[test]
    fn test_invalid_chain_id() {
        assert!(ChainId::from_str("1").is_err());
        assert!(ChainId::from_str("-1").is_err());
        assert!(ChainId::from_str("   -1").is_err());
        assert!(ChainId::from_str("chainA").is_err());
        assert!(ChainId::from_str("chainA-").is_err());
        assert!(ChainId::from_str("chainA-a").is_err());
        assert!(ChainId::from_str("chainA-01").is_err());
        assert!(ChainId::from_str("/chainA-1").is_err());
        assert!(ChainId::from_str("chainA-1-").is_err());
    }
}
