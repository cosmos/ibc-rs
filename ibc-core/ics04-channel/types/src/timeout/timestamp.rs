use core::fmt::{Display, Error as FmtError, Formatter};
use core::ops::{Add, Sub};
use core::time::Duration;

use ibc_primitives::prelude::*;
use ibc_primitives::Timestamp;

use crate::error::PacketError;

/// Indicates a timestamp on the destination chain after which the packet will
/// no longer be processed, and will instead count as having timed-out.
///
/// The IBC protocol represents timestamps as u64 Unix timestamps in
/// nanoseconds. A protocol value of 0 indicates that the timestamp is not set.
/// In this case, we use the explicit `Never` variant to distinguish the absence
/// of a timeout when converting from a zero protobuf value.
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
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
// #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub enum TimeoutTimestamp {
    Never,
    At(Timestamp),
}

impl TimeoutTimestamp {
    /// Creates a new timeout timestamp from a given nanosecond value.
    pub fn from_nanoseconds(nanoseconds: u64) -> Result<Self, PacketError> {
        Self::try_from(nanoseconds)
    }

    /// Returns the timestamp in nanoseconds, where 0 indicates that the absence
    /// of a timeout.
    pub fn nanoseconds(&self) -> u64 {
        match self {
            Self::At(timestamp) => timestamp.nanoseconds(),
            Self::Never => 0,
        }
    }

    /// Returns if the timeout timestamp is set.
    pub fn is_set(&self) -> bool {
        match self {
            TimeoutTimestamp::At(_) => true,
            TimeoutTimestamp::Never => false,
        }
    }

    /// Returns a timeout timestamp that never expires.
    pub fn no_timeout() -> Self {
        Self::Never
    }

    /// Check if a timestamp is *strictly past* the timeout timestamp, and thus
    /// is deemed expired.
    pub fn has_expired(&self, timestamp: &Timestamp) -> bool {
        match self {
            Self::At(timeout_timestamp) => timestamp > timeout_timestamp,
            // When there's no timeout, timestamps are never expired
            Self::Never => false,
        }
    }
}

impl TryFrom<u64> for TimeoutTimestamp {
    type Error = PacketError;
    fn try_from(timestamp: u64) -> Result<Self, Self::Error> {
        let timeout_timestamp = if timestamp == 0 {
            TimeoutTimestamp::Never
        } else {
            let timestamp = Timestamp::from_nanoseconds(timestamp)?;
            TimeoutTimestamp::At(timestamp)
        };

        Ok(timeout_timestamp)
    }
}

impl Display for TimeoutTimestamp {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        match self {
            TimeoutTimestamp::At(timeout_timestamp) => write!(f, "{timeout_timestamp}"),
            TimeoutTimestamp::Never => write!(f, "no timeout timestamp"),
        }
    }
}

impl Add<Duration> for TimeoutTimestamp {
    type Output = Result<Self, PacketError>;

    fn add(self, rhs: Duration) -> Self::Output {
        match self {
            TimeoutTimestamp::At(timestamp) => {
                let new_timestamp = timestamp.add(rhs)?;
                Ok(TimeoutTimestamp::At(new_timestamp))
            }
            TimeoutTimestamp::Never => Err(PacketError::MissingTimeout),
        }
    }
}

impl Sub<Duration> for TimeoutTimestamp {
    type Output = Result<Self, PacketError>;

    fn sub(self, rhs: Duration) -> Self::Output {
        match self {
            TimeoutTimestamp::At(timestamp) => {
                let new_timestamp = timestamp.sub(rhs)?;
                Ok(TimeoutTimestamp::At(new_timestamp))
            }
            TimeoutTimestamp::Never => Err(PacketError::MissingTimeout),
        }
    }
}

#[cfg(feature = "serde")]
mod serialize {
    use serde::{Deserialize, Serialize};

    use super::TimeoutTimestamp;

    impl Serialize for TimeoutTimestamp {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            match self {
                TimeoutTimestamp::At(timestamp) => timestamp.serialize(serializer),
                TimeoutTimestamp::Never => 0.serialize(serializer),
            }
        }
    }

    impl<'de> Deserialize<'de> for TimeoutTimestamp {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let timestamp = u64::deserialize(deserializer)?;
            TimeoutTimestamp::try_from(timestamp).map_err(serde::de::Error::custom)
        }
    }
}
