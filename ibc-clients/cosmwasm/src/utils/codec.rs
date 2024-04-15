use ibc_core::client::types::error::ClientError;
use ibc_core::primitives::proto::Any;
use prost::Message;

pub trait AnyCodec {
    fn decode_thru_any<C>(data: Vec<u8>) -> Result<C, ClientError>
    where
        C: TryFrom<Any, Error = ClientError>,
    {
        let raw = Any::decode(&mut data.as_slice()).map_err(|e| ClientError::Other {
            description: e.to_string(),
        })?;

        C::try_from(raw)
    }

    fn encode_thru_any<C>(value: C) -> Vec<u8>
    where
        C: Into<Any>,
    {
        value.into().encode_to_vec()
    }
}

impl<T> AnyCodec for T where T: TryFrom<Any, Error = ClientError> + Into<Any> {}
