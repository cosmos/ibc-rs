//! Defines the core `Height` type used throughout the library

use core::cmp::Ordering;
use core::str::FromStr;

use ibc_core_host_types::error::DecodingError;
use ibc_primitives::prelude::*;
use ibc_proto::ibc::core::client::v1::Height as RawHeight;
use ibc_proto::Protobuf;

use crate::error::ClientError;

/// The core IBC height type, which represents the height of a chain,
/// which typically is the number of blocks since genesis
/// (or more generally, since the last revision/hard upgrade).
#[cfg_attr(
    feature = "parity-scale-codec",
    derive(
        parity_scale_codec::Encode,
        parity_scale_codec::Decode,
        scale_info::TypeInfo
    )
)]
#[cfg_attr(
    feature = "borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Height {
    /// Previously known as "epoch"
    revision_number: u64,

    /// The height of a block
    revision_height: u64,
}

impl Height {
    pub fn new(revision_number: u64, revision_height: u64) -> Result<Self, ClientError> {
        if revision_height == 0 {
            return Err(ClientError::InvalidHeight);
        }

        Ok(Self {
            revision_number,
            revision_height,
        })
    }

    pub fn min(revision_number: u64) -> Self {
        Self {
            revision_number,
            revision_height: 1,
        }
    }

    pub fn revision_number(&self) -> u64 {
        self.revision_number
    }

    pub fn revision_height(&self) -> u64 {
        self.revision_height
    }

    pub fn add(&self, delta: u64) -> Height {
        Height {
            revision_number: self.revision_number,
            revision_height: self.revision_height + delta,
        }
    }

    pub fn increment(&self) -> Height {
        self.add(1)
    }

    pub fn sub(&self, delta: u64) -> Result<Height, ClientError> {
        if self.revision_height <= delta {
            return Err(ClientError::InvalidHeight);
        }

        Ok(Height {
            revision_number: self.revision_number,
            revision_height: self.revision_height - delta,
        })
    }

    pub fn decrement(&self) -> Result<Height, ClientError> {
        self.sub(1)
    }
}

impl PartialOrd for Height {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Height {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.revision_number < other.revision_number {
            Ordering::Less
        } else if self.revision_number > other.revision_number {
            Ordering::Greater
        } else if self.revision_height < other.revision_height {
            Ordering::Less
        } else if self.revision_height > other.revision_height {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    }
}

impl Protobuf<RawHeight> for Height {}

impl TryFrom<RawHeight> for Height {
    type Error = DecodingError;

    fn try_from(raw_height: RawHeight) -> Result<Self, Self::Error> {
        Height::new(raw_height.revision_number, raw_height.revision_height)
            .map_err(|_| DecodingError::invalid_raw_data("height of 0 not allowed"))
    }
}

impl From<Height> for RawHeight {
    fn from(ics_height: Height) -> Self {
        RawHeight {
            revision_number: ics_height.revision_number,
            revision_height: ics_height.revision_height,
        }
    }
}

impl core::fmt::Debug for Height {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        f.debug_struct("Height")
            .field("revision", &self.revision_number)
            .field("height", &self.revision_height)
            .finish()
    }
}

/// Custom debug output to omit the packet data
impl core::fmt::Display for Height {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        write!(f, "{}-{}", self.revision_number, self.revision_height)
    }
}

impl TryFrom<&str> for Height {
    type Error = DecodingError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let (rev_number_str, rev_height_str) = value.split_once('-').ok_or_else(|| {
            DecodingError::invalid_raw_data(format!("height `{value}` not properly formatted"))
        })?;

        let revision_number = rev_number_str.parse::<u64>()?;
        let revision_height = rev_height_str.parse::<u64>()?;

        Height::new(revision_number, revision_height)
            .map_err(|_| DecodingError::invalid_raw_data("height of 0 not allowed"))
    }
}

impl From<Height> for String {
    fn from(height: Height) -> Self {
        format!("{}-{}", height.revision_number, height.revision_height)
    }
}

impl FromStr for Height {
    type Err = DecodingError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Height::try_from(s)
    }
}

#[test]
fn test_valid_height() {
    assert_eq!(
        "1-1".parse::<Height>().unwrap(),
        Height {
            revision_number: 1,
            revision_height: 1
        }
    );
    assert_eq!(
        "1-10".parse::<Height>().unwrap(),
        Height {
            revision_number: 1,
            revision_height: 10
        }
    );
}

#[test]
fn test_invalid_height() {
    assert!("0-0".parse::<Height>().is_err());
    assert!("0-".parse::<Height>().is_err());
    assert!("-0".parse::<Height>().is_err());
    assert!("-".parse::<Height>().is_err());
    assert!("1-1-1".parse::<Height>().is_err());

    let decoding_err = "1".parse::<Height>().unwrap_err();
    let decoding_err = decoding_err.to_string();
    assert!(decoding_err.contains("height `1` not properly formatted"));

    let decoding_err = "".parse::<Height>().unwrap_err();
    let decoding_err = decoding_err.to_string();
    assert!(decoding_err.contains("height `` not properly formatted"));
}
