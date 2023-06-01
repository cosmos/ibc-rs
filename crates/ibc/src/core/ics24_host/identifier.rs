//! Defines identifier types

pub(crate) mod validate;
use validate::*;

use core::convert::From;
use core::fmt::{Error as FmtError, Formatter};
use core::str::FromStr;

use derive_more::Into;
use displaydoc::Display;

use crate::clients::ics07_tendermint::client_type as tm_client_type;
use crate::core::ics02_client::client_type::ClientType;

use crate::prelude::*;

const CONNECTION_ID_PREFIX: &str = "connection";
const CHANNEL_ID_PREFIX: &str = "channel";

const DEFAULT_CHAIN_ID: &str = "defaultChainId";
const DEFAULT_PORT_ID: &str = "defaultPort";
const TRANSFER_PORT_ID: &str = "transfer";

/// A `ChainId` is in "epoch format" if it is of the form `{chain name}-{epoch number}`,
/// where the epoch number is the number of times the chain was upgraded. Chain IDs not
/// in that format will be assumed to have epoch number 0.
///
/// This is not standardized yet, although compatible with ibc-go.
/// See: <https://github.com/informalsystems/ibc-rs/pull/304#discussion_r503917283>.
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
#[cfg_attr(
    feature = "serde",
    serde(from = "tendermint::chain::Id", into = "tendermint::chain::Id")
)]
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ChainId {
    id: String,
    version: u64,
}

impl ChainId {
    /// Creates a new `ChainId` given a chain name and an epoch number.
    ///
    /// The returned `ChainId` will have the format: `{chain name}-{epoch number}`.
    /// ```
    /// use ibc::core::ics24_host::identifier::ChainId;
    ///
    /// let epoch_number = 10;
    /// let id = ChainId::new("chainA", epoch_number);
    /// assert_eq!(id.version(), epoch_number);
    /// ```
    pub fn new(name: &str, version: u64) -> Self {
        Self {
            id: format!("{name}-{version}"),
            version,
        }
    }

    /// Get a reference to the underlying string.
    pub fn as_str(&self) -> &str {
        &self.id
    }

    // TODO: this should probably be named epoch_number.
    /// Extract the version from this chain identifier.
    pub fn version(&self) -> u64 {
        self.version
    }

    /// Extract the version from the given chain identifier.
    /// ```
    /// use ibc::core::ics24_host::identifier::ChainId;
    ///
    /// assert_eq!(ChainId::chain_version("chain--a-0"), 0);
    /// assert_eq!(ChainId::chain_version("ibc-10"), 10);
    /// assert_eq!(ChainId::chain_version("cosmos-hub-97"), 97);
    /// assert_eq!(ChainId::chain_version("testnet-helloworld-2"), 2);
    /// ```
    pub fn chain_version(chain_id: &str) -> u64 {
        Self::split_chain_id(chain_id).1.unwrap_or(0)
    }

    /// is_epoch_format() checks if a chain_id is in the format required for parsing epochs
    /// The chainID must be in the form: `{chainID}-{version}`
    /// ```
    /// use ibc::core::ics24_host::identifier::ChainId;
    /// assert_eq!(ChainId::is_epoch_format("chainA-0"), false);
    /// assert_eq!(ChainId::is_epoch_format("chainA"), false);
    /// assert_eq!(ChainId::is_epoch_format("chainA-1"), true);
    /// assert_eq!(ChainId::is_epoch_format("c-1"), true);
    /// assert_eq!(ChainId::is_epoch_format("-1"), false);
    /// ```
    pub fn is_epoch_format(chain_id: &str) -> bool {
        Self::split_chain_id(chain_id).1.is_some()
    }

    /// If the chain id is in epoch format, replaces its version with the one
    /// given.
    ///
    /// On success returns modified chain id as `Ok` value.  If the chain id is
    /// not in epoch format or it’s in invalid format returns `Err` value.
    ///
    ///
    /// ```
    /// # use ibc::core::ics24_host::identifier::ChainId;
    /// use tendermint::chain::Id;
    ///
    /// // Hack to be able to create ids in non-epoch format.
    /// // For demonstration purposes only.
    /// fn raw_id(id: &str) -> ChainId { Id::try_from(id).unwrap().into() }
    ///
    /// assert_eq!(ChainId::new("chainA", 1).with_version(2),
    ///            Ok(ChainId::new("chainA", 2)));
    /// assert_eq!(raw_id("chain1").with_version(2),
    ///            Err(raw_id("chain1")));
    /// assert_eq!(raw_id("chain-0").with_version(2),
    ///            Err(raw_id("chain-0")));
    /// ```
    pub fn with_version(mut self, version: u64) -> Result<Self, Self> {
        if self.version != 0 {
            if let (name, Some(_)) = Self::split_chain_id(&self.id) {
                self.id = format!("{name}-{version}");
                self.version = version;
                return Ok(self);
            }
        }
        Err(self)
    }

    /// Splits chain_id into name and version if the id includes epoch number.
    ///
    /// Chain id with epoch number has format `{name}-{version}` where version
    /// is a non-zero unsigned 64-bit number starting with non-zero digit.  If
    /// `chain_id` is in that format, returns `(name, Some(version))`; otherwise
    /// returns `(chain_id, None)`.
    fn split_chain_id(chain_id: &str) -> (&str, Option<u64>) {
        fn split(chain_id: &str) -> Option<(&str, u64)> {
            let (name, version) = chain_id.rsplit_once('-')?;
            let first_digit = *version.as_bytes().first()?;
            if !name.is_empty() && matches!(first_digit, b'1'..=b'9') {
                u64::from_str(version).ok().map(|version| (name, version))
            } else {
                None
            }
        }
        match split(chain_id) {
            Some((name, version)) => (name, Some(version)),
            None => (chain_id, None),
        }
    }
}

impl Default for ChainId {
    fn default() -> Self {
        Self {
            id: String::from(DEFAULT_CHAIN_ID),
            version: 0,
        }
    }
}

impl TryFrom<&'_ str> for ChainId {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let version = Self::chain_version(value);
        if version != 0 {
            Ok(Self {
                id: String::from(value),
                version,
            })
        } else {
            Err(())
        }
    }
}

impl TryFrom<String> for ChainId {
    type Error = ();

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let version = Self::chain_version(&value);
        if version != 0 {
            Ok(Self { id: value, version })
        } else {
            Err(())
        }
    }
}

impl TryFrom<&'_ String> for ChainId {
    type Error = ();

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

impl FromStr for ChainId {
    type Err = ();

    fn from_str(id: &str) -> Result<Self, Self::Err> {
        Self::try_from(id)
    }
}

impl core::fmt::Display for ChainId {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        self.id.fmt(f)
    }
}

impl From<ChainId> for tendermint::chain::Id {
    fn from(id: ChainId) -> Self {
        tendermint::chain::Id::from_str(id.as_str()).unwrap()
    }
}

impl From<tendermint::chain::Id> for ChainId {
    fn from(id: tendermint::chain::Id) -> Self {
        let id = String::from(id);
        let version = Self::chain_version(&id);
        Self { id, version }
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
    /// let tm_client_id = ClientId::new(ClientType::from("07-tendermint".to_string()), 0);
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
impl core::fmt::Display for ClientId {
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
        Self::new(tm_client_type(), 0).unwrap()
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
impl core::fmt::Display for ConnectionId {
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
impl core::fmt::Display for PortId {
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
impl core::fmt::Display for ChannelId {
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
        length: usize,
        min: usize,
        max: usize,
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
