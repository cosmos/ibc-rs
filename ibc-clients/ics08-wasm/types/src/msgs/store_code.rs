use ibc_primitives::prelude::*;
use ibc_primitives::Signer;
use ibc_proto::ibc::lightclients::wasm::v1::MsgStoreCode as RawMsgStoreCode;
use ibc_proto::Protobuf;

use crate::error::Error;
use crate::Bytes;

pub const STORE_CODE_TYPE_URL: &str = "/ibc.lightclients.wasm.v1.MsgStoreCode";

/// Defines the message type for storing the Wasm byte code on the chain.
#[derive(Clone, PartialEq, Debug, Eq)]
pub struct MsgStoreCode {
    pub signer: Signer,
    pub wasm_byte_code: Bytes,
}

impl Protobuf<RawMsgStoreCode> for MsgStoreCode {}

impl From<MsgStoreCode> for RawMsgStoreCode {
    fn from(value: MsgStoreCode) -> Self {
        Self {
            signer: value.signer.to_string(),
            wasm_byte_code: value.wasm_byte_code,
        }
    }
}

impl TryFrom<RawMsgStoreCode> for MsgStoreCode {
    type Error = Error;

    fn try_from(value: RawMsgStoreCode) -> Result<Self, Self::Error> {
        Ok(Self {
            signer: Signer::from(value.signer),
            wasm_byte_code: value.wasm_byte_code,
        })
    }
}
