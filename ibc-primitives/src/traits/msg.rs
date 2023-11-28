use ibc_proto::{google::protobuf::Any, Protobuf};

use crate::prelude::*;

use core::fmt::Display;

/// Trait to be implemented by all IBC messages
pub trait Msg: Protobuf<Self::Raw>
where
    <Self as TryFrom<Self::Raw>>::Error: Display,
{
    type Raw: From<Self> + prost::Message + prost::Name + Default;

    fn to_any(self) -> Any {
        Any {
            type_url: self.type_url(),
            value: self.encode_vec(),
        }
    }
}
