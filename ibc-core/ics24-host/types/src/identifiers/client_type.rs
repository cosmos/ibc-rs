//! Defines the `ClientType` format, typically used in chain IDs.

use core::str::FromStr;

use ibc_primitives::prelude::*;

use super::ClientId;
use crate::error::IdentifierError;
use crate::validate::validate_client_type;

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
/// Type of the client, depending on the specific consensus algorithm.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, derive_more::Display)]
pub struct ClientType(String);

impl ClientType {
    /// Constructs a new `ClientType` from the given `String` if it ends with a valid client identifier.
    pub fn new(client_type: &str) -> Result<Self, IdentifierError> {
        let client_type = client_type.trim();
        validate_client_type(client_type).map(|()| Self(client_type.into()))
    }

    /// Constructs a new [`ClientId`] with this types client type and given
    /// `counter`.
    ///
    /// This is equivalent to `ClientId::new(self.as_str(), counter)` but
    /// infallible since client type is assumed to be correct.
    ///
    /// ```
    /// # use ibc_core_host_types::identifiers::ClientId;
    /// # use ibc_core_host_types::identifiers::ClientType;
    /// # use std::str::FromStr;
    /// let client_type = ClientType::from_str("07-tendermint").unwrap();
    /// let client_id = client_type.build_client_id(14);
    /// assert_eq!(client_id.as_str(), "07-tendermint-14");
    /// ```
    pub fn build_client_id(&self, counter: u64) -> ClientId {
        ClientId::format(self.as_str(), counter)
    }

    /// Yields this identifier as a borrowed `&str`
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for ClientType {
    type Err = IdentifierError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

#[cfg(test)]
mod test {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case::tendermint("07-tendermint")]
    #[case::wasm("08-wasm")]
    #[case::zero_double_digits("00-dummy")]
    #[case::zero_single_digit("0-dummy")]
    #[case::length_seven("1234567")]
    fn client_type_from_str(#[case] client_str: &str) {
        let client_type = ClientType::from_str(client_str).unwrap();
        assert_eq!(client_type.as_str(), client_str);
    }

    #[rstest]
    #[case::length_less_than_seven("00-foo")]
    #[case::length_six("123456")]
    fn client_type_from_str_fails(#[case] client_str: &str) {
        let client_type = ClientType::from_str(client_str);
        assert!(client_type.is_err());
    }

    #[rstest]
    #[case::tendermint("07-tendermint", 118)]
    #[case::wasm("08-wasm", 2)]
    #[case::zero_counter("01-dummy", 0)]
    fn client_type_build_client_id(#[case] client_str: &str, #[case] counter: u64) {
        let client_type = ClientType::from_str(client_str).unwrap();
        let client_id = client_type.build_client_id(counter);
        assert_eq!(client_id.as_str(), format!("{}-{}", client_str, counter));
    }
}
