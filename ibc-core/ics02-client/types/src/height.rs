//! Defines the core `Height` type used throughout the library

use core::cmp::Ordering;
use core::num::ParseIntError;
use core::str::FromStr;

use displaydoc::Display;
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
            return Err(ClientError::InvalidHeightResult);
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
    type Error = ClientError;

    fn try_from(raw_height: RawHeight) -> Result<Self, Self::Error> {
        Height::new(raw_height.revision_number, raw_height.revision_height)
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

/// Encodes all errors related to chain heights
#[derive(Debug, Display, PartialEq, Eq)]
pub enum HeightError {
    /// cannot convert into a `Height` type from string `{height}`
    HeightConversion {
        height: String,
        error: ParseIntError,
    },
    /// attempted to parse an invalid zero height
    ZeroHeight,
    /// the height(`{raw_height}`) is not valid format, this format must be used: \[revision_number\]-\[revision_height\]
    InvalidFormat { raw_height: String },
}

#[cfg(feature = "std")]
impl std::error::Error for HeightError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self {
            HeightError::HeightConversion { error: e, .. } => Some(e),
            HeightError::ZeroHeight | HeightError::InvalidFormat { .. } => None,
        }
    }
}

impl TryFrom<&str> for Height {
    type Error = HeightError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let (rev_number_str, rev_height_str) =
            value
                .split_once('-')
                .ok_or_else(|| HeightError::InvalidFormat {
                    raw_height: value.to_owned(),
                })?;

        let revision_number =
            rev_number_str
                .parse::<u64>()
                .map_err(|e| HeightError::HeightConversion {
                    height: value.to_owned(),
                    error: e,
                })?;

        let revision_height =
            rev_height_str
                .parse::<u64>()
                .map_err(|e| HeightError::HeightConversion {
                    height: value.to_owned(),
                    error: e,
                })?;

        Height::new(revision_number, revision_height).map_err(|_| HeightError::ZeroHeight)
    }
}

impl From<Height> for String {
    fn from(height: Height) -> Self {
        format!("{}-{}", height.revision_number, height.revision_height)
    }
}

impl FromStr for Height {
    type Err = HeightError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Height::try_from(s)
    }
}

#[test]
fn test_valid_height() {
    assert_eq!(
        "1-1".parse::<Height>(),
        Ok(Height {
            revision_number: 1,
            revision_height: 1
        })
    );
    assert_eq!(
        "1-10".parse::<Height>(),
        Ok(Height {
            revision_number: 1,
            revision_height: 10
        })
    );
}

#[test]
fn test_invalid_height() {
    assert_eq!(
        HeightError::ZeroHeight,
        "0-0".parse::<Height>().unwrap_err()
    );
    assert!("0-".parse::<Height>().is_err());
    assert!("-0".parse::<Height>().is_err());
    assert!("-".parse::<Height>().is_err());
    assert!("1-1-1".parse::<Height>().is_err());
    assert_eq!(
        "1".parse::<Height>(),
        Err(HeightError::InvalidFormat {
            raw_height: "1".to_owned()
        })
    );
    assert_eq!(
        "".parse::<Height>(),
        Err(HeightError::InvalidFormat {
            raw_height: "".to_owned()
        })
    );
}
