use core::str::FromStr;

use derive_more::Into;
use ibc_primitives::prelude::*;

use crate::error::IdentifierError;
use crate::validate::{validate_client_identifier, validate_client_type};

#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
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
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Into, derive_more::Display)]
pub struct ClientId(String);

impl ClientId {
    /// Builds a new client identifier.
    ///
    /// Client identifiers are deterministically formed from two elements:
    /// a prefix derived from the client type `ctype`, and a monotonically
    /// increasing `counter`; these are separated by a dash "-".
    ///
    /// See also [`ClientType::build_client_id`](super::ClientType::build_client_id)
    /// method.
    ///
    /// # Example
    ///
    /// ```
    /// # use ibc_core_host_types::identifiers::ClientId;
    /// # use ibc_core_host_types::identifiers::ClientType;
    /// # use std::str::FromStr;
    /// let client_type = ClientType::from_str("07-tendermint").unwrap();
    /// let client_id = &client_type.build_client_id(0);
    /// assert_eq!(client_id.as_str(), "07-tendermint-0");
    /// ```
    pub fn new(client_type: &str, counter: u64) -> Result<Self, IdentifierError> {
        let client_type = client_type.trim();
        validate_client_type(client_type).map(|()| Self::format(client_type, counter))
    }

    pub(super) fn format(client_type: &str, counter: u64) -> Self {
        let client_id = format!("{client_type}-{counter}");
        if cfg!(debug_assertions) {
            validate_client_type(client_type).expect("valid client type");
            validate_client_identifier(&client_id).expect("valid client id");
        }
        Self(client_id)
    }

    /// Get this identifier as a borrowed `&str`
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Get this identifier as a borrowed byte slice
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }

    /// Check if the client identifier is for 08-wasm light client.
    pub fn is_wasm_client_id(&self) -> bool {
        const WASM_CLIENT_PREFIX: &str = "08-wasm-";

        // prefixed with wasm client type identifier.
        self.0.starts_with(WASM_CLIENT_PREFIX)
            // followed by non-empty string.
            && self.0.len() > WASM_CLIENT_PREFIX.len()
            // and the rest of the string is numeric.
            && self.0.chars().skip(WASM_CLIENT_PREFIX.len()).all(char::is_numeric)
    }
}

impl FromStr for ClientId {
    type Err = IdentifierError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        validate_client_identifier(s).map(|_| Self(s.to_string()))
    }
}

/// Equality check against string literal (satisfies &ClientId == &str).
/// ```
/// use core::str::FromStr;
/// use ibc_core_host_types::identifiers::ClientId;
/// let client_id = ClientId::from_str("clientidtwo");
/// assert!(client_id.is_ok());
/// client_id.map(|id| {assert_eq!(&id, "clientidtwo")});
/// ```
impl PartialEq<str> for ClientId {
    fn eq(&self, other: &str) -> bool {
        self.as_str().eq(other)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rstest::rstest]
    #[case("08-wasm-1", true)]
    #[case("08-wasm-", false)]
    #[case("08-wasm-abc", false)]
    #[case("08-wasm-1-2", false)]
    #[case("08-wasm", false)]
    #[case("wasm", false)]
    #[case("08-", false)]
    fn test_is_wasm_client_id(#[case] client_id: &str, #[case] expected: bool) {
        assert_eq!(
            matches!(
                client_id.parse().map(|id: ClientId| id.is_wasm_client_id()),
                Ok(true)
            ),
            expected
        );
    }
}
