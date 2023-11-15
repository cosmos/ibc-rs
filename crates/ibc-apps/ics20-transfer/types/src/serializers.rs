use core::fmt::Display;
use core::str::FromStr;

use ibc::prelude::*;
use serde::{de, Deserialize, Deserializer, Serializer};

// Note: used String version (slower + heap) instead of str,
// because both str ser/de hit some kind of f64/f32 case when compiled into wasm
// and fails to be validated f32/f64 wasm runtimes
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
