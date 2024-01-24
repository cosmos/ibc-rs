use core::fmt::Display;
use core::str::FromStr;

use serde::{de, Deserialize, Deserializer, Serializer};

use crate::prelude::*;

// Note: This method serializes to a String instead of a str in order to
// avoid a wasm compilation issue. Specifically, str (de)serialization hits
// some kind of f64/f32 case when compiled into wasm, but this fails
// validation on f32/f64 wasm runtimes.
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
