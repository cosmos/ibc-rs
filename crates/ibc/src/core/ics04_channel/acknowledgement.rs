//! Defines acknowledgment types used by various IBC messages and applications.

use core::fmt::{Display, Error as FmtError, Formatter};
use derive_more::Into;

use super::error::PacketError;
use crate::prelude::*;

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
/// [`Acknowledgement`](super::acknowledgement::Acknowledgement) based on the
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
/// [`AcknowledgementStatus`](crate::core::ics04_channel::acknowledgement::AcknowledgementStatus)
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
            AcknowledgementStatus::Success(v) => write!(f, "{v}"),
            AcknowledgementStatus::Error(v) => write!(f, "{v}"),
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::applications::transfer::{ack_success_b64, error::TokenTransferError};

    #[test]
    fn test_ack_ser() {
        fn ser_json_assert_eq(ack: AcknowledgementStatus, json_str: &str) {
            let ser = serde_json::to_string(&ack).unwrap();
            assert_eq!(ser, json_str)
        }

        ser_json_assert_eq(
            AcknowledgementStatus::success(ack_success_b64()),
            r#"{"result":"AQ=="}"#,
        );
        ser_json_assert_eq(
            AcknowledgementStatus::error(TokenTransferError::PacketDataDeserialization.into()),
            r#"{"error":"failed to deserialize packet data"}"#,
        );
    }

    #[test]
    fn test_ack_success_to_vec() {
        let ack_success: Vec<u8> = AcknowledgementStatus::success(ack_success_b64()).into();

        // Check that it's the same output as ibc-go
        // Note: this also implicitly checks that the ack bytes are non-empty,
        // which would make the conversion to `Acknowledgement` panic
        assert_eq!(ack_success, r#"{"result":"AQ=="}"#.as_bytes());
    }

    #[test]
    fn test_ack_error_to_vec() {
        let ack_error: Vec<u8> =
            AcknowledgementStatus::error(TokenTransferError::PacketDataDeserialization.into())
                .into();

        // Check that it's the same output as ibc-go
        // Note: this also implicitly checks that the ack bytes are non-empty,
        // which would make the conversion to `Acknowledgement` panic
        assert_eq!(
            ack_error,
            r#"{"error":"failed to deserialize packet data"}"#.as_bytes()
        );
    }

    #[test]
    fn test_ack_de() {
        fn de_json_assert_eq(json_str: &str, ack: AcknowledgementStatus) {
            let de = serde_json::from_str::<AcknowledgementStatus>(json_str).unwrap();
            std::println!("de: {:?}", de);
            std::println!("ack: {:?}", ack);
            assert_eq!(de, ack)
        }

        de_json_assert_eq(
            r#"{"result":"AQ=="}"#,
            AcknowledgementStatus::success(ack_success_b64()),
        );
        de_json_assert_eq(
            r#"{"error":"failed to deserialize packet data"}"#,
            AcknowledgementStatus::error(TokenTransferError::PacketDataDeserialization.into()),
        );

        assert!(serde_json::from_str::<AcknowledgementStatus>(r#"{"success":"AQ=="}"#).is_err());
    }
}
