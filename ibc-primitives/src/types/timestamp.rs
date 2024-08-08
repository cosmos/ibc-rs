//! Defines the representation of timestamps used in IBC.

use core::fmt::{Display, Error as FmtError, Formatter};
use core::hash::Hash;
use core::num::{ParseIntError, TryFromIntError};
use core::ops::{Add, Sub};
use core::str::FromStr;
use core::time::Duration;

use displaydoc::Display;
use ibc_proto::google::protobuf::Timestamp as RawTimestamp;
use ibc_proto::Protobuf;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use tendermint::Time;
use time::error::ComponentRange;
use time::macros::offset;
use time::{OffsetDateTime, PrimitiveDateTime};

use crate::prelude::*;

pub const ZERO_DURATION: Duration = Duration::from_secs(0);

/// A new type wrapper over `PrimitiveDateTime` with extended capabilities to
/// keep track of host timestamps.
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(PartialEq, Eq, Copy, Clone, Debug, PartialOrd, Ord, Hash)]
pub struct Timestamp {
    // Note: The schema representation is the timestamp in nanoseconds (as we do with borsh).
    #[cfg_attr(feature = "schema", schemars(with = "u64"))]
    time: PrimitiveDateTime,
}

impl Timestamp {
    pub fn from_nanoseconds(nanoseconds: u64) -> Result<Self, TimestampError> {
        // As the `u64` representation can only represent times up to
        // about year 2554, there is no risk of overflowing `Time`
        // or `OffsetDateTime`.
        Self::try_from(nanoseconds)
    }

    pub fn from_unix_timestamp(secs: u64, nanos: u32) -> Result<Self, TimestampError> {
        if nanos > 999_999_999 {
            return Err(TimestampError::DateOutOfRange);
        }

        let total_nanos = secs as i128 * 1_000_000_000 + nanos as i128;

        let odt = OffsetDateTime::from_unix_timestamp_nanos(total_nanos)?;

        Self::from_utc(odt)
    }

    /// Internal helper to produce a `Timestamp` value validated with regard to
    /// the date range allowed in protobuf timestamps. The source
    /// `OffsetDateTime` value must have the zero UTC offset.
    fn from_utc(t: OffsetDateTime) -> Result<Self, TimestampError> {
        debug_assert_eq!(t.offset(), offset!(UTC));
        match t.year() {
            1..=9999 => Ok(Self {
                time: PrimitiveDateTime::new(t.date(), t.time()),
            }),
            _ => Err(TimestampError::DateOutOfRange),
        }
    }

    /// Returns a `Timestamp` representation of the current time.
    #[cfg(feature = "std")]
    pub fn now() -> Self {
        OffsetDateTime::now_utc()
            .try_into()
            .expect("now is in the range of 0..=9999 years")
    }

    /// Computes the duration difference of another `Timestamp` from the current
    /// one. Returns the difference in time as an [`core::time::Duration`].
    pub fn duration_since(&self, other: &Self) -> Option<Duration> {
        let duration = self.time.assume_utc() - other.time.assume_utc();
        duration.try_into().ok()
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
    /// ```
    pub fn nanoseconds(self) -> u64 {
        let odt: OffsetDateTime = self.into();
        let s = odt.unix_timestamp_nanos();
        s.try_into()
            .expect("Fails UNIX timestamp is negative, but we don't allow that to be constructed")
    }

    pub fn into_tm_time(self) -> Time {
        Time::try_from(self.time.assume_offset(offset!(UTC)))
            .expect("Timestamp is in the range of 0..=9999 years")
    }
}

impl Protobuf<RawTimestamp> for Timestamp {}

impl TryFrom<RawTimestamp> for Timestamp {
    type Error = TimestampError;

    fn try_from(raw: RawTimestamp) -> Result<Self, Self::Error> {
        let nanos = raw.nanos.try_into()?;
        let seconds = raw.seconds.try_into()?;
        Self::from_unix_timestamp(seconds, nanos)
    }
}

impl From<Timestamp> for RawTimestamp {
    fn from(value: Timestamp) -> Self {
        let t = value.time.assume_utc();
        let seconds = t.unix_timestamp();
        // Safe to convert to i32 because .nanosecond()
        // is guaranteed to return a value in 0..1_000_000_000 range.
        let nanos = t.nanosecond() as i32;
        RawTimestamp { seconds, nanos }
    }
}

impl TryFrom<OffsetDateTime> for Timestamp {
    type Error = TimestampError;

    fn try_from(t: OffsetDateTime) -> Result<Self, Self::Error> {
        Self::from_utc(t.to_offset(offset!(UTC)))
    }
}

impl From<Timestamp> for OffsetDateTime {
    fn from(t: Timestamp) -> Self {
        t.time.assume_utc()
    }
}

impl TryFrom<u64> for Timestamp {
    type Error = TimestampError;

    fn try_from(nanoseconds: u64) -> Result<Self, Self::Error> {
        let odt = OffsetDateTime::from_unix_timestamp_nanos(nanoseconds.into())?;
        Self::from_utc(odt)
    }
}

impl FromStr for Timestamp {
    type Err = TimestampError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let nanoseconds = u64::from_str(s)?;
        Self::try_from(nanoseconds)
    }
}

impl Display for Timestamp {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        write!(f, "Timestamp({})", self.time)
    }
}

impl Add<Duration> for Timestamp {
    type Output = Result<Self, TimestampError>;

    fn add(self, rhs: Duration) -> Self::Output {
        let duration = rhs.try_into().map_err(|_| TimestampError::DateOutOfRange)?;
        let t = self
            .time
            .checked_add(duration)
            .ok_or(TimestampError::DateOutOfRange)?;
        Self::from_utc(t.assume_utc())
    }
}

impl Sub<Duration> for Timestamp {
    type Output = Result<Self, TimestampError>;

    fn sub(self, rhs: Duration) -> Self::Output {
        let duration = rhs.try_into().map_err(|_| TimestampError::DateOutOfRange)?;
        let t = self
            .time
            .checked_sub(duration)
            .ok_or(TimestampError::DateOutOfRange)?;

        if t.assume_utc().unix_timestamp() < 0 {
            return Err(TimestampError::DateOutOfRange);
        }
        Self::from_utc(t.assume_utc())
    }
}

impl TryFrom<Time> for Timestamp {
    type Error = TimestampError;

    fn try_from(tm_time: Time) -> Result<Self, Self::Error> {
        let odt: OffsetDateTime = tm_time.into();
        if odt.unix_timestamp() < 0 {
            return Err(TimestampError::DateOutOfRange);
        }
        odt.try_into()
    }
}

#[cfg(feature = "serde")]
impl Serialize for Timestamp {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.nanoseconds().serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for Timestamp {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let timestamp = u64::deserialize(deserializer)?;
        Timestamp::from_nanoseconds(timestamp).map_err(serde::de::Error::custom)
    }
}

#[cfg(feature = "borsh")]
impl borsh::BorshSerialize for Timestamp {
    fn serialize<W: borsh::io::Write>(&self, writer: &mut W) -> borsh::io::Result<()> {
        let timestamp = self.nanoseconds();
        borsh::BorshSerialize::serialize(&timestamp, writer)
    }
}

#[cfg(feature = "borsh")]
impl borsh::BorshDeserialize for Timestamp {
    fn deserialize_reader<R: borsh::io::Read>(reader: &mut R) -> borsh::io::Result<Self> {
        let timestamp = u64::deserialize_reader(reader)?;
        Ok(Self::from_nanoseconds(timestamp).map_err(|_| borsh::io::ErrorKind::Other)?)
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

#[derive(Debug, Display, derive_more::From)]
pub enum TimestampError {
    /// parsing u64 integer from string error: `{0}`
    ParseInt(ParseIntError),
    /// error converting integer to `Timestamp`: `{0}`
    TryFromInt(TryFromIntError),
    /// date out of range
    DateOutOfRange,
    /// Timestamp overflow when modifying with duration
    TimestampOverflow,
    /// Timestamp is not set
    Conversion(ComponentRange),
}

#[cfg(feature = "std")]
impl std::error::Error for TimestampError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self {
            Self::ParseInt(e) => Some(e),
            Self::TryFromInt(e) => Some(e),
            Self::Conversion(e) => Some(e),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use core::time::Duration;
    use std::thread::sleep;

    use rstest::rstest;
    use time::OffsetDateTime;

    use super::{Timestamp, ZERO_DURATION};

    #[rstest]
    #[case::zero(0)]
    #[case::one(1)]
    #[case::billions(1_000_000_000)]
    #[case::u64_max(u64::MAX)]
    #[case::i64_max(i64::MAX.try_into().unwrap())]
    fn test_timestamp_from_nanoseconds(#[case] nanos: u64) {
        let timestamp = Timestamp::from_nanoseconds(nanos).unwrap();
        let dt: OffsetDateTime = timestamp.into();
        assert_eq!(dt.unix_timestamp_nanos(), nanos as i128);
        assert_eq!(timestamp.nanoseconds(), nanos);
    }

    #[rstest]
    #[case::one(1, 0)]
    #[case::billions(1_000_000_000, 0)]
    #[case::u64_max(u64::MAX, 0)]
    #[case::u64_from_i62_max(u64::MAX, i64::MAX.try_into().unwrap())]
    fn test_timestamp_comparisons(#[case] nanos_1: u64, #[case] nanos_2: u64) {
        let t_1 = Timestamp::from_nanoseconds(nanos_1).unwrap();
        let t_2 = Timestamp::from_nanoseconds(nanos_2).unwrap();
        assert!(t_1 > t_2);
    }

    #[test]
    fn test_timestamp_arithmetic() {
        let time0 = Timestamp::from_nanoseconds(0).unwrap();
        let time1 = Timestamp::from_nanoseconds(100).unwrap();
        let time2 = Timestamp::from_nanoseconds(150).unwrap();
        let time3 = Timestamp::from_nanoseconds(50).unwrap();
        let duration = Duration::from_nanos(50);

        assert_eq!(time1, (time1 + ZERO_DURATION).unwrap());
        assert_eq!(time2, (time1 + duration).unwrap());
        assert_eq!(time3, (time1 - duration).unwrap());
        assert_eq!(time0, (time3 - duration).unwrap());
        assert!((time0 - duration).is_err());
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

    #[cfg(feature = "serde")]
    #[rstest]
    #[case::zero(0)]
    #[case::one(1)]
    #[case::billions(1_000_000_000)]
    #[case::u64_max(u64::MAX)]
    fn test_timestamp_serde(#[case] nanos: u64) {
        let timestamp = Timestamp::from_nanoseconds(nanos).unwrap();
        let serialized = serde_json::to_string(&timestamp).unwrap();
        let deserialized = serde_json::from_str::<Timestamp>(&serialized).unwrap();
        assert_eq!(timestamp, deserialized);
    }

    #[test]
    #[cfg(feature = "borsh")]
    fn test_timestamp_borsh_ser_der() {
        let timestamp = Timestamp::now();
        let encode_timestamp = borsh::to_vec(&timestamp).unwrap();
        let _ = borsh::from_slice::<Timestamp>(&encode_timestamp).unwrap();
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
