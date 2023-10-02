use serde::ser::{Serialize, Serializer};
use subtle_encoding::{Encoding, Hex};

pub fn ser_hex_upper<S, T>(data: T, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: AsRef<[u8]>,
{
    let hex = Hex::upper_case()
        .encode_to_string(data)
        .map_err(|e| serde::ser::Error::custom(alloc::format!("failed to serialize hex: {}", e)))?;
    hex.serialize(serializer)
}

pub mod serde_string {
    use crate::prelude::*;
    use core::fmt::Display;
    use core::str::FromStr;
    use serde::{de, Deserialize, Deserializer, Serializer};

    // used String version (slower + heap) instead of str,
    // because both str ser/de hit some kind of f64/f32 case which compiled into wasm
    // and fails to be validated f32/f64 wasm runtimes
    // sure fix must be eventually in serde
    pub fn serialize<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: Display,
        S: Serializer,
    {
        serializer.serialize_str(value.to_string().as_ref())
    }

    pub fn deserialize<'de, T, D>(deserializer: D) -> Result<T, D::Error>
    where
        T: FromStr,
        T::Err: Display,
        D: Deserializer<'de>,
    {
        T::from_str(<String>::deserialize(deserializer)?.as_str()).map_err(de::Error::custom)
    }
}

/// Test that a struct `T` can be:
///
/// - parsed out of the provided JSON data
/// - serialized back to JSON
/// - parsed back from the serialized JSON of the previous step
/// - that the two parsed structs are equal according to their `PartialEq` impl
#[cfg(test)]
pub mod tests {
    use serde::de::DeserializeOwned;

    use super::*;

    pub fn test_serialization_roundtrip<T>(json_data: &str)
    where
        T: core::fmt::Debug + PartialEq + Serialize + DeserializeOwned,
    {
        let parsed0 = serde_json::from_str::<T>(json_data);
        assert!(parsed0.is_ok());
        let parsed0 = parsed0.unwrap();

        let serialized = serde_json::to_string(&parsed0);
        assert!(serialized.is_ok());
        let serialized = serialized.unwrap();

        let parsed1 = serde_json::from_str::<T>(&serialized);
        assert!(parsed1.is_ok());
        let parsed1 = parsed1.unwrap();

        assert_eq!(parsed0, parsed1);
    }
}
