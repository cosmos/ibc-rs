use crate::clients::ics06_solomachine::error::Error;
use crate::core::timestamp::Timestamp;
use crate::prelude::*;
use ibc_proto::ibc::lightclients::solomachine::v2::TimestampedSignatureData as RawTimestampedSignatureData;
use ibc_proto::protobuf::Protobuf;

/// TimestampedSignatureData contains the signature data and the timestamp of the
/// signature.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, PartialEq)]
pub struct TimestampedSignatureData {
    pub signature_data: Vec<u8>,
    pub timestamp: Timestamp,
}

impl Protobuf<RawTimestampedSignatureData> for TimestampedSignatureData {}

impl TryFrom<RawTimestampedSignatureData> for TimestampedSignatureData {
    type Error = Error;

    fn try_from(raw: RawTimestampedSignatureData) -> Result<Self, Self::Error> {
        Ok(Self {
            signature_data: raw.signature_data,
            timestamp: Timestamp::from_nanoseconds(raw.timestamp).map_err(Error::ParseTimeError)?,
        })
    }
}

impl From<TimestampedSignatureData> for RawTimestampedSignatureData {
    fn from(value: TimestampedSignatureData) -> Self {
        Self {
            signature_data: value.signature_data,
            timestamp: value.timestamp.nanoseconds(),
        }
    }
}
