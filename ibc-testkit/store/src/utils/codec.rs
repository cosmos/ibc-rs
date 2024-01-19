use std::marker::PhantomData;

use serde::de::DeserializeOwned;
use serde::Serialize;

/// A trait that defines how types are decoded/encoded.
pub trait Codec {
    type Type;
    type Encoded: AsRef<[u8]>;

    fn encode(d: &Self::Type) -> Option<Self::Encoded>;

    fn decode(bytes: &[u8]) -> Option<Self::Type>;
}

/// A JSON codec that uses `serde_json` to encode/decode as a JSON string
#[derive(Clone, Debug)]
pub struct JsonCodec<T>(PhantomData<T>);

impl<T> Codec for JsonCodec<T>
where
    T: Serialize + DeserializeOwned,
{
    type Type = T;
    type Encoded = String;

    fn encode(d: &Self::Type) -> Option<Self::Encoded> {
        serde_json::to_string(d).ok()
    }

    fn decode(bytes: &[u8]) -> Option<Self::Type> {
        let json_string = String::from_utf8(bytes.to_vec()).ok()?;
        serde_json::from_str(&json_string).ok()
    }
}

/// A Null codec that can be used for paths that are only meant to be set/reset and do not hold any
/// typed value.
#[derive(Clone)]
pub struct NullCodec;

impl Codec for NullCodec {
    type Type = ();
    type Encoded = Vec<u8>;

    fn encode(_d: &Self::Type) -> Option<Self::Encoded> {
        // using [0x00] to represent null
        Some(vec![0x00])
    }

    fn decode(bytes: &[u8]) -> Option<Self::Type> {
        match bytes {
            // the encoded bytes must be [0x00]
            [0x00] => Some(()),
            _ => None,
        }
    }
}

/// A Protobuf codec that uses `prost` to encode/decode
#[derive(Clone, Debug)]
pub struct ProtobufCodec<T, R> {
    domain_type: PhantomData<T>,
    raw_type: PhantomData<R>,
}

impl<T, R> Codec for ProtobufCodec<T, R>
where
    T: Into<R> + Clone,
    R: TryInto<T> + Default + prost::Message,
{
    type Type = T;
    type Encoded = Vec<u8>;

    fn encode(d: &Self::Type) -> Option<Self::Encoded> {
        let r = d.clone().into();
        Some(r.encode_to_vec())
    }

    fn decode(bytes: &[u8]) -> Option<Self::Type> {
        let r = R::decode(bytes).ok()?;
        r.try_into().ok()
    }
}

/// A binary codec that uses `AsRef<[u8]>` and `From<Vec<u8>>` to encode and decode respectively.
#[derive(Clone, Debug)]
pub struct BinCodec<T>(PhantomData<T>);

impl<T> Codec for BinCodec<T>
where
    T: AsRef<[u8]> + From<Vec<u8>>,
{
    type Type = T;
    type Encoded = Vec<u8>;

    fn encode(d: &Self::Type) -> Option<Self::Encoded> {
        Some(d.as_ref().to_vec())
    }

    fn decode(bytes: &[u8]) -> Option<Self::Type> {
        Some(bytes.to_vec().into())
    }
}
