use crate::prelude::*;
use displaydoc::Display;

#[derive(Debug, Display)]
pub enum HostError {
    /// Missing the host height
    MissingHeight,
    /// Missing the host timestamp
    MissingTimestamp,
    /// Missing the host (self) consensus state at the given height
    MissingSelfConsensusState { height: String },
    /// Invalid client state of the host (self) on the counterparty chain
    InvalidSelfClientState { reason: String },
}

#[cfg(feature = "std")]
impl std::error::Error for HostError {}

#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[derive(Debug, Display)]
pub enum ValidationError {
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
    /// identifier cannot be empty
    Empty,
    /// Invalid channel id in counterparty
    InvalidCounterpartyChannelId,
}

#[cfg(feature = "std")]
impl std::error::Error for ValidationError {}
