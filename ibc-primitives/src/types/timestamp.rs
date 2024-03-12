//! Defines the representation of timestamps used in IBC.

use core::fmt::{Display, Error as FmtError, Formatter};
use core::hash::Hash;
use core::num::ParseIntError;
use core::ops::{Add, Sub};
use core::str::FromStr;
use core::time::Duration;

use displaydoc::Display;
use tendermint::Time;
use time::OffsetDateTime;

use crate::prelude::*;

pub const ZERO_DURATION: Duration = Duration::from_secs(0);

/// A new type wrapper over `Option<Time>` to keep track of
/// IBC packet timeout.
///
/// We use an explicit `Option` type to distinguish this when converting between
/// a `u64` value and a raw timestamp. In protocol buffer, the timestamp is
/// represented as a `u64` Unix timestamp in nanoseconds, with 0 representing the absence
/// of timestamp.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(PartialEq, Eq, Copy, Clone, Debug, PartialOrd, Ord, Hash)]
pub struct Timestamp {
    // Note: The schema representation is the timestamp in nanoseconds (as we do with borsh).
    #[cfg_attr(feature = "schema", schemars(with = "u64"))]
    time: Option<Time>,
}

#[cfg(feature = "borsh")]
impl borsh::BorshSerialize for Timestamp {
    fn serialize<W: borsh::maybestd::io::Write>(
        &self,
        writer: &mut W,
    ) -> borsh::maybestd::io::Result<()> {
        let timestamp = self.nanoseconds();
        borsh::BorshSerialize::serialize(&timestamp, writer)
    }
}

#[cfg(feature = "borsh")]
impl borsh::BorshDeserialize for Timestamp {
    fn deserialize_reader<R: borsh::maybestd::io::Read>(
        reader: &mut R,
    ) -> borsh::maybestd::io::Result<Self> {
        let timestamp = u64::deserialize_reader(reader)?;
        Ok(Self::from_nanoseconds(timestamp).map_err(|_| borsh::maybestd::io::ErrorKind::Other)?)
    }
}

#[cfg(feature = "parity-scale-codec")]
impl parity_scale_codec::Encode for Timestamp {
    fn encode_to<T: parity_scale_codec::Output + ?Sized>(&self, writer: &mut T) {
        let timestamp = self.nanoseconds();
        timestamp.encode_to(writer);
    }
}
#[cfg(feature = "parity-scale-codec")]
impl parity_scale_codec::Decode for Timestamp {
    fn decode<I: parity_scale_codec::Input>(
        input: &mut I,
    ) -> Result<Self, parity_scale_codec::Error> {
        let timestamp = u64::decode(input)?;
        Self::from_nanoseconds(timestamp)
            .map_err(|_| parity_scale_codec::Error::from("from nanoseconds error"))
    }
}

#[cfg(feature = "parity-scale-codec")]
impl scale_info::TypeInfo for Timestamp {
    type Identity = Self;

    fn type_info() -> scale_info::Type {
        scale_info::Type::builder()
            .path(scale_info::Path::new("Timestamp", module_path!()))
            .composite(
                scale_info::build::Fields::named()
                    .field(|f| f.ty::<u64>().name("time").type_name("u64")),
            )
    }
}

/// The expiry result when comparing two timestamps.
/// - If either timestamp is invalid (0), the result is `InvalidTimestamp`.
/// - If the left timestamp is strictly after the right timestamp, the result is `Expired`.
/// - Otherwise, the result is `NotExpired`.
///
/// User of this result may want to determine whether error should be raised,
/// when either of the timestamp being compared is invalid.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(PartialEq, Eq, Copy, Clone, Debug, Hash)]
pub enum Expiry {
    Expired,
    NotExpired,
    InvalidTimestamp,
}

impl Timestamp {
    /// The IBC protocol represents timestamps as u64 Unix
    /// timestamps in nanoseconds.
    ///
    /// A protocol value of 0 indicates that the timestamp
    /// is not set. In this case, our domain type takes the
    /// value of None.
    ///
    pub fn from_nanoseconds(nanoseconds: u64) -> Result<Self, ParseTimestampError> {
        if nanoseconds == 0 {
            Ok(Self { time: None })
        } else {
            // As the `u64` representation can only represent times up to
            // about year 2554, there is no risk of overflowing `Time`
            // or `OffsetDateTime`.
            let ts = OffsetDateTime::from_unix_timestamp_nanos(nanoseconds.into())
                .map_err(|e: time::error::ComponentRange| {
                    ParseTimestampError::DataOutOfRange(e.to_string())
                })?
                .try_into()
                .map_err(|e: tendermint::error::Error| {
                    ParseTimestampError::DataOutOfRange(e.to_string())
                })?;
            Ok(Self { time: Some(ts) })
        }
    }

    /// Returns a `Timestamp` representation of the current time.
    #[cfg(feature = "std")]
    pub fn now() -> Self {
        Time::now().into()
    }

    /// Returns a `Timestamp` representation of a timestamp not being set.
    pub fn none() -> Self {
        Self { time: None }
    }

    /// Computes the duration difference of another `Timestamp` from the current one.
    /// Returns the difference in time as an [`core::time::Duration`].
    /// Returns `None` if the other `Timestamp` is more advanced
    /// than the current or if either of the `Timestamp`s is not set.
    pub fn duration_since(&self, other: &Self) -> Option<Duration> {
        match (self.time, other.time) {
            (Some(time1), Some(time2)) => time1.duration_since(time2).ok(),
            _ => None,
        }
    }

    /// Convert a `Timestamp` to `u64` value in nanoseconds. If no timestamp
    /// is set, the result is 0.
    ///
    #[deprecated(since = "0.9.1", note = "use `nanoseconds` instead")]
    pub fn as_nanoseconds(&self) -> u64 {
        (*self).nanoseconds()
    }

    /// Convert a `Timestamp` to `u64` value in nanoseconds. If no timestamp
    /// is set, the result is 0.
    /// ```
    /// use ibc_primitives::Timestamp;
    ///
    /// let max = u64::MAX;
    /// let tx = Timestamp::from_nanoseconds(max).unwrap();
    /// let utx = tx.nanoseconds();
    /// assert_eq!(utx, max);
    /// let min = u64::MIN;
    /// let ti = Timestamp::from_nanoseconds(min).unwrap();
    /// let uti = ti.nanoseconds();
    /// assert_eq!(uti, min);
    /// let tz = Timestamp::none();
    /// let utz = tz.nanoseconds();
    /// assert_eq!(utz, 0);
    /// ```
    pub fn nanoseconds(self) -> u64 {
        self.time.map_or(0, |time| {
            let t: OffsetDateTime = time.into();
            let s = t.unix_timestamp_nanos();
            assert!(s >= 0, "time {time:?} has negative `.timestamp()`");
            s.try_into().expect(
                "Fails UNIX timestamp is negative, but we don't allow that to be constructed",
            )
        })
    }

    /// Convert a `Timestamp` to an optional [`OffsetDateTime`]
    pub fn into_datetime(self) -> Option<OffsetDateTime> {
        self.time.map(Into::into)
    }

    /// Convert a `Timestamp` to an optional [`tendermint::Time`]
    pub fn into_tm_time(self) -> Option<Time> {
        self.time
    }

    /// Checks whether the timestamp has expired when compared to the
    /// `other` timestamp. Returns an [`Expiry`] result.
    pub fn check_expiry(&self, other: &Self) -> Expiry {
        match (self.time, other.time) {
            (Some(time1), Some(time2)) => {
                if time1 > time2 {
                    Expiry::Expired
                } else {
                    Expiry::NotExpired
                }
            }
            _ => Expiry::InvalidTimestamp,
        }
    }

    /// Checks whether the timestamp is set.
    pub fn is_set(&self) -> bool {
        self.time.is_some()
    }
}

impl Display for Timestamp {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        write!(
            f,
            "Timestamp({})",
            self.time
                .map_or("NoTimestamp".to_string(), |time| time.to_rfc3339())
        )
    }
}

#[derive(Debug, Display)]
pub enum TimestampOverflowError {
    /// Timestamp overflow when modifying with duration
    TimestampOverflow,
}

#[cfg(feature = "std")]
impl std::error::Error for TimestampOverflowError {}

impl Add<Duration> for Timestamp {
    type Output = Result<Self, TimestampOverflowError>;

    fn add(self, duration: Duration) -> Result<Self, TimestampOverflowError> {
        self.time
            .map(|time| time + duration)
            .transpose()
            .map(|time| Self { time })
            .map_err(|_| TimestampOverflowError::TimestampOverflow)
    }
}

impl Sub<Duration> for Timestamp {
    type Output = Result<Self, TimestampOverflowError>;

    fn sub(self, duration: Duration) -> Result<Self, TimestampOverflowError> {
        self.time
            .map(|time| time - duration)
            .transpose()
            .map(|time| Self { time })
            .map_err(|_| TimestampOverflowError::TimestampOverflow)
    }
}

#[derive(Debug, Display)]
pub enum ParseTimestampError {
    /// parsing u64 integer from string error: `{0}`
    ParseInt(ParseIntError),
    /// Out of Range: `{0}`
    DataOutOfRange(String),
}

#[cfg(feature = "std")]
impl std::error::Error for ParseTimestampError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self {
            Self::ParseInt(e) => Some(e),
            Self::DataOutOfRange(_) => None,
        }
    }
}

impl FromStr for Timestamp {
    type Err = ParseTimestampError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let nanoseconds = u64::from_str(s).map_err(ParseTimestampError::ParseInt)?;

        Self::from_nanoseconds(nanoseconds)
    }
}

impl From<Time> for Timestamp {
    fn from(tendermint_time: Time) -> Self {
        Self {
            time: Some(tendermint_time),
        }
    }
}

#[cfg(test)]
mod tests {
    use core::time::Duration;
    use std::thread::sleep;

    use time::OffsetDateTime;

    use super::{Expiry, Timestamp, ZERO_DURATION};

    #[test]
    fn test_timestamp_comparisons() {
        let nil_timestamp = Timestamp::from_nanoseconds(0).unwrap();
        assert_eq!(nil_timestamp.time, None);
        assert_eq!(nil_timestamp.nanoseconds(), 0);

        let timestamp1 = Timestamp::from_nanoseconds(1).unwrap();
        let dt: OffsetDateTime = timestamp1.time.unwrap().into();
        assert_eq!(dt.unix_timestamp_nanos(), 1);
        assert_eq!(timestamp1.nanoseconds(), 1);

        let timestamp2 = Timestamp::from_nanoseconds(1_000_000_000).unwrap();
        let dt: OffsetDateTime = timestamp2.time.unwrap().into();
        assert_eq!(dt.unix_timestamp_nanos(), 1_000_000_000);
        assert_eq!(timestamp2.nanoseconds(), 1_000_000_000);

        assert!(Timestamp::from_nanoseconds(u64::MAX).is_ok());
        assert!(Timestamp::from_nanoseconds(i64::MAX.try_into().unwrap()).is_ok());

        assert_eq!(timestamp1.check_expiry(&timestamp2), Expiry::NotExpired);
        assert_eq!(timestamp1.check_expiry(&timestamp1), Expiry::NotExpired);
        assert_eq!(timestamp2.check_expiry(&timestamp2), Expiry::NotExpired);
        assert_eq!(timestamp2.check_expiry(&timestamp1), Expiry::Expired);
        assert_eq!(
            timestamp1.check_expiry(&nil_timestamp),
            Expiry::InvalidTimestamp
        );
        assert_eq!(
            nil_timestamp.check_expiry(&timestamp2),
            Expiry::InvalidTimestamp
        );
        assert_eq!(
            nil_timestamp.check_expiry(&nil_timestamp),
            Expiry::InvalidTimestamp
        );
    }

    #[test]
    fn test_timestamp_arithmetic() {
        let time0 = Timestamp::none();
        let time1 = Timestamp::from_nanoseconds(100).unwrap();
        let time2 = Timestamp::from_nanoseconds(150).unwrap();
        let time3 = Timestamp::from_nanoseconds(50).unwrap();
        let duration = Duration::from_nanos(50);

        assert_eq!(time1, (time1 + ZERO_DURATION).unwrap());
        assert_eq!(time2, (time1 + duration).unwrap());
        assert_eq!(time3, (time1 - duration).unwrap());
        assert_eq!(time0, (time0 + duration).unwrap());
        assert_eq!(time0, (time0 - duration).unwrap());
    }

    #[test]
    fn subtract_compare() {
        let sleep_duration = Duration::from_micros(100);

        let start = Timestamp::now();
        sleep(sleep_duration);
        let end = Timestamp::now();

        let res = end.duration_since(&start);
        assert!(res.is_some());

        let inner = res.unwrap();
        assert!(inner > sleep_duration);
    }

    #[test]
    #[cfg(feature = "borsh")]
    fn test_timestamp_borsh_ser_der() {
        use borsh::{BorshDeserialize, BorshSerialize};
        let timestamp = Timestamp::now();
        let encode_timestamp = timestamp.try_to_vec().unwrap();
        let _ = Timestamp::try_from_slice(&encode_timestamp).unwrap();
    }

    #[test]
    #[cfg(feature = "parity-scale-codec")]
    fn test_timestamp_parity_scale_codec_ser_der() {
        use parity_scale_codec::{Decode, Encode};
        let timestamp = Timestamp::now();
        let encode_timestamp = timestamp.encode();
        let _ = Timestamp::decode(&mut encode_timestamp.as_slice()).unwrap();
    }
}
