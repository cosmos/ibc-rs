use ibc::primitives::Timestamp;
use tendermint::Time;

/// Returns a `Timestamp` representation of beginning of year 2023.
///
/// This is introduced to initialize [`MockGenericContext`](crate::context::MockGenericContext)s
/// with the same latest timestamp by default.
/// If two [`MockGenericContext`](crate::context::MockGenericContext)
/// are initialized using [`Time::now()`], second one will have a greater timestamp than the first one.
/// So, the latest header of the second context can not be submitted to first one.
/// We can still set a custom timestamp via [`TestContextConfig`](crate::fixtures::core::context::TestContextConfig).
pub fn year_2023() -> Timestamp {
    // TODO(rano): can we turn this into a fixture?

    // Sun Jan 01 2023 00:00:00 GMT+0000
    Time::from_unix_timestamp(1_672_531_200, 0)
        .expect("should be a valid time")
        .into()
}
