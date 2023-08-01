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
