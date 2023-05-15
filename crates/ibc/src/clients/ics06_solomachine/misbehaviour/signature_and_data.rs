use crate::clients::ics06_solomachine::error::Error;
use crate::clients::ics06_solomachine::types::DataType;
use crate::core::timestamp::Timestamp;
use crate::prelude::*;
use ibc_proto::ibc::lightclients::solomachine::v2::SignatureAndData as RawSignatureAndData;
use ibc_proto::protobuf::Protobuf;

/// SignatureAndData contains a signature and the data signed over to create that
/// signature.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, PartialEq)]
pub struct SignatureAndData {
    pub signature: Vec<u8>,
    pub data_type: DataType,
    pub data: Vec<u8>,
    pub timestamp: Timestamp,
}
impl core::fmt::Display for SignatureAndData {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        write!(
            f,
            "signature: {:?}, data_type: {}, data: {:?}, timestamp: {}",
            self.signature, self.data_type, self.data, self.timestamp
        )
    }
}

impl Protobuf<RawSignatureAndData> for SignatureAndData {}

impl TryFrom<RawSignatureAndData> for SignatureAndData {
    type Error = Error;

    fn try_from(raw: RawSignatureAndData) -> Result<Self, Self::Error> {
        let signature = raw.signature;
        let data_type = DataType::try_from(raw.data_type)?;
        let data = raw.data;
        let timestamp =
            Timestamp::from_nanoseconds(raw.timestamp).map_err(Error::ParseTimeError)?;
        Ok(Self {
            signature,
            data_type,
            data,
            timestamp,
        })
    }
}

impl From<SignatureAndData> for RawSignatureAndData {
    fn from(value: SignatureAndData) -> Self {
        Self {
            signature: value.signature,
            data_type: i32::from(value.data_type),
            data: value.data,
            timestamp: value.timestamp.nanoseconds(),
        }
    }
}
