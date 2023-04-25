use crate::clients::ics06_solomachine::error::Error;
use crate::prelude::*;
use crate::timestamp::Timestamp;
use ibc_proto::ibc::lightclients::solomachine::v1::SignBytes as RawSignBytes;

use super::DataType;
// use ibc_proto::protobuf::Protobuf;

/// SignBytes defines the signed bytes used for signature verification.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, PartialEq)]
pub struct SignBytes {
    pub sequence: u64,
    pub timestamp: Timestamp,
    pub diversifier: String,
    /// type of the data used
    pub data_type: DataType,
    /// marshaled data
    pub data: Vec<u8>,
}

// impl Protobuf<RawSignBytes> for SignBytes {}

impl TryFrom<RawSignBytes> for SignBytes {
    type Error = Error;

    fn try_from(raw: RawSignBytes) -> Result<Self, Self::Error> {
        Ok(Self {
            sequence: raw.sequence,
            timestamp: Timestamp::from_nanoseconds(raw.timestamp).map_err(Error::ParseTimeError)?,
            diversifier: raw.diversifier,
            data_type: DataType::try_from(raw.data_type)?,
            data: raw.data,
        })
    }
}

impl From<SignBytes> for RawSignBytes {
    fn from(value: SignBytes) -> Self {
        Self {
            sequence: value.sequence,
            timestamp: value.timestamp.nanoseconds(),
            diversifier: value.diversifier,
            data_type: i32::from(value.data_type),
            data: value.data,
        }
    }
}
