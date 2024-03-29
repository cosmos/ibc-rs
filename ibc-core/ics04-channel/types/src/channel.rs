//! Implementation of IBC channels, as described in ICS-04.

use core::fmt::{Display, Error as FmtError, Formatter};
use core::str::FromStr;

use ibc_core_host_types::identifiers::{ChannelId, ConnectionId, PortId};
use ibc_primitives::prelude::*;
use ibc_primitives::utils::PrettySlice;
use ibc_proto::ibc::core::channel::v1::{
    Channel as RawChannel, Counterparty as RawCounterparty,
    IdentifiedChannel as RawIdentifiedChannel,
};
use ibc_proto::Protobuf;

use crate::error::ChannelError;
use crate::Version;

/// A [`ChannelEnd`] along with its ID and the port it is bound to
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
pub struct IdentifiedChannelEnd {
    pub port_id: PortId,
    pub channel_id: ChannelId,
    pub channel_end: ChannelEnd,
}

impl IdentifiedChannelEnd {
    pub fn new(port_id: PortId, channel_id: ChannelId, channel_end: ChannelEnd) -> Self {
        IdentifiedChannelEnd {
            port_id,
            channel_id,
            channel_end,
        }
    }
}

impl Protobuf<RawIdentifiedChannel> for IdentifiedChannelEnd {}

impl TryFrom<RawIdentifiedChannel> for IdentifiedChannelEnd {
    type Error = ChannelError;

    fn try_from(value: RawIdentifiedChannel) -> Result<Self, Self::Error> {
        if value.upgrade_sequence != 0 {
            return Err(ChannelError::UnsupportedChannelUpgradeSequence);
        }

        let raw_channel_end = RawChannel {
            state: value.state,
            ordering: value.ordering,
            counterparty: value.counterparty,
            connection_hops: value.connection_hops,
            version: value.version,
            upgrade_sequence: value.upgrade_sequence,
        };

        Ok(IdentifiedChannelEnd {
            port_id: value.port_id.parse()?,
            channel_id: value.channel_id.parse()?,
            channel_end: raw_channel_end.try_into()?,
        })
    }
}

impl From<IdentifiedChannelEnd> for RawIdentifiedChannel {
    fn from(value: IdentifiedChannelEnd) -> Self {
        RawIdentifiedChannel {
            state: value.channel_end.state as i32,
            ordering: value.channel_end.ordering as i32,
            counterparty: Some(value.channel_end.counterparty().clone().into()),
            connection_hops: value
                .channel_end
                .connection_hops
                .iter()
                .map(|v| v.as_str().to_string())
                .collect(),
            version: value.channel_end.version.to_string(),
            port_id: value.port_id.to_string(),
            channel_id: value.channel_id.to_string(),
            upgrade_sequence: 0,
        }
    }
}

/// One end of a channel
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
pub struct ChannelEnd {
    pub state: State,
    pub ordering: Order,
    pub remote: Counterparty,
    pub connection_hops: Vec<ConnectionId>,
    pub version: Version,
}

impl Display for ChannelEnd {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        write!(
            f,
            "ChannelEnd {{ state: {}, ordering: {}, remote: {}, connection_hops: {}, version: {} }}",
            self.state, self.ordering, self.remote, PrettySlice(&self.connection_hops), self.version
        )
    }
}

impl Protobuf<RawChannel> for ChannelEnd {}

impl TryFrom<RawChannel> for ChannelEnd {
    type Error = ChannelError;

    fn try_from(value: RawChannel) -> Result<Self, Self::Error> {
        let chan_state: State = State::from_i32(value.state)?;

        let chan_ordering = Order::from_i32(value.ordering)?;

        // Assemble the 'remote' attribute of the Channel, which represents the Counterparty.
        let remote = value
            .counterparty
            .ok_or(ChannelError::MissingCounterparty)?
            .try_into()?;

        // Parse each item in connection_hops into a ConnectionId.
        let connection_hops = value
            .connection_hops
            .into_iter()
            .map(|conn_id| ConnectionId::from_str(conn_id.as_str()))
            .collect::<Result<Vec<_>, _>>()?;

        let version = value.version.into();

        ChannelEnd::new(chan_state, chan_ordering, remote, connection_hops, version)
    }
}

impl From<ChannelEnd> for RawChannel {
    fn from(value: ChannelEnd) -> Self {
        RawChannel {
            state: value.state as i32,
            ordering: value.ordering as i32,
            counterparty: Some(value.counterparty().clone().into()),
            connection_hops: value
                .connection_hops
                .iter()
                .map(|v| v.as_str().to_string())
                .collect(),
            version: value.version.to_string(),
            upgrade_sequence: 0,
        }
    }
}

impl ChannelEnd {
    /// Creates a new `ChannelEnd` without performing basic validation on its arguments.
    ///
    /// NOTE: This method is meant for the proto message conversion from the domain
    /// `MsgChannelOpenInit` and `MsgChannelOpenTry` types to satisfy their `Protobuf`
    /// trait bounds.
    pub(super) fn new_without_validation(
        state: State,
        ordering: Order,
        remote: Counterparty,
        connection_hops: Vec<ConnectionId>,
        version: Version,
    ) -> Self {
        Self {
            state,
            ordering,
            remote,
            connection_hops,
            version,
        }
    }

    /// Creates a new `ChannelEnd` with performing basic validation on its arguments.
    pub fn new(
        state: State,
        ordering: Order,
        remote: Counterparty,
        connection_hops: Vec<ConnectionId>,
        version: Version,
    ) -> Result<Self, ChannelError> {
        let channel_end =
            Self::new_without_validation(state, ordering, remote, connection_hops, version);
        channel_end.validate_basic()?;
        Ok(channel_end)
    }

    /// Updates the ChannelEnd to assume a new State 's'.
    pub fn set_state(&mut self, s: State) {
        self.state = s;
    }

    pub fn set_version(&mut self, v: Version) {
        self.version = v;
    }

    pub fn set_counterparty_channel_id(&mut self, c: ChannelId) {
        self.remote.channel_id = Some(c);
    }

    /// Returns `true` if this `ChannelEnd` is in state [`State::Open`].
    pub fn is_open(&self) -> bool {
        self.state == State::Open
    }

    pub fn state(&self) -> &State {
        &self.state
    }

    pub fn ordering(&self) -> &Order {
        &self.ordering
    }

    pub fn counterparty(&self) -> &Counterparty {
        &self.remote
    }

    pub fn connection_hops(&self) -> &Vec<ConnectionId> {
        &self.connection_hops
    }

    pub fn version(&self) -> &Version {
        &self.version
    }

    pub fn validate_basic(&self) -> Result<(), ChannelError> {
        if self.state == State::Uninitialized {
            return Err(ChannelError::InvalidState {
                expected: "Channel state cannot be Uninitialized".to_string(),
                actual: self.state.to_string(),
            });
        }

        if self.ordering == Order::None {
            return Err(ChannelError::InvalidOrderType {
                expected: "Channel ordering cannot be None".to_string(),
                actual: self.ordering.to_string(),
            });
        }

        Ok(())
    }

    /// Checks if the state of this channel end matches the expected state.
    pub fn verify_state_matches(&self, expected: &State) -> Result<(), ChannelError> {
        if !self.state.eq(expected) {
            return Err(ChannelError::InvalidState {
                expected: expected.to_string(),
                actual: self.state.to_string(),
            });
        }
        Ok(())
    }

    /// Checks if the state of this channel end is not closed.
    pub fn verify_not_closed(&self) -> Result<(), ChannelError> {
        if self.state.eq(&State::Closed) {
            return Err(ChannelError::InvalidState {
                expected: "Channel state cannot be Closed".to_string(),
                actual: self.state.to_string(),
            });
        }
        Ok(())
    }

    /// Helper function to compare the order of this end with another order.
    #[deprecated(
        since = "0.50.1",
        note = "Use `Eq` or `match` directly on the `Order` enum instead"
    )]
    pub fn order_matches(&self, other: &Order) -> bool {
        self.ordering.eq(other)
    }

    pub fn connection_hops_matches(&self, other: &Vec<ConnectionId>) -> bool {
        self.connection_hops.eq(other)
    }

    /// Checks if the counterparty of this channel end matches with an expected counterparty.
    pub fn verify_counterparty_matches(&self, expected: &Counterparty) -> Result<(), ChannelError> {
        if !self.counterparty().eq(expected) {
            return Err(ChannelError::InvalidCounterparty {
                expected: expected.clone(),
                actual: self.counterparty().clone(),
            });
        }
        Ok(())
    }

    /// Checks if the `connection_hops` has a length of `expected`.
    ///
    /// Note: Current IBC version only supports one connection hop.
    pub fn verify_connection_hops_length(&self) -> Result<(), ChannelError> {
        verify_connection_hops_length(&self.connection_hops, 1)
    }

    pub fn version_matches(&self, other: &Version) -> bool {
        self.version().eq(other)
    }
}

/// Checks if the `connection_hops` has a length of `expected`.
pub(crate) fn verify_connection_hops_length(
    connection_hops: &[ConnectionId],
    expected: u64,
) -> Result<(), ChannelError> {
    if connection_hops.len() as u64 != expected {
        return Err(ChannelError::InvalidConnectionHopsLength {
            expected,
            actual: connection_hops.len() as u64,
        });
    }
    Ok(())
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
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Counterparty {
    pub port_id: PortId,
    pub channel_id: Option<ChannelId>,
}

impl Counterparty {
    pub fn new(port_id: PortId, channel_id: Option<ChannelId>) -> Self {
        Self {
            port_id,
            channel_id,
        }
    }

    pub fn port_id(&self) -> &PortId {
        &self.port_id
    }

    pub fn channel_id(&self) -> Option<&ChannelId> {
        self.channel_id.as_ref()
    }

    /// Called upon initiating a channel handshake on the host chain to verify
    /// that the counterparty channel id has not been set.
    pub(crate) fn verify_empty_channel_id(&self) -> Result<(), ChannelError> {
        if self.channel_id().is_some() {
            return Err(ChannelError::InvalidChannelId {
                expected: "Counterparty channel id must be empty".to_string(),
                actual: format!("{:?}", self.channel_id),
            });
        }
        Ok(())
    }
}

impl Display for Counterparty {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        match &self.channel_id {
            Some(channel_id) => write!(
                f,
                "Counterparty(port_id: {}, channel_id: {})",
                self.port_id, channel_id
            ),
            None => write!(
                f,
                "Counterparty(port_id: {}, channel_id: None)",
                self.port_id
            ),
        }
    }
}

impl Protobuf<RawCounterparty> for Counterparty {}

impl TryFrom<RawCounterparty> for Counterparty {
    type Error = ChannelError;

    fn try_from(raw_counterparty: RawCounterparty) -> Result<Self, Self::Error> {
        let channel_id: Option<ChannelId> = if raw_counterparty.channel_id.is_empty() {
            None
        } else {
            Some(raw_counterparty.channel_id.parse()?)
        };

        Ok(Counterparty::new(
            raw_counterparty.port_id.parse()?,
            channel_id,
        ))
    }
}

impl From<Counterparty> for RawCounterparty {
    fn from(value: Counterparty) -> Self {
        RawCounterparty {
            port_id: value.port_id.as_str().to_string(),
            channel_id: value
                .channel_id
                .map_or_else(|| "".to_string(), |v| v.to_string()),
        }
    }
}

/// Represents the channel ordering
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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Order {
    None = 0isize,
    Unordered = 1isize,
    Ordered = 2isize,
}

impl Display for Order {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        write!(f, "{}", self.as_str())
    }
}

impl Order {
    /// Yields the Order as a string
    pub fn as_str(&self) -> &'static str {
        match self {
            // Note: taken from [ibc-go](https://github.com/cosmos/ibc-go/blob/e3a32a61098d463cd00b8937e18cb671bd20c6b7/modules/core/04-channel/types/channel.pb.go#L95-L97)
            Self::None => "ORDER_NONE_UNSPECIFIED",
            Self::Unordered => "ORDER_UNORDERED",
            Self::Ordered => "ORDER_ORDERED",
        }
    }

    // Parses the Order out from a i32.
    pub fn from_i32(nr: i32) -> Result<Self, ChannelError> {
        match nr {
            0 => Ok(Self::None),
            1 => Ok(Self::Unordered),
            2 => Ok(Self::Ordered),
            _ => Err(ChannelError::InvalidOrderType {
                expected: "Must be one of 0, 1, 2".to_string(),
                actual: nr.to_string(),
            }),
        }
    }
}

impl FromStr for Order {
    type Err = ChannelError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().trim_start_matches("order_") {
            "uninitialized" => Ok(Self::None),
            "unordered" => Ok(Self::Unordered),
            "ordered" => Ok(Self::Ordered),
            _ => Err(ChannelError::InvalidOrderType {
                expected: "Must be one of 'uninitialized', 'unordered', 'ordered'".to_string(),
                actual: s.to_string(),
            }),
        }
    }
}

/// Represents the state of a [`ChannelEnd`]
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
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum State {
    Uninitialized = 0isize,
    Init = 1isize,
    TryOpen = 2isize,
    Open = 3isize,
    Closed = 4isize,
}

impl State {
    /// Yields the state as a string
    pub fn as_string(&self) -> &'static str {
        match self {
            Self::Uninitialized => "UNINITIALIZED",
            Self::Init => "INIT",
            Self::TryOpen => "TRYOPEN",
            Self::Open => "OPEN",
            Self::Closed => "CLOSED",
        }
    }

    // Parses the State out from a i32.
    pub fn from_i32(s: i32) -> Result<Self, ChannelError> {
        match s {
            0 => Ok(Self::Uninitialized),
            1 => Ok(Self::Init),
            2 => Ok(Self::TryOpen),
            3 => Ok(Self::Open),
            4 => Ok(Self::Closed),
            _ => Err(ChannelError::InvalidState {
                expected: "Must be one of: 0, 1, 2, 3, 4".to_string(),
                actual: s.to_string(),
            }),
        }
    }

    /// Returns whether or not this channel state is `Open`.
    pub fn is_open(self) -> bool {
        self == State::Open
    }

    /// Returns whether or not the channel with this state
    /// has progressed less or the same than the argument.
    ///
    /// # Example
    /// ```rust,ignore
    /// assert!(State::Init.less_or_equal_progress(State::Open));
    /// assert!(State::TryOpen.less_or_equal_progress(State::TryOpen));
    /// assert!(!State::Closed.less_or_equal_progress(State::Open));
    /// ```
    pub fn less_or_equal_progress(self, other: Self) -> bool {
        self as u32 <= other as u32
    }
}

/// Provides a `to_string` method.
impl Display for State {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        write!(f, "{}", self.as_string())
    }
}
