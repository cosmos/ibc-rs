use core::fmt::{Display, Error as FmtError, Formatter};

use super::error::TokenTransferError;
use crate::core::ics04_channel::msgs::acknowledgement::Acknowledgement;
use crate::prelude::*;

/// A string constant included in error acknowledgements.
/// NOTE: Changing this const is state machine breaking as acknowledgements are written into state
pub const ACK_ERR_STR: &str = "error handling packet on destination chain: see events for details";

/// A successful acknowledgement, equivalent to `base64::encode(0x01)`.
pub const ACK_SUCCESS_B64: &str = "AQ==";

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConstAckSuccess {
    #[cfg_attr(feature = "serde", serde(rename = "AQ=="))]
    Success,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TokenTransferAcknowledgement {
    /// Successful Acknowledgement
    /// e.g. `{"result":"AQ=="}`
    #[cfg_attr(feature = "serde", serde(rename = "result"))]
    Success(ConstAckSuccess),
    /// Error Acknowledgement
    /// e.g. `{"error":"cannot unmarshal ICS-20 transfer packet data"}`
    #[cfg_attr(feature = "serde", serde(rename = "error"))]
    Error(String),
}

impl TokenTransferAcknowledgement {
    pub fn success() -> Self {
        Self::Success(ConstAckSuccess::Success)
    }

    pub fn from_error(err: TokenTransferError) -> Self {
        Self::Error(format!("{ACK_ERR_STR}: {err}"))
    }

    pub fn is_successful(&self) -> bool {
        matches!(self, TokenTransferAcknowledgement::Success(_))
    }
}

impl Display for TokenTransferAcknowledgement {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        match self {
            TokenTransferAcknowledgement::Success(_) => write!(f, "{ACK_SUCCESS_B64}"),
            TokenTransferAcknowledgement::Error(err_str) => write!(f, "{err_str}"),
        }
    }
}

impl From<TokenTransferAcknowledgement> for Vec<u8> {
    fn from(ack: TokenTransferAcknowledgement) -> Self {
        // WARNING: Make sure all branches always return a non-empty vector.
        // Otherwise, the conversion to `Acknowledgement` will panic.
        match ack {
            TokenTransferAcknowledgement::Success(_) => r#"{"result":"AQ=="}"#.as_bytes().into(),
            TokenTransferAcknowledgement::Error(s) => alloc::format!(r#"{{"error":"{s}"}}"#).into(),
        }
    }
}

impl From<TokenTransferAcknowledgement> for Acknowledgement {
    fn from(ack: TokenTransferAcknowledgement) -> Self {
        let v: Vec<u8> = ack.into();

        v.try_into()
            .expect("token transfer internal error: ack is never supposed to be empty")
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_ack_ser() {
        fn ser_json_assert_eq(ack: TokenTransferAcknowledgement, json_str: &str) {
            let ser = serde_json::to_string(&ack).unwrap();
            assert_eq!(ser, json_str)
        }

        ser_json_assert_eq(
            TokenTransferAcknowledgement::success(),
            r#"{"result":"AQ=="}"#,
        );
        ser_json_assert_eq(
            TokenTransferAcknowledgement::Error(
                "cannot unmarshal ICS-20 transfer packet data".to_owned(),
            ),
            r#"{"error":"cannot unmarshal ICS-20 transfer packet data"}"#,
        );
    }

    #[test]
    fn test_ack_success_to_vec() {
        let ack_success: Vec<u8> = TokenTransferAcknowledgement::success().into();

        // Check that it's the same output as ibc-go
        // Note: this also implicitly checks that the ack bytes are non-empty,
        // which would make the conversion to `Acknowledgement` panic
        assert_eq!(ack_success, r#"{"result":"AQ=="}"#.as_bytes());
    }

    #[test]
    fn test_ack_error_to_vec() {
        let ack_error: Vec<u8> = TokenTransferAcknowledgement::Error(
            "cannot unmarshal ICS-20 transfer packet data".to_string(),
        )
        .into();

        // Check that it's the same output as ibc-go
        // Note: this also implicitly checks that the ack bytes are non-empty,
        // which would make the conversion to `Acknowledgement` panic
        assert_eq!(
            ack_error,
            r#"{"error":"cannot unmarshal ICS-20 transfer packet data"}"#.as_bytes()
        );
    }

    #[test]
    fn test_ack_de() {
        fn de_json_assert_eq(json_str: &str, ack: TokenTransferAcknowledgement) {
            let de = serde_json::from_str::<TokenTransferAcknowledgement>(json_str).unwrap();
            assert_eq!(de, ack)
        }

        de_json_assert_eq(
            r#"{"result":"AQ=="}"#,
            TokenTransferAcknowledgement::success(),
        );
        de_json_assert_eq(
            r#"{"error":"cannot unmarshal ICS-20 transfer packet data"}"#,
            TokenTransferAcknowledgement::Error(
                "cannot unmarshal ICS-20 transfer packet data".to_owned(),
            ),
        );

        assert!(
            serde_json::from_str::<TokenTransferAcknowledgement>(r#"{"result":"AQ="}"#).is_err()
        );
        assert!(
            serde_json::from_str::<TokenTransferAcknowledgement>(r#"{"success":"AQ=="}"#).is_err()
        );
    }
}
