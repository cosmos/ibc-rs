use ibc_proto::{google::protobuf::Any, Protobuf};

use crate::prelude::*;

use core::fmt::Display;

/// Types that implement this trait are able to be converted to
/// a raw Protobuf `Any` type.
pub trait ToProto: Protobuf<Self::Proto>
where
    Self::Proto: From<Self> + prost::Message + prost::Name + Default,
    <Self as TryFrom<Self::Proto>>::Error: Display,
{
    type Proto;

    fn to_any(self) -> Any {
        use prost::Name;

        Any {
            type_url: Self::Proto::type_url(),
            value: self.encode_vec(),
        }
    }
}
