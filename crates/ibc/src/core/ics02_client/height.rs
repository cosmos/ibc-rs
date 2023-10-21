//! Defines the core `Height` type used throughout the library

use core::num::{NonZeroU64, ParseIntError};
use core::str::FromStr;

use displaydoc::Display;
use ibc_proto::ibc::core::client::v1::Height as RawHeight;
use ibc_proto::protobuf::Protobuf;

use crate::core::ics02_client::error::ClientError;
use crate::prelude::*;

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
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Height {
    /// Previously known as "epoch"
    revision_number: u64,

    /// The height of a block
    revision_height: NonZeroU64,
}

impl Height {
    pub fn new(revision_number: u64, revision_height: u64) -> Result<Self, ClientError> {
        NonZeroU64::new(revision_height)
            .map(|revision_height| Self {
                revision_number,
                revision_height,
            })
            .ok_or(ClientError::InvalidHeight)
    }

    pub fn min(revision_number: u64) -> Self {
        Self {
            revision_number,
            revision_height: NonZeroU64::MIN,
        }
    }

    pub fn revision_number(&self) -> u64 {
        self.revision_number
    }

    pub fn revision_height(&self) -> u64 {
        self.revision_height.get()
    }

    pub fn add(&self, delta: u64) -> Height {
        let revision_height = self
            .revision_height
            .checked_add(delta)
            .expect("height should never overflow u64");
        Height {
            revision_number: self.revision_number,
            revision_height,
        }
    }

    pub fn increment(&self) -> Height {
        self.add(1)
    }

    pub fn sub(&self, delta: u64) -> Result<Height, ClientError> {
        let revision_height = self
            .revision_height
            .get()
            .checked_sub(delta)
            .and_then(NonZeroU64::new)
            .ok_or(ClientError::InvalidHeightResult)?;
        Ok(Height {
            revision_number: self.revision_number,
            revision_height,
        })
    }

    pub fn decrement(&self) -> Result<Height, ClientError> {
        self.sub(1)
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
            revision_height: ics_height.revision_height.get(),
        }
    }
}

impl core::fmt::Debug for Height {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        f.debug_struct("Height")
            .field("revision", &self.revision_number)
            .field("height", &self.revision_height.get())
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
#[derive(Debug, Display, PartialEq)]
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
            HeightError::ZeroHeight => None,
            HeightError::InvalidFormat { .. } => None,
        }
    }
}

impl TryFrom<&str> for Height {
    type Error = HeightError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let (rev_number_str, rev_height_str) = match value.split_once('-') {
            Some((rev_number_str, rev_height_str)) => (rev_number_str, rev_height_str),
            None => {
                return Err(HeightError::InvalidFormat {
                    raw_height: value.to_owned(),
                })
            }
        };

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
    assert_eq!("1-1".parse::<Height>(), Ok(Height::new(1, 1).unwrap()));
    assert_eq!("1-10".parse::<Height>(), Ok(Height::new(1, 10).unwrap()));
    assert_eq!("10-1".parse::<Height>(), Ok(Height::new(10, 1).unwrap()));
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

#[cfg(feature = "borsh")]
#[test]
fn test_borsh() {
    use borsh::BorshDeserialize;

    let height = Height::new(42, 24).unwrap();
    let encoded = borsh::to_vec(&height).unwrap();
    assert_eq!(
        &[42, 0, 0, 0, 0, 0, 0, 0, 24, 0, 0, 0, 0, 0, 0, 0],
        encoded.as_slice()
    );
    let decoded = Height::try_from_slice(&encoded).unwrap();
    assert_eq!(height, decoded);

    // Test 0 revision height doesn’t deserialize.
    let encoded = [42, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    Height::try_from_slice(&encoded).unwrap_err();
}

#[cfg(feature = "serde")]
#[test]
fn test_serde() {
    let height = Height::new(42, 24).unwrap();
    let encoded = serde_json::to_string(&height).unwrap();
    assert_eq!(r#"{"revision_number":42,"revision_height":24}"#, encoded);
    let decoded = serde_json::from_str::<Height>(&encoded).unwrap();
    assert_eq!(height, decoded);

    // Test 0 revision height doesn’t deserialize.
    let encoded = r#"{"revision_number":42,"revision_height":0}"#;
    serde_json::from_str::<Height>(encoded).unwrap_err();
}
