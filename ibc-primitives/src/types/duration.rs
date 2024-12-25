   use core::time::Duration;
   use ibc_proto::google::protobuf::Duration as RawDuration;
   use crate::prelude::*;

   /// Converts a core::time::Duration into a protobuf Duration
   pub fn duration_to_proto(d: Duration) -> Option<RawDuration> {
       let seconds = i64::try_from(d.as_secs()).ok()?;
       let nanos = i32::try_from(d.subsec_nanos()).ok()?;
       Some(RawDuration { seconds, nanos })
   }

   /// Converts a protobuf Duration into a core::time::Duration
   pub fn duration_from_proto(d: RawDuration) -> Option<Duration> {
       if d.seconds.is_negative() || d.nanos.is_negative() {
           return None;
       }
       let seconds = u64::try_from(d.seconds).ok()?;
       let nanos = u32::try_from(d.nanos).ok()?;
       Some(Duration::new(seconds, nanos))
   }

   #[cfg(test)]
   mod tests {
       use super::*;

       #[test]
       fn test_duration_conversions() {
           // Test positive durations
           let core_duration = Duration::new(5, 500_000_000);
           let proto = duration_to_proto(core_duration).unwrap();
           assert_eq!(proto.seconds, 5);
           assert_eq!(proto.nanos, 500_000_000);
           let converted_back = duration_from_proto(proto).unwrap();
           assert_eq!(converted_back, core_duration);

           // Test zero duration
           let zero_duration = Duration::new(0, 0);
           let proto = duration_to_proto(zero_duration).unwrap();
           assert_eq!(proto.seconds, 0);
           assert_eq!(proto.nanos, 0);
           let converted_back = duration_from_proto(proto).unwrap();
           assert_eq!(converted_back, zero_duration);

           // Test negative duration (should return None)
           let negative_proto = RawDuration {
               seconds: -1,
               nanos: -500_000_000,
           };
           assert!(duration_from_proto(negative_proto).is_none());
       }
   }
