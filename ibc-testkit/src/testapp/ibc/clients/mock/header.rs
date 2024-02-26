use alloc::string::ToString;
use core::fmt::{Display, Error as FmtError, Formatter};

use ibc::core::client::types::error::ClientError;
use ibc::core::client::types::Height;
use ibc::core::primitives::Timestamp;
use ibc::primitives::proto::{Any, Protobuf};

use crate::testapp::ibc::clients::mock::proto::Header as RawMockHeader;

pub const MOCK_HEADER_TYPE_URL: &str = "/ibc.mock.Header";

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct MockHeader {
    pub height: Height,
    pub timestamp: Timestamp,
}

impl Default for MockHeader {
    fn default() -> Self {
        Self {
            height: Height::min(0),
            timestamp: Timestamp::none(),
        }
    }
}

impl Display for MockHeader {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        write!(
            f,
            "MockHeader {{ height: {}, timestamp: {} }}",
            self.height, self.timestamp
        )
    }
}

impl Protobuf<RawMockHeader> for MockHeader {}

impl TryFrom<RawMockHeader> for MockHeader {
    type Error = ClientError;

    fn try_from(raw: RawMockHeader) -> Result<Self, Self::Error> {
        Ok(MockHeader {
            height: raw
                .height
                .and_then(|raw_height| raw_height.try_into().ok())
                .ok_or(ClientError::MissingClientMessage)?,

            timestamp: Timestamp::from_nanoseconds(raw.timestamp)
                .map_err(ClientError::InvalidPacketTimestamp)?,
        })
    }
}

impl From<MockHeader> for RawMockHeader {
    fn from(value: MockHeader) -> Self {
        RawMockHeader {
            height: Some(value.height.into()),
            timestamp: value.timestamp.nanoseconds(),
        }
    }
}

impl MockHeader {
    pub fn height(&self) -> Height {
        self.height
    }

    pub fn new(height: Height) -> Self {
        Self {
            height,
            timestamp: Timestamp::none(),
        }
    }

    pub fn with_current_timestamp(self) -> Self {
        Self {
            timestamp: Timestamp::now(),
            ..self
        }
    }

    pub fn with_timestamp(self, timestamp: Timestamp) -> Self {
        Self { timestamp, ..self }
    }
}

impl Protobuf<Any> for MockHeader {}

impl TryFrom<Any> for MockHeader {
    type Error = ClientError;

    fn try_from(raw: Any) -> Result<Self, Self::Error> {
        match raw.type_url.as_str() {
            MOCK_HEADER_TYPE_URL => Ok(Protobuf::<RawMockHeader>::decode_vec(&raw.value).map_err(
                |e| ClientError::InvalidRawHeader {
                    reason: e.to_string(),
                },
            )?),
            _ => Err(ClientError::UnknownHeaderType {
                header_type: raw.type_url,
            }),
        }
    }
}

impl From<MockHeader> for Any {
    fn from(header: MockHeader) -> Self {
        Any {
            type_url: MOCK_HEADER_TYPE_URL.to_string(),
            value: Protobuf::<RawMockHeader>::encode_vec(header),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn encode_any() {
        let header = MockHeader::new(Height::new(1, 10).expect("Never fails"))
            .with_timestamp(Timestamp::none());
        let bytes = <MockHeader as Protobuf<Any>>::encode_vec(header);

        assert_eq!(
            &bytes,
            &[
                10, 16, 47, 105, 98, 99, 46, 109, 111, 99, 107, 46, 72, 101, 97, 100, 101, 114, 18,
                6, 10, 4, 8, 1, 16, 10
            ]
        );
    }
}
