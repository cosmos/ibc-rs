use core::time::Duration;

use ibc::primitives::Timestamp;
use ibc_proto::google::protobuf::{Duration as GDuration, Timestamp as GTimestamp};

pub fn timestamp_gpb_to_ibc(gpb_timestamp: GTimestamp) -> Timestamp {
    let GTimestamp { seconds, nanos } = gpb_timestamp;
    Timestamp::from_nanoseconds(seconds as u64 * 1_000_000_000 + nanos as u64)
        .expect("no hmm overflow")
}

pub fn timestamp_ibc_to_gpb(ibc_timestamp: Timestamp) -> GTimestamp {
    let tendermint_proto::google::protobuf::Timestamp { seconds, nanos } = ibc_timestamp
        .into_tm_time()
        .unwrap_or_else(|| tendermint::Time::from_unix_timestamp(0, 0).expect("no overflow"))
        .into();
    GTimestamp { seconds, nanos }
}

pub fn duration_gpb_to_ibc(gbp_duration: GDuration) -> Duration {
    let GDuration { seconds, nanos } = gbp_duration;
    Duration::from_nanos(seconds as u64 * 1_000_000_000 + nanos as u64)
}

pub fn duration_ibc_to_gbp(ibc_duration: Duration) -> GDuration {
    GDuration {
        seconds: ibc_duration.as_secs() as i64,
        nanos: ibc_duration.subsec_nanos() as i32,
    }
}
