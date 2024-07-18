use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use ibc_primitives::prelude::*;
use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

pub struct Base64;

impl Base64 {
    pub fn serialize<S: Serializer>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error> {
        let encoded = BASE64_STANDARD.encode(bytes);
        String::serialize(&encoded, serializer)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<u8>, D::Error> {
        let base64 = String::deserialize(deserializer)?;
        let bytes = BASE64_STANDARD
            .decode(base64.as_bytes())
            .map_err(Error::custom)?;

        Ok(bytes)
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
    struct Foo(#[serde(with = "Base64")] crate::Bytes);

    // Ensures Base64 serialize and deserialize work as expected
    #[rstest]
    #[case(b"", "")]
    #[case(&[118], "dg==")]
    #[case(&[0x1, 0x2, 0x3, 0x4, 0x5], "AQIDBAU=")]
    #[case(b"hello world", "aGVsbG8gd29ybGQ=")]
    pub fn test_ser_and_deser(#[case] bytes: &[u8], #[case] hash: &str) {
        let foo = Foo(bytes.to_vec());
        let json = format!("\"{hash}\"");
        assert_eq!(serde_json::to_string(&foo).unwrap(), json);
        assert_eq!(serde_json::from_str::<Foo>(&json).unwrap(), foo);
    }
}
