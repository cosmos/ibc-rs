use ibc_primitives::prelude::*;
use ibc_primitives::Signer;
use ibc_proto::ibc::lightclients::wasm::v1::MsgRemoveChecksum as RawMsgRemoveChecksum;
use ibc_proto::Protobuf;

use crate::Bytes;

pub const REMOVE_CHECKSUM_TYPE_URL: &str = "/ibc.lightclients.wasm.v1.MsgRemoveChecksum";

/// Defines the message type for removing a checksum from the chain.
#[derive(Clone, PartialEq, Debug, Eq)]
pub struct MsgRemoveChecksum {
    pub signer: Signer,
    pub checksum: Bytes,
}

impl Protobuf<RawMsgRemoveChecksum> for MsgRemoveChecksum {}

impl From<MsgRemoveChecksum> for RawMsgRemoveChecksum {
    fn from(value: MsgRemoveChecksum) -> Self {
        Self {
            signer: value.signer.to_string(),
            checksum: value.checksum,
        }
    }
}

impl From<RawMsgRemoveChecksum> for MsgRemoveChecksum {
    fn from(value: RawMsgRemoveChecksum) -> Self {
        Self {
            signer: Signer::from(value.signer),
            checksum: value.checksum,
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("signer", b"checksum")]
    fn test_roundtrip(#[case] signer: &str, #[case] checksum: &[u8]) {
        let raw_msg = RawMsgRemoveChecksum {
            signer: signer.to_string(),
            checksum: checksum.to_vec(),
        };
        assert_eq!(
            RawMsgRemoveChecksum::from(MsgRemoveChecksum::from(raw_msg.clone())),
            raw_msg,
        )
    }
}
