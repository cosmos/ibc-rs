use ibc::primitives::Timestamp;
use tendermint::Time;

/// Returns a `Timestamp` representation of beginning of year 2023.
pub fn year_2023() -> Timestamp {
    // Sun Jan 01 2023 00:00:00 GMT+0000
    Time::from_unix_timestamp(1_672_531_200, 0)
        .expect("should be a valid time")
        .into()
}
