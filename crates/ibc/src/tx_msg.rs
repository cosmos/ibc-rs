use crate::prelude::*;
use ibc_proto::google::protobuf::Any;

pub trait Msg: Clone {
    type Raw: From<Self> + prost::Message;

    /// Unique type identifier for this message, to support encoding to/from `prost_types::Any`.
    fn type_url(&self) -> String;

    fn get_sign_bytes(self) -> Vec<u8> {
        let raw_msg: Self::Raw = self.into();
        prost::Message::encode_to_vec(&raw_msg)
    }

    fn to_any(self) -> Any {
        Any {
            type_url: self.type_url(),
            value: self.get_sign_bytes(),
        }
    }
}
