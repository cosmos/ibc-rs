use crate::prelude::*;

use core::time::Duration;
use core::{
    fmt::{Display, Error as FmtError, Formatter},
    u64,
};

use ibc_proto::protobuf::Protobuf;

use ibc_proto::ibc::core::connection::v1::{
    ConnectionEnd as RawConnectionEnd, Counterparty as RawCounterparty,
    IdentifiedConnection as RawIdentifiedConnection,
};

use crate::core::ics02_client::error::ClientError;
use crate::core::ics03_connection::error::ConnectionError;
use crate::core::ics03_connection::version::Version;
use crate::core::ics23_commitment::commitment::CommitmentPrefix;
use crate::core::ics24_host::error::ValidationError;
use crate::core::ics24_host::identifier::{ClientId, ConnectionId};
use crate::timestamp::ZERO_DURATION;

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
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct IdentifiedConnectionEnd {
    pub connection_id: ConnectionId,
    pub connection_end: ConnectionEnd,
}

impl IdentifiedConnectionEnd {
    pub fn new(connection_id: ConnectionId, connection_end: ConnectionEnd) -> Self {
        IdentifiedConnectionEnd {
            connection_id,
            connection_end,
        }
    }

    pub fn id(&self) -> &ConnectionId {
        &self.connection_id
    }

    pub fn end(&self) -> &ConnectionEnd {
        &self.connection_end
    }
}

impl Protobuf<RawIdentifiedConnection> for IdentifiedConnectionEnd {}

impl TryFrom<RawIdentifiedConnection> for IdentifiedConnectionEnd {
    type Error = ConnectionError;

    fn try_from(value: RawIdentifiedConnection) -> Result<Self, Self::Error> {
        let raw_connection_end = RawConnectionEnd {
            client_id: value.client_id.to_string(),
            versions: value.versions,
            state: value.state,
            counterparty: value.counterparty,
            delay_period: value.delay_period,
        };

        Ok(IdentifiedConnectionEnd {
            connection_id: value
                .id
                .parse()
                .map_err(ConnectionError::InvalidIdentifier)?,
            connection_end: raw_connection_end.try_into()?,
        })
    }
}

impl From<IdentifiedConnectionEnd> for RawIdentifiedConnection {
    fn from(value: IdentifiedConnectionEnd) -> Self {
        RawIdentifiedConnection {
            id: value.connection_id.to_string(),
            client_id: value.connection_end.client_id.to_string(),
            versions: value
                .connection_end
                .versions
                .iter()
                .map(|v| From::from(v.clone()))
                .collect(),
            state: value.connection_end.state as i32,
            delay_period: value.connection_end.delay_period.as_nanos() as u64,
            counterparty: Some(value.connection_end.counterparty().clone().into()),
        }
    }
}

#[cfg_attr(
    feature = "parity-scale-codec",
    derive(parity_scale_codec::Encode, parity_scale_codec::Decode,)
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ConnectionEnd {
    pub state: State,
    client_id: ClientId,
    counterparty: Counterparty,
    versions: Vec<Version>,
    delay_period: Duration,
}

mod sealed {
    use super::*;

    #[cfg_attr(
        feature = "borsh",
        derive(borsh::BorshSerialize, borsh::BorshDeserialize)
    )]
    struct InnerConnectionEnd {
        pub state: State,
        client_id: ClientId,
        counterparty: Counterparty,
        versions: Vec<Version>,
        delay_period_secs: u64,
        delay_period_nanos: u32,
    }

    impl From<InnerConnectionEnd> for ConnectionEnd {
        fn from(value: InnerConnectionEnd) -> Self {
            Self {
                state: value.state,
                client_id: value.client_id,
                counterparty: value.counterparty,
                versions: value.versions,
                delay_period: Duration::new(value.delay_period_secs, value.delay_period_nanos),
            }
        }
    }

    impl From<ConnectionEnd> for InnerConnectionEnd {
        fn from(value: ConnectionEnd) -> Self {
            Self {
                state: value.state,
                client_id: value.client_id,
                counterparty: value.counterparty,
                versions: value.versions,
                delay_period_secs: value.delay_period.as_secs(),
                delay_period_nanos: value.delay_period.subsec_nanos(),
            }
        }
    }

    #[cfg(feature = "borsh")]
    impl borsh::BorshSerialize for ConnectionEnd {
        fn serialize<W: borsh::maybestd::io::Write>(
            &self,
            writer: &mut W,
        ) -> borsh::maybestd::io::Result<()> {
            let value = InnerConnectionEnd::from(self.clone());
            borsh::BorshSerialize::serialize(&value, writer)
        }
    }

    #[cfg(feature = "borsh")]
    impl borsh::BorshDeserialize for ConnectionEnd {
        fn deserialize_reader<R: borsh::maybestd::io::Read>(
            reader: &mut R,
        ) -> borsh::maybestd::io::Result<Self> {
            let inner_conn_end = InnerConnectionEnd::deserialize_reader(reader)?;
            Ok(ConnectionEnd::from(inner_conn_end))
        }
    }

    #[cfg(feature = "parity-scale-codec")]
    impl scale_info::TypeInfo for ConnectionEnd {
        type Identity = Self;

        fn type_info() -> scale_info::Type {
            scale_info::Type::builder()
                .path(scale_info::Path::new("ConnectionEnd", module_path!()))
                .composite(
                    scale_info::build::Fields::named()
                        .field(|f| f.ty::<State>().name("state").type_name("State"))
                        .field(|f| f.ty::<ClientId>().name("client_id").type_name("ClientId"))
                        .field(|f| {
                            f.ty::<Counterparty>()
                                .name("counterparty")
                                .type_name("Counterparty")
                        })
                        .field(|f| {
                            f.ty::<Vec<Version>>()
                                .name("versions")
                                .type_name("Vec<Version>")
                        })
                        .field(|f| f.ty::<u64>().name("delay_period_secs").type_name("u64"))
                        .field(|f| f.ty::<u32>().name("delay_period_nanos").type_name("u32")),
                )
        }
    }
}

impl Default for ConnectionEnd {
    fn default() -> Self {
        Self {
            state: State::Uninitialized,
            client_id: Default::default(),
            counterparty: Default::default(),
            versions: Vec::new(),
            delay_period: ZERO_DURATION,
        }
    }
}

impl Protobuf<RawConnectionEnd> for ConnectionEnd {}

impl TryFrom<RawConnectionEnd> for ConnectionEnd {
    type Error = ConnectionError;
    fn try_from(value: RawConnectionEnd) -> Result<Self, Self::Error> {
        let state = value.state.try_into()?;
        if state == State::Uninitialized {
            return Ok(ConnectionEnd::default());
        }
        if value.client_id.is_empty() {
            return Err(ConnectionError::EmptyProtoConnectionEnd);
        }

        Ok(Self::new(
            state,
            value
                .client_id
                .parse()
                .map_err(ConnectionError::InvalidIdentifier)?,
            value
                .counterparty
                .ok_or(ConnectionError::MissingCounterparty)?
                .try_into()?,
            value
                .versions
                .into_iter()
                .map(Version::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            Duration::from_nanos(value.delay_period),
        ))
    }
}

impl From<ConnectionEnd> for RawConnectionEnd {
    fn from(value: ConnectionEnd) -> Self {
        RawConnectionEnd {
            client_id: value.client_id.to_string(),
            versions: value
                .versions
                .iter()
                .map(|v| From::from(v.clone()))
                .collect(),
            state: value.state as i32,
            counterparty: Some(value.counterparty.into()),
            delay_period: value.delay_period.as_nanos() as u64,
        }
    }
}

impl ConnectionEnd {
    pub fn new(
        state: State,
        client_id: ClientId,
        counterparty: Counterparty,
        versions: Vec<Version>,
        delay_period: Duration,
    ) -> Self {
        Self {
            state,
            client_id,
            counterparty,
            versions,
            delay_period,
        }
    }

    /// Getter for the state of this connection end.
    pub fn state(&self) -> &State {
        &self.state
    }

    /// Setter for the `state` field.
    pub fn set_state(&mut self, new_state: State) {
        self.state = new_state;
    }

    /// Setter for the `counterparty` field.
    pub fn set_counterparty(&mut self, new_cparty: Counterparty) {
        self.counterparty = new_cparty;
    }

    /// Setter for the `version` field.
    pub fn set_version(&mut self, new_version: Version) {
        self.versions = vec![new_version];
    }

    /// Helper function to compare the counterparty of this end with another counterparty.
    pub fn counterparty_matches(&self, other: &Counterparty) -> bool {
        self.counterparty.eq(other)
    }

    /// Helper function to compare the client id of this end with another client identifier.
    pub fn client_id_matches(&self, other: &ClientId) -> bool {
        self.client_id.eq(other)
    }

    /// Helper function to determine whether the connection is open.
    pub fn is_open(&self) -> bool {
        self.state_matches(&State::Open)
    }

    /// Helper function to determine whether the connection is uninitialized.
    pub fn is_uninitialized(&self) -> bool {
        self.state_matches(&State::Uninitialized)
    }

    /// Helper function to compare the state of this end with another state.
    pub fn state_matches(&self, other: &State) -> bool {
        self.state.eq(other)
    }

    /// Getter for the client id on the local party of this connection end.
    pub fn client_id(&self) -> &ClientId {
        &self.client_id
    }

    /// Getter for the list of versions in this connection end.
    pub fn versions(&self) -> &[Version] {
        &self.versions
    }

    /// Getter for the counterparty.
    pub fn counterparty(&self) -> &Counterparty {
        &self.counterparty
    }

    /// Getter for the delay_period field. This represents the duration, at minimum,
    /// to delay the sending of a packet after the client update for that packet has been submitted.
    pub fn delay_period(&self) -> Duration {
        self.delay_period
    }

    /// TODO: Clean this up, probably not necessary.
    pub fn validate_basic(&self) -> Result<(), ValidationError> {
        self.counterparty.validate_basic()
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
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Counterparty {
    client_id: ClientId,
    pub connection_id: Option<ConnectionId>,
    prefix: CommitmentPrefix,
}

impl Protobuf<RawCounterparty> for Counterparty {}

// Converts from the wire format RawCounterparty. Typically used from the relayer side
// during queries for response validation and to extract the Counterparty structure.
impl TryFrom<RawCounterparty> for Counterparty {
    type Error = ConnectionError;

    fn try_from(raw_counterparty: RawCounterparty) -> Result<Self, Self::Error> {
        let connection_id: Option<ConnectionId> = if raw_counterparty.connection_id.is_empty() {
            None
        } else {
            Some(
                raw_counterparty
                    .connection_id
                    .parse()
                    .map_err(ConnectionError::InvalidIdentifier)?,
            )
        };
        Ok(Counterparty::new(
            raw_counterparty
                .client_id
                .parse()
                .map_err(ConnectionError::InvalidIdentifier)?,
            connection_id,
            raw_counterparty
                .prefix
                .ok_or(ConnectionError::MissingCounterparty)?
                .key_prefix
                .try_into()
                .map_err(|_| ConnectionError::Client(ClientError::EmptyPrefix))?,
        ))
    }
}

impl From<Counterparty> for RawCounterparty {
    fn from(value: Counterparty) -> Self {
        RawCounterparty {
            client_id: value.client_id.as_str().to_string(),
            connection_id: value
                .connection_id
                .map_or_else(|| "".to_string(), |v| v.as_str().to_string()),
            prefix: Some(ibc_proto::ibc::core::commitment::v1::MerklePrefix {
                key_prefix: value.prefix.into_vec(),
            }),
        }
    }
}

impl Counterparty {
    pub fn new(
        client_id: ClientId,
        connection_id: Option<ConnectionId>,
        prefix: CommitmentPrefix,
    ) -> Self {
        Self {
            client_id,
            connection_id,
            prefix,
        }
    }

    /// Getter for the client id.
    pub fn client_id(&self) -> &ClientId {
        &self.client_id
    }

    /// Getter for connection id.
    pub fn connection_id(&self) -> Option<&ConnectionId> {
        self.connection_id.as_ref()
    }

    pub fn prefix(&self) -> &CommitmentPrefix {
        &self.prefix
    }

    pub fn validate_basic(&self) -> Result<(), ValidationError> {
        Ok(())
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
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum State {
    Uninitialized = 0isize,
    Init = 1isize,
    TryOpen = 2isize,
    Open = 3isize,
}

impl State {
    /// Yields the State as a string.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Uninitialized => "UNINITIALIZED",
            Self::Init => "INIT",
            Self::TryOpen => "TRYOPEN",
            Self::Open => "OPEN",
        }
    }

    /// Parses the State out from a i32.
    pub fn from_i32(s: i32) -> Result<Self, ConnectionError> {
        match s {
            0 => Ok(Self::Uninitialized),
            1 => Ok(Self::Init),
            2 => Ok(Self::TryOpen),
            3 => Ok(Self::Open),
            _ => Err(ConnectionError::InvalidState { state: s }),
        }
    }

    /// Returns whether or not this connection state is `Open`.
    pub fn is_open(self) -> bool {
        self == State::Open
    }

    /// Returns whether or not this connection with this state
    /// has progressed less or the same than the argument.
    ///
    /// # Example
    /// ```rust,ignore
    /// assert!(State::Init.less_or_equal_progress(State::Open));
    /// assert!(State::TryOpen.less_or_equal_progress(State::TryOpen));
    /// assert!(!State::Open.less_or_equal_progress(State::Uninitialized));
    /// ```
    pub fn less_or_equal_progress(self, other: Self) -> bool {
        self as u32 <= other as u32
    }
}

impl Display for State {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        write!(f, "{}", self.as_str())
    }
}

impl TryFrom<i32> for State {
    type Error = ConnectionError;
    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Uninitialized),
            1 => Ok(Self::Init),
            2 => Ok(Self::TryOpen),
            3 => Ok(Self::Open),
            _ => Err(ConnectionError::InvalidState { state: value }),
        }
    }
}

impl From<State> for i32 {
    fn from(value: State) -> Self {
        value.into()
    }
}
