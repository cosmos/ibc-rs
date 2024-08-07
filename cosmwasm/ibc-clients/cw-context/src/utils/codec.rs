use ibc_core::client::types::error::ClientError;
use ibc_core::primitives::proto::Any;
use prost::Message;

/// AnyCodec is a convenient trait that provides a generic way to encode and
/// decode domain types through the `Any` type.
pub trait AnyCodec {
    fn decode_any_vec<C>(data: Vec<u8>) -> Result<C, ClientError>
    where
        C: TryFrom<Any>,
        <C as TryFrom<Any>>::Error: Into<ClientError>,
    {
        let raw = Any::decode(&mut data.as_slice()).map_err(|e| ClientError::Other {
            description: e.to_string(),
        })?;

        C::try_from(raw).map_err(Into::into)
    }

    fn encode_to_any_vec<C>(value: C) -> Vec<u8>
    where
        C: Into<Any>,
    {
        value.into().encode_to_vec()
    }
}

impl<T> AnyCodec for T where T: TryFrom<Any> + Into<Any> {}
