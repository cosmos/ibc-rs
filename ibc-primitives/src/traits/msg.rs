use ibc_proto::{google::protobuf::Any, Protobuf};

use crate::prelude::*;

use core::fmt::Display;

/// Types that implement this trait are able to be converted to
/// a raw Protobuf `Any` type.
pub trait ToProto: Protobuf<Self::Proto> + prost::Name
where
    Self::Proto: From<Self> + prost::Message + Default,
    <Self as TryFrom<Self::Proto>>::Error: Display,
{
    type Proto;

    fn to_any(self) -> Any {
        Any {
            type_url: Self::type_url(),
            value: self.encode_vec(),
        }
    }
}
