use crate::prelude::*;

/// Dummy type that mirrors `ibc_proto::google::protobuf::Any`.
/// Meant to be used with `#[cfg_attr(feature = "schema", schemars(with = "crate::utils::schema::AnySchema"))]`
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct AnySchema {
    pub type_url: String,
    pub value: Vec<u8>,
}
