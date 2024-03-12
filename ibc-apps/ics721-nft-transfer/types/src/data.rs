//! Defines Non-Fungible Token Transfer (ICS-721) data types.
use core::fmt::{self, Display, Formatter};
use core::str::FromStr;

#[cfg(feature = "serde")]
use base64::prelude::BASE64_STANDARD;
#[cfg(feature = "serde")]
use base64::Engine;
use ibc_core::primitives::prelude::*;
use mime::Mime;

use crate::error::NftTransferError;

#[cfg_attr(
    feature = "parity-scale-codec",
    derive(
        parity_scale_codec::Encode,
        parity_scale_codec::Decode,
        scale_info::TypeInfo
    )
)]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, PartialEq, Eq, derive_more::From)]
pub struct Data(String);

#[cfg(feature = "serde")]
impl Data {
    /// Parses the data in the format specified by ICS-721.
    pub fn parse_as_ics721_data(&self) -> Result<Ics721Data, NftTransferError> {
        self.0.parse::<Ics721Data>()
    }
}

impl Display for Data {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for Data {
    type Err = NftTransferError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_string()))
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Data {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&BASE64_STANDARD.encode(&self.0))
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Data {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let encoded = String::deserialize(deserializer)?;
        let decoded = BASE64_STANDARD
            .decode(encoded)
            .map_err(serde::de::Error::custom)?;
        let decoded_str = String::from_utf8(decoded).map_err(serde::de::Error::custom)?;
        Ok(Data(decoded_str))
    }
}

#[cfg_attr(
    feature = "parity-scale-codec",
    derive(
        parity_scale_codec::Encode,
        parity_scale_codec::Decode,
        scale_info::TypeInfo
    )
)]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ics721Data(BTreeMap<String, DataValue>);

#[cfg(feature = "serde")]
impl FromStr for Ics721Data {
    type Err = NftTransferError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s).map_err(|_| NftTransferError::InvalidIcs721Data)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DataValue {
    value: String,
    mime: Option<Mime>,
}

#[cfg(feature = "serde")]
impl serde::Serialize for DataValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("DataValue", 2)?;
        state.serialize_field("value", &self.value)?;
        match &self.mime {
            Some(mime) if *mime != "" => {
                state.serialize_field("mime", &mime.to_string())?;
            }
            _ => {}
        }
        state.end()
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for DataValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        struct StringDataValue {
            value: String,
            mime: Option<String>,
        }

        let data_value = StringDataValue::deserialize(deserializer)?;
        let mime = data_value
            .mime
            .map(|s| Mime::from_str(&s).map_err(serde::de::Error::custom))
            .transpose()?;

        Ok(DataValue {
            value: data_value.value,
            mime,
        })
    }
}

#[cfg(feature = "borsh")]
impl borsh::BorshSerialize for DataValue {
    fn serialize<W: borsh::maybestd::io::Write>(
        &self,
        writer: &mut W,
    ) -> borsh::maybestd::io::Result<()> {
        borsh::BorshSerialize::serialize(&self.value, writer)?;
        let mime = match &self.mime {
            Some(mime) => mime.to_string(),
            None => String::default(),
        };
        borsh::BorshSerialize::serialize(&mime, writer)?;
        Ok(())
    }
}

#[cfg(feature = "borsh")]
impl borsh::BorshDeserialize for DataValue {
    fn deserialize_reader<R: borsh::maybestd::io::Read>(
        reader: &mut R,
    ) -> borsh::maybestd::io::Result<Self> {
        let value = String::deserialize_reader(reader)?;
        let mime = String::deserialize_reader(reader)?;
        let mime = if mime.is_empty() {
            None
        } else {
            Some(Mime::from_str(&mime).map_err(|_| borsh::maybestd::io::ErrorKind::Other)?)
        };

        Ok(Self { value, mime })
    }
}

#[cfg(feature = "parity-scale-codec")]
impl parity_scale_codec::Encode for DataValue {
    fn encode_to<T: parity_scale_codec::Output + ?Sized>(&self, writer: &mut T) {
        self.value.encode_to(writer);
        if let Some(mime) = &self.mime {
            mime.to_string().encode_to(writer);
        } else {
            "".encode_to(writer);
        }
    }
}

#[cfg(feature = "parity-scale-codec")]
impl parity_scale_codec::Decode for DataValue {
    fn decode<I: parity_scale_codec::Input>(
        input: &mut I,
    ) -> Result<Self, parity_scale_codec::Error> {
        let value = String::decode(input)?;
        let mime_str = String::decode(input)?;
        let mime = if mime_str.is_empty() {
            None
        } else {
            Some(
                Mime::from_str(&mime_str)
                    .map_err(|_| parity_scale_codec::Error::from("from str error"))?,
            )
        };

        Ok(DataValue { value, mime })
    }
}

#[cfg(feature = "parity-scale-codec")]
impl scale_info::TypeInfo for DataValue {
    type Identity = Self;

    fn type_info() -> scale_info::Type {
        scale_info::Type::builder()
            .path(scale_info::Path::new("DataValue", module_path!()))
            .composite(
                scale_info::build::Fields::named()
                    .field(|f| f.ty::<String>().name("value").type_name("String"))
                    .field(|f| f.ty::<String>().name("mime").type_name("String")),
            )
    }
}

#[cfg(feature = "schema")]
impl schemars::JsonSchema for DataValue {
    fn schema_name() -> String {
        "DataValue".to_string()
    }

    fn schema_id() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed(concat!(module_path!(), "::DataValue"))
    }

    fn json_schema(gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        gen.subschema_for::<Self>()
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[cfg(feature = "serde")]
    #[rstest]
    #[case(r#"{"value":"foo"}"#)]
    #[case(r#"{"value":"foo-42","mime":"multipart/form-data; boundary=ABCDEFG"}"#)]
    fn test_valid_json_deserialization(#[case] data_value_json: &str) {
        assert!(serde_json::from_str::<DataValue>(data_value_json).is_ok());
    }

    #[cfg(feature = "serde")]
    #[rstest]
    #[case(r#"{"value":"foo-42","mime":"invalid"}"#)]
    #[case(r#"{"value":"invalid","mime":""}"#)]
    fn test_invalid_json_deserialization(#[case] data_value_json: &str) {
        assert!(serde_json::from_str::<DataValue>(data_value_json).is_err());
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_serde_json_roundtrip() {
        fn serde_roundtrip(data_value: DataValue) {
            let serialized =
                serde_json::to_string(&data_value).expect("failed to serialize DataValue");
            let deserialized = serde_json::from_str::<DataValue>(&serialized)
                .expect("failed to deserialize DataValue");

            assert_eq!(deserialized, data_value);
        }

        serde_roundtrip(DataValue {
            value: String::from("foo"),
            mime: None,
        });

        serde_roundtrip(DataValue {
            value: String::from("foo"),
            mime: Some(mime::TEXT_PLAIN_UTF_8),
        });
    }

    #[cfg(feature = "borsh")]
    #[test]
    fn test_borsh_roundtrip() {
        fn borsh_roundtrip(data_value: DataValue) {
            use borsh::{BorshDeserialize, BorshSerialize};

            let data_value_bytes = data_value.try_to_vec().unwrap();
            let res = DataValue::try_from_slice(&data_value_bytes).unwrap();

            assert_eq!(data_value, res);
        }

        borsh_roundtrip(DataValue {
            value: String::from("foo"),
            mime: None,
        });

        borsh_roundtrip(DataValue {
            value: String::from("foo"),
            mime: Some(mime::TEXT_PLAIN_UTF_8),
        });
    }
}
