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
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub enum TimeoutTimestamp {
    Never,
    At(Timestamp),
}

impl TimeoutTimestamp {
    /// Creates a new timeout timestamp from a given nanosecond value.
    pub fn from_nanoseconds(nanoseconds: u64) -> Self {
        Self::from(nanoseconds)
    }

    /// Returns the timestamp in nanoseconds, where 0 indicates the absence
    /// of a timeout.
    pub fn nanoseconds(&self) -> u64 {
        match self {
            Self::At(timestamp) => timestamp.nanoseconds(),
            Self::Never => 0,
        }
    }

    /// Returns `true` if the timeout timestamp is set.
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

impl From<Timestamp> for TimeoutTimestamp {
    fn from(timestamp: Timestamp) -> Self {
        TimeoutTimestamp::At(timestamp)
    }
}

impl From<u64> for TimeoutTimestamp {
    fn from(timestamp: u64) -> Self {
        if timestamp == 0 {
            TimeoutTimestamp::Never
        } else {
            TimeoutTimestamp::At(Timestamp::from_nanoseconds(timestamp))
        }
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

#[cfg(test)]
mod tests {
    use rstest::rstest;

    fn never() -> TimeoutTimestamp {
        TimeoutTimestamp::Never
    }

    fn some(t: u64) -> TimeoutTimestamp {
        TimeoutTimestamp::At(Timestamp::from_nanoseconds(t))
    }

    use super::*;
    #[rstest]
    #[case::zero_plus_zero(some(0), 0, some(0))] // NOTE: zero timestamp is 1970-01-01T00:00:00Z
    #[case::zero_plus_one(some(0), 1, some(1))]
    #[case::some_plus_zero(some(123456), 0, some(123456))]
    #[case::some_plus_one(some(123456), 1, some(123457))]
    fn test_timeout_add_arithmetic(
        #[case] timeout_timestamp: TimeoutTimestamp,
        #[case] duration: u64,
        #[case] expect: TimeoutTimestamp,
    ) {
        let duration = Duration::from_nanos(duration);
        let result = timeout_timestamp + duration;
        assert_eq!(result.unwrap(), expect);
    }

    #[rstest]
    #[case::never(never(), 0)]
    #[case::never_plus_one(never(), 1)]
    fn test_invalid_timeout_add_arithmetic(
        #[case] timeout_timestamp: TimeoutTimestamp,
        #[case] duration: u64,
    ) {
        let duration = Duration::from_nanos(duration);
        let result = timeout_timestamp + duration;
        assert!(result.is_err());
    }

    #[rstest]
    #[case::zero_minus_zero(some(0), 0, some(0))]
    #[case::some_minus(some(123456), 123456, some(0))]
    #[case::some_minus_zero(some(123456), 0, some(123456))]
    #[case::some_minus_one(some(123456), 1, some(123455))]
    fn test_timeout_sub_arithmetic(
        #[case] timeout_timestamp: TimeoutTimestamp,
        #[case] duration: u64,
        #[case] expect: TimeoutTimestamp,
    ) {
        let duration = Duration::from_nanos(duration);
        let result = timeout_timestamp - duration;
        assert_eq!(result.unwrap(), expect);
    }

    #[rstest]
    #[case::never(never(), 0)]
    #[case::never_minus_one(never(), 1)]
    #[case::zero_minus_one(some(0), 1)]
    fn test_invalid_sub_arithmetic(
        #[case] timeout_timestamp: TimeoutTimestamp,
        #[case] duration: u64,
    ) {
        let duration = Duration::from_nanos(duration);
        let result = timeout_timestamp - duration;
        assert!(result.is_err());
    }

    #[cfg(feature = "serde")]
    #[rstest::rstest]
    #[case::never(TimeoutTimestamp::Never)]
    #[case::at_zero(TimeoutTimestamp::At(Timestamp::from_nanoseconds(0)))]
    #[case::at_some(TimeoutTimestamp::At(Timestamp::from_nanoseconds(123456)))]
    #[case::at_u64_max(TimeoutTimestamp::At(Timestamp::from_nanoseconds(u64::MAX)))]
    fn test_timeout_timestamp_serde(#[case] timeout_timestamp: TimeoutTimestamp) {
        let serialized = serde_json::to_string(&timeout_timestamp).unwrap();
        let deserialized: TimeoutTimestamp = serde_json::from_str(&serialized).unwrap();

        assert_eq!(timeout_timestamp, deserialized);
    }
}
