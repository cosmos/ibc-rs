use ibc::primitives::Timestamp;
#[cfg(feature = "serde")]
use serde::{de::DeserializeOwned, Serialize};
use tendermint::Time;

/// Returns a `Timestamp` representation of beginning of year 2023.
///
/// This is introduced to initialize [`StoreGenericTestContext`](crate::context::StoreGenericTestContext)s
/// with the same latest timestamp by default.
/// If two [`StoreGenericTestContext`](crate::context::StoreGenericTestContext)
/// are initialized using [`Time::now()`], second one will have a greater timestamp than the first one.
/// So, the latest header of the second context can not be submitted to first one.
/// We can still set a custom timestamp via [`TestContextConfig`](crate::fixtures::core::context::TestContextConfig).
pub fn year_2023() -> Timestamp {
    // Sun Jan 01 2023 00:00:00 GMT+0000
    Time::from_unix_timestamp(1_672_531_200, 0)
        .expect("should be a valid time")
        .into()
}

/// Utility function that asserts that the given JSON input can be
/// serialized into and deserialized from the specified type `T`.
#[cfg(feature = "serde")]
pub fn test_serialization_roundtrip<T>(json_data: &str)
where
    T: core::fmt::Debug + PartialEq + Serialize + DeserializeOwned,
{
    let parsed0 = serde_json::from_str::<T>(json_data);
    assert!(parsed0.is_ok());
    let parsed0 = parsed0.expect("should not fail");

    let serialized = serde_json::to_string(&parsed0);
    assert!(serialized.is_ok());
    let serialized = serialized.expect("should not fail");

    let parsed1 = serde_json::from_str::<T>(&serialized);
    assert!(parsed1.is_ok());
    let parsed1 = parsed1.expect("should not fail");

    assert_eq!(parsed0, parsed1);
}
