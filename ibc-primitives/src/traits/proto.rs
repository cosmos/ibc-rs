use core::fmt::Display;

use ibc_proto::google::protobuf::Any;
use ibc_proto::Protobuf;

use crate::prelude::*;

/// Types that implement this trait are able to be converted to
/// a raw Protobuf `Any` type.
pub trait ToProto<P>: Protobuf<P>
where
    P: From<Self> + prost::Message + prost::Name + Default,
    <Self as TryFrom<P>>::Error: Display,
{
    fn type_url() -> String {
        P::type_url()
    }

    fn to_any(self) -> Any {
        Any {
            type_url: P::type_url(),
            value: self.encode_vec(),
        }
    }
}

impl<T, P> ToProto<P> for T
where
    T: Protobuf<P>,
    P: From<Self> + prost::Message + prost::Name + Default,
    <Self as TryFrom<P>>::Error: Display,
{
}

/// Convenient trait for converting types to a raw Protobuf `Vec<u8>`.
pub trait ToVec {
    fn to_vec(&self) -> Vec<u8>;
}

impl<T: prost::Message> ToVec for T {
    fn to_vec(&self) -> Vec<u8> {
        self.encode_to_vec()
    }
}
