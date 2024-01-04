//! Defines Non-Fungible Token Transfer (ICS-721) data types.
use core::fmt::{self, Display, Formatter};
use core::str::FromStr;

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
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Data(BTreeMap<String, DataValue>);

#[derive(Clone, Debug, Default, PartialEq, Eq)]
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
        borsh::BorshSerialize::serialize(&mime.to_string(), writer)?;
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

impl Display for Data {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", serde_json::to_string(&self.0).expect("infallible"))
    }
}

impl FromStr for Data {
    type Err = NftTransferError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let data: BTreeMap<String, DataValue> =
            serde_json::from_str(s).map_err(|_| NftTransferError::InvalidJsonData)?;

        Ok(Self(data))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "borsh")]
    #[test]
    fn test_valid_borsh_ser_de_roundtrip() {
        fn borsh_ser_de_roundtrip(data_value: DataValue) {
            use borsh::{BorshDeserialize, BorshSerialize};

            let data_value_bytes = data_value.try_to_vec().unwrap();
            let res = DataValue::try_from_slice(&data_value_bytes).unwrap();

            assert_eq!(data_value, res);
        }

        borsh_ser_de_roundtrip(DataValue {
            value: String::from("foo"),
            mime: None,
        });

        borsh_ser_de_roundtrip(DataValue {
            value: String::from("foo"),
            mime: Some(mime::TEXT_PLAIN_UTF_8),
        });
    }
}
