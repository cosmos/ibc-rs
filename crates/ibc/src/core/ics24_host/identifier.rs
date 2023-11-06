//! Defines identifier types

pub(crate) mod validate;
use core::fmt::{Debug, Display, Error as FmtError, Formatter};
use core::str::FromStr;

use derive_more::Into;
use displaydoc::Display;
use validate::*;

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
    /// Creates a new `ChainId` with the given chain identifier.
    ///
    /// It checks the identifier for valid characters according to `ICS-24`
    /// specification and returns a `ChainId` successfully.
    /// Stricter checks beyond `ICS-24` rests with the users,
    /// based on their requirements.
    ///
    /// If the chain identifier is in the {chain name}-{revision number} format,
    /// the revision number is parsed. Otherwise, revision number is set to 0.
    ///
    /// ```
    /// use ibc::core::ics24_host::identifier::ChainId;
    ///
    /// let chain_id = "chainA";
    /// let id = ChainId::new(chain_id).unwrap();
    /// assert_eq!(id.revision_number(), 0);
    /// assert_eq!(id.as_str(), chain_id);
    ///
    /// let chain_id = "chainA-12";
    /// let id = ChainId::new(chain_id).unwrap();
    /// assert_eq!(id.revision_number(), 12);
    /// assert_eq!(id.as_str(), chain_id);
    /// ```
    pub fn new(chain_id: &str) -> Result<Self, IdentifierError> {
        Self::from_str(chain_id)
    }

    /// Get a reference to the underlying string.
    pub fn as_str(&self) -> &str {
        &self.id
    }

    pub fn split_chain_id(&self) -> Result<(&str, u64), IdentifierError> {
        parse_chain_id_string(self.as_str())
    }

    /// Extract the revision number from the chain identifier
    pub fn revision_number(&self) -> u64 {
        self.revision_number
    }

    /// Increases `ChainId`s revision number by one.
    /// Fails if the chain identifier is not in
    /// `{chain_name}-{revision_number}` format or
    /// the revision number overflows.
    ///
    /// ```
    /// use ibc::core::ics24_host::identifier::ChainId;
    ///
    /// let mut chain_id = ChainId::new("chainA-1").unwrap();
    /// assert!(chain_id.increment_revision_number().is_ok());
    /// assert_eq!(chain_id.revision_number(), 2);
    ///
    /// let mut chain_id = ChainId::new(&format!("chainA-{}", u64::MAX)).unwrap();
    /// assert!(chain_id.increment_revision_number().is_err());
    /// assert_eq!(chain_id.revision_number(), u64::MAX);
    /// ```
    pub fn increment_revision_number(&mut self) -> Result<(), IdentifierError> {
        let (chain_name, _) = self.split_chain_id()?;
        let inc_revision_number = self
            .revision_number
            .checked_add(1)
            .ok_or(IdentifierError::RevisionNumberOverflow)?;
        self.id = format!("{}-{}", chain_name, inc_revision_number);
        self.revision_number = inc_revision_number;
        Ok(())
    }

    /// A convenient method to check if the `ChainId` forms a valid identifier
    /// with the desired min/max length. However, ICS-24 does not specify a
    /// certain min or max lengths for chain identifiers.
    pub fn validate_length(&self, min_length: u64, max_length: u64) -> Result<(), IdentifierError> {
        match self.split_chain_id() {
            Ok((chain_name, _)) => validate_prefix_length(chain_name, min_length, max_length),
            _ => validate_identifier_length(&self.id, min_length, max_length),
        }
    }
}

/// Construct a `ChainId` from a string literal only if it forms a valid
/// identifier.
impl FromStr for ChainId {
    type Err = IdentifierError;

    fn from_str(id: &str) -> Result<Self, Self::Err> {
        // Identifier string must have a maximum length of 64 characters.

        // Validates the chain name for allowed characters according to ICS-24.
        validate_identifier_chars(id)?;
        match parse_chain_id_string(id) {
            Ok((chain_name, revision_number)) => {
                // Validate if the chain name with revision number has a valid length.
                validate_prefix_length(chain_name, 1, 64)?;
                Ok(Self {
                    id: id.into(),
                    revision_number,
                })
            }

            _ => {
                // Validate if the identifier has a valid length.
                validate_identifier_length(id, 1, 64)?;
                Ok(Self {
                    id: id.into(),
                    revision_number: 0,
                })
            }
        }
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
    chain_id_str
        .rsplit_once('-')
        .filter(|(_, rev_number_str)| {
            // Validates the revision number not to start with leading zeros, like "01".
            // Zero is the only allowed revision number with leading zero.
            rev_number_str.as_bytes().first() != Some(&b'0') || rev_number_str.len() == 1
        })
        .and_then(|(chain_name, rev_number_str)| {
            // Parses the revision number string into a `u64` and checks its validity.
            rev_number_str
                .parse()
                .ok()
                .map(|revision_number| (chain_name, revision_number))
        })
        .ok_or(IdentifierError::UnformattedRevisionNumber {
            chain_id: chain_id_str.to_string(),
        })
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
    /// chain identifier is not formatted with revision number
    UnformattedRevisionNumber { chain_id: String },
    /// revision number overflowed
    RevisionNumberOverflow,
    /// identifier cannot be empty
    Empty,
}

#[cfg(feature = "std")]
impl std::error::Error for IdentifierError {}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("chainA-0", "chainA", 0)]
    #[case("chainA-1", "chainA", 1)]
    #[case("chainA--1", "chainA-", 1)]
    #[case("chainA-1-2", "chainA-1", 2)]
    #[case("111-2", "111", 2)]
    #[case("----1", "---", 1)]
    #[case("._+-1", "._+", 1)]
    #[case(&("A".repeat(43) + "-3"), &("A".repeat(43)), 3)]
    fn test_valid_chain_id_with_rev(
        #[case] raw_chain_id: &str,
        #[case] chain_name: &str,
        #[case] revision_number: u64,
    ) {
        let chain_id = ChainId::new(raw_chain_id).unwrap();
        assert!(chain_id.validate_length(1, 64).is_ok());
        assert_eq!(
            chain_id,
            ChainId {
                id: format!("{chain_name}-{revision_number}"),
                revision_number
            }
        );
    }

    #[rstest]
    #[case("chainA")]
    #[case("chainA.2")]
    #[case("123")]
    #[case("._+")]
    #[case("chainA-")]
    #[case("chainA-a")]
    #[case("chainA-01")]
    #[case("chainA-1-")]
    #[case(&"A".repeat(64))]
    #[case::special_case("chainA-0")]
    fn test_valid_chain_id_without_rev(#[case] chain_name: &str) {
        let chain_id = ChainId::new(chain_name).unwrap();
        assert!(chain_id.validate_length(1, 64).is_ok());
        assert_eq!(
            chain_id,
            ChainId {
                id: chain_name.into(),
                revision_number: 0
            }
        );
    }

    #[rstest]
    #[case(&"A".repeat(65))]
    #[case(&("A".repeat(44) + "-123"))]
    #[case("-1")]
    #[case(" ----1")]
    #[case(" ")]
    #[case(" chainA")]
    #[case("chain A")]
    #[case(" chainA.2")]
    #[case(" chainA.2-1")]
    #[case(" 1")]
    #[case(" -")]
    #[case("   -1")]
    #[case("/chainA-1")]
    fn test_invalid_chain_id(#[case] chain_id_str: &str) {
        assert!(ChainId::new(chain_id_str).is_err());
    }

    #[test]
    fn test_inc_revision_number() {
        let mut chain_id = ChainId::new("chainA-1").unwrap();

        assert!(chain_id.increment_revision_number().is_ok());
        assert_eq!(chain_id.revision_number(), 2);
        assert_eq!(chain_id.as_str(), "chainA-2");

        assert!(chain_id.increment_revision_number().is_ok());
        assert_eq!(chain_id.revision_number(), 3);
        assert_eq!(chain_id.as_str(), "chainA-3");
    }

    #[test]
    fn test_failed_inc_revision_number() {
        let mut chain_id = ChainId::new("chainA").unwrap();

        assert!(chain_id.increment_revision_number().is_err());
        assert_eq!(chain_id.revision_number(), 0);
        assert_eq!(chain_id.as_str(), "chainA");
    }
}
