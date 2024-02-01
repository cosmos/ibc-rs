use ibc_primitives::prelude::*;
use ibc_primitives::Signer;
use ibc_proto::ibc::lightclients::wasm::v1::MsgStoreCode as RawMsgStoreCode;
use ibc_proto::Protobuf;

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

impl From<RawMsgStoreCode> for MsgStoreCode {
    fn from(value: RawMsgStoreCode) -> Self {
        Self {
            signer: Signer::from(value.signer),
            wasm_byte_code: value.wasm_byte_code,
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("signer", b"wasm_data")]
    fn test_roundtrip(#[case] signer: &str, #[case] wasm_byte_code: &[u8]) {
        let raw_msg = RawMsgStoreCode {
            signer: signer.to_string(),
            wasm_byte_code: wasm_byte_code.to_vec(),
        };
        assert_eq!(
            RawMsgStoreCode::from(MsgStoreCode::from(raw_msg.clone())),
            raw_msg,
        )
    }
}
