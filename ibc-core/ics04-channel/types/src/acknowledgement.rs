//! Defines acknowledgment types used by various IBC messages and applications.

use core::fmt::{Display, Error as FmtError, Formatter};

use derive_more::Into;
use ibc_primitives::prelude::*;

use super::error::PacketError;

/// A generic Acknowledgement type that modules may interpret as they like.
///
/// NOTE: An acknowledgement cannot be empty.
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
#[derive(Clone, Debug, PartialEq, Eq, Into)]
pub struct Acknowledgement(Vec<u8>);

impl Acknowledgement {
    // Returns the data as a slice of bytes.
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_slice()
    }
}

impl AsRef<[u8]> for Acknowledgement {
    fn as_ref(&self) -> &[u8] {
        self.0.as_slice()
    }
}

impl TryFrom<Vec<u8>> for Acknowledgement {
    type Error = PacketError;

    fn try_from(bytes: Vec<u8>) -> Result<Self, Self::Error> {
        if bytes.is_empty() {
            Err(PacketError::InvalidAcknowledgement)
        } else {
            Ok(Self(bytes))
        }
    }
}

/// Defines a convenience type for IBC applications to construct an
/// [`Acknowledgement`] based on the
/// success or failure of processing a received packet.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AcknowledgementStatus {
    /// Successful Acknowledgement
    /// e.g. `{"result":"AQ=="}`
    #[cfg_attr(feature = "serde", serde(rename = "result"))]
    Success(StatusValue),
    /// Error Acknowledgement
    /// e.g. `{"error":"cannot unmarshal ICS-20 transfer packet data"}`
    #[cfg_attr(feature = "serde", serde(rename = "error"))]
    Error(StatusValue),
}

/// A wrapper type that guards variants of
/// [`AcknowledgementStatus`]
/// against being constructed with an empty value.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StatusValue(String);

impl StatusValue {
    /// Constructs a new instance of `StatusValue` if the given value is not empty.
    pub fn new(value: impl ToString) -> Result<Self, PacketError> {
        let value = value.to_string();

        if value.is_empty() {
            return Err(PacketError::EmptyAcknowledgementStatus);
        }

        Ok(Self(value))
    }
}

impl Display for StatusValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        write!(f, "{status_value}", status_value = self.0)
    }
}

impl AcknowledgementStatus {
    /// Creates a success acknowledgement status with the given value.
    pub fn success(value: StatusValue) -> Self {
        Self::Success(value)
    }

    /// Creates an error acknowledgement status with the given value.
    pub fn error(value: StatusValue) -> Self {
        Self::Error(value)
    }

    /// Returns true if the acknowledgement status is successful.
    pub fn is_successful(&self) -> bool {
        matches!(self, AcknowledgementStatus::Success(_))
    }
}

impl Display for AcknowledgementStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        match self {
            AcknowledgementStatus::Success(v) | AcknowledgementStatus::Error(v) => write!(f, "{v}"),
        }
    }
}

/// Converts an acknowledgement result into a vector of bytes.
impl From<AcknowledgementStatus> for Vec<u8> {
    fn from(ack: AcknowledgementStatus) -> Self {
        // WARNING: Make sure all branches always return a non-empty vector.
        // Otherwise, the conversion to `Acknowledgement` will panic.
        match ack {
            AcknowledgementStatus::Success(v) => alloc::format!(r#"{{"result":"{v}"}}"#).into(),
            AcknowledgementStatus::Error(v) => alloc::format!(r#"{{"error":"{v}"}}"#).into(),
        }
    }
}

impl From<AcknowledgementStatus> for Acknowledgement {
    fn from(ack_status: AcknowledgementStatus) -> Self {
        let v: Vec<u8> = ack_status.into();

        v.try_into()
            .expect("token transfer internal error: ack is never supposed to be empty")
    }
}
