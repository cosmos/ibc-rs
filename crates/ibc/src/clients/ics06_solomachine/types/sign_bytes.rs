use crate::clients::ics06_solomachine::error::Error;
use crate::prelude::*;
use ibc_proto::ibc::lightclients::solomachine::v1::SignBytes as RawSignBytes;
use ibc_proto::ibc::lightclients::solomachine::v1::TimestampedSignatureData as RawTimestampedSignatureData;
// use ibc_proto::protobuf::Protobuf;

/// SignBytes defines the signed bytes used for signature verification.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, PartialEq)]
pub struct SignBytes {
    pub sequence: u64,
    pub timestamp: u64,
    pub diversifier: String,
    /// type of the data used
    pub data_type: i32,
    /// marshaled data
    pub data: Vec<u8>,
}

// impl Protobuf<RawSignBytes> for SignBytes {}

impl TryFrom<RawSignBytes> for SignBytes {
    type Error = Error;

    fn try_from(raw: RawSignBytes) -> Result<Self, Self::Error> {
        todo!()
    }
}

impl From<SignBytes> for RawTimestampedSignatureData {
    fn from(value: SignBytes) -> Self {
        todo!()
    }
}
