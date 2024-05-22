use ibc::primitives::Timestamp;
use tendermint::Time;

/// Returns a `Timestamp` representation of the beginning of year 2023.
///
/// This is introduced to initialize [`StoreGenericTestContext`](crate::context::StoreGenericTestContext)s
/// with the same latest timestamp by default.
/// If two [`StoreGenericTestContext`](crate::context::StoreGenericTestContext)
/// are initialized using [`Time::now()`], the second one will have a greater timestamp than the first one.
/// So, the latest header of the second context cannot be submitted to first one.
/// We can still set a custom timestamp via [`TestContextConfig`](crate::fixtures::core::context::TestContextConfig).
pub fn year_2023() -> Timestamp {
    // Sun Jan 01 2023 00:00:00 GMT+0000
    Time::from_unix_timestamp(1_672_531_200, 0)
        .expect("should be a valid time")
        .into()
}
