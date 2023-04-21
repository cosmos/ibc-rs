use crate::clients::ics06_solomachine::error::Error;
use crate::clients::ics06_solomachine::types::DataType;
use crate::prelude::*;
use ibc_proto::ibc::lightclients::solomachine::v1::SignatureAndData as RawSignatureAndData;
use ibc_proto::protobuf::Protobuf;

/// SignatureAndData contains a signature and the data signed over to create that
/// signature.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, PartialEq)]
pub struct SignatureAndData {
    pub signature: Vec<u8>,
    pub data_type: DataType,
    pub data: Vec<u8>,
    pub timestamp: u64,
}

impl Protobuf<RawSignatureAndData> for SignatureAndData {}

impl TryFrom<RawSignatureAndData> for SignatureAndData {
    type Error = Error;

    fn try_from(raw: RawSignatureAndData) -> Result<Self, Self::Error> {
        todo!()
    }
}

impl From<SignatureAndData> for RawSignatureAndData {
    fn from(value: SignatureAndData) -> Self {
        todo!()
    }
}
