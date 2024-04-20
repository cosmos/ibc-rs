use ibc_primitives::prelude::*;

use crate::error::IdentifierError as Error;
use crate::identifiers::{ChannelId, ConnectionId};

const VALID_SPECIAL_CHARS: &str = "._+-#[]<>";

/// Checks if the identifier only contains valid characters as specified in the
/// [`ICS-24`](https://github.com/cosmos/ibc/tree/main/spec/core/ics-024-host-requirements#paths-identifiers-separators)]
/// spec.
pub fn validate_identifier_chars(id: &str) -> Result<(), Error> {
    // Check that the identifier comprises only valid characters:
    // - Alphanumeric
    // - `.`, `_`, `+`, `-`, `#`
    // - `[`, `]`, `<`, `>`
    if !id
        .chars()
        .all(|c| c.is_alphanumeric() || VALID_SPECIAL_CHARS.contains(c))
    {
        return Err(Error::InvalidCharacter { id: id.into() });
    }

    // All good!
    Ok(())
}

/// Checks if the identifier forms a valid identifier with the given min/max length as specified in the
/// [`ICS-24`](https://github.com/cosmos/ibc/tree/main/spec/core/ics-024-host-requirements#paths-identifiers-separators)]
/// spec.
pub fn validate_identifier_length(id: &str, min: u64, max: u64) -> Result<(), Error> {
    // Make sure min is at least one so we reject empty identifiers.
    let min = min.max(1);
    let length = id.len() as u64;
    if (min..=max).contains(&length) {
        Ok(())
    } else {
        Err(Error::InvalidLength {
            id: id.into(),
            min,
            max,
        })
    }
}

/// Checks if a prefix forms a valid identifier with the given min/max identifier's length.
/// The prefix must be between `min_id_length - 2`, considering `u64::MIN` (1 char) and "-"
/// and `max_id_length - 21` characters, considering `u64::MAX` (20 chars) and "-".
pub fn validate_prefix_length(
    prefix: &str,
    min_id_length: u64,
    max_id_length: u64,
) -> Result<(), Error> {
    // Prefix must be at least `min_id_length - 2` characters long since the
    // shortest identifier we can construct is `{prefix}-0` which extends prefix
    // by 2 characters.
    let min = min_id_length.saturating_sub(2);
    // Prefix must be at most `max_id_length - 21` characters long since the
    // longest identifier we can construct is `{prefix}-{u64::MAX}` which
    // extends prefix by 21 characters.
    let max = max_id_length.saturating_sub(21);

    validate_identifier_length(prefix, min, max)
}

/// Checks if the identifier is a valid named u64 index: {name}-{u64}.
/// Example: "connection-0", "connection-100", "channel-0", "channel-100".
pub fn validate_named_u64_index(id: &str, name: &str) -> Result<(), Error> {
    let number_s = id
        .strip_prefix(name)
        .ok_or_else(|| Error::InvalidPrefix { prefix: id.into() })?
        .strip_prefix('-')
        .ok_or_else(|| Error::InvalidPrefix { prefix: id.into() })?;

    if number_s.starts_with('0') && number_s.len() > 1 {
        return Err(Error::InvalidPrefix { prefix: id.into() });
    }

    _ = number_s
        .parse::<u64>()
        .map_err(|_| Error::InvalidPrefix { prefix: id.into() })?;

    Ok(())
}

/// Default validator function for the Client types.
pub fn validate_client_type(id: &str) -> Result<(), Error> {
    validate_identifier_chars(id)?;
    validate_prefix_length(id, 9, 64)
}

/// Default validator function for Client identifiers.
///
/// A valid client identifier must be between 9-64 characters as specified in
/// the ICS-24 spec.
pub fn validate_client_identifier(id: &str) -> Result<(), Error> {
    validate_identifier_chars(id)?;
    validate_identifier_length(id, 9, 64)
}

/// Default validator function for Connection identifiers.
///
/// A valid connection identifier must be between 10-64 characters as specified
/// in the ICS-24 spec.
pub fn validate_connection_identifier(id: &str) -> Result<(), Error> {
    validate_identifier_chars(id)?;
    validate_identifier_length(id, 10, 64)?;
    validate_named_u64_index(id, ConnectionId::prefix())?;
    Ok(())
}

/// Default validator function for Port identifiers.
///
/// A valid port identifier must be between 2-128 characters as specified in the
/// ICS-24 spec.
pub fn validate_port_identifier(id: &str) -> Result<(), Error> {
    validate_identifier_chars(id)?;
    validate_identifier_length(id, 2, 128)
}

/// Default validator function for Channel identifiers.
///
/// A valid channel identifier must be between 8-64 characters as specified in
/// the ICS-24 spec.
pub fn validate_channel_identifier(id: &str) -> Result<(), Error> {
    validate_identifier_chars(id)?;
    validate_identifier_length(id, 8, 64)?;
    validate_named_u64_index(id, ChannelId::prefix())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[test]
    fn parse_invalid_port_id_min() {
        // invalid min port id
        let id = validate_port_identifier("p");
        assert!(id.is_err())
    }

    #[test]
    fn parse_invalid_port_id_max() {
        // invalid max port id (test string length is 130 chars)
        let id = validate_port_identifier(
            "9anxkcme6je544d5lnj46zqiiiygfqzf8w4bjecbnyj4lj6s7zlpst67yln64tixp9anxkcme6je544d5lnj46zqiiiygfqzf8w4bjecbnyj4lj6s7zlpst67yln64tixp",
        );
        assert!(id.is_err())
    }

    #[test]
    fn parse_invalid_connection_id_min() {
        // invalid min connection id
        let id = validate_connection_identifier("connect01");
        assert!(id.is_err())
    }

    #[test]
    fn parse_connection_id_max() {
        // invalid max connection id (test string length is 65)
        let id = validate_connection_identifier(
            "ihhankr30iy4nna65hjl2wjod7182io1t2s7u3ip3wqtbbn1sl0rgcntqc540r36r",
        );
        assert!(id.is_err())
    }

    #[test]
    fn parse_invalid_connection_id_indexed() {
        // valid connection id with index
        validate_connection_identifier("connection-0").expect("success");
        validate_connection_identifier("connection-123").expect("success");
        validate_connection_identifier("connection-18446744073709551615").expect("success");
    }

    #[test]
    fn parse_invalid_connection_id_non_indexed() {
        // invalid indexing for connection id
        validate_connection_identifier("connection-0123").expect_err("failure");
        validate_connection_identifier("connection0123").expect_err("failure");
        validate_connection_identifier("connection000").expect_err("failure");
        // 1 << 64 = 18446744073709551616
        validate_connection_identifier("connection-18446744073709551616").expect_err("failure");
    }

    #[test]
    fn parse_invalid_channel_id_min() {
        // invalid channel id, must be at least 8 characters
        let id = validate_channel_identifier("channel");
        assert!(id.is_err())
    }

    #[test]
    fn parse_channel_id_max() {
        // invalid channel id (test string length is 65)
        let id = validate_channel_identifier(
            "ihhankr30iy4nna65hjl2wjod7182io1t2s7u3ip3wqtbbn1sl0rgcntqc540r36r",
        );
        assert!(id.is_err())
    }

    #[test]
    fn parse_invalid_channel_id_indexed() {
        // valid channel id with index
        validate_channel_identifier("channel-0").expect("success");
        validate_channel_identifier("channel-123").expect("success");
        validate_channel_identifier("channel-18446744073709551615").expect("success");
    }

    #[test]
    fn parse_invalid_channel_id_non_indexed() {
        // invalid indexing for channel id
        validate_channel_identifier("channel-0123").expect_err("failure");
        validate_channel_identifier("channel0123").expect_err("failure");
        validate_channel_identifier("channel000").expect_err("failure");
        // 1 << 64 = 18446744073709551616
        validate_channel_identifier("channel-18446744073709551616").expect_err("failure");
    }

    #[test]
    fn parse_invalid_client_id_min() {
        // invalid min client id
        let id = validate_client_identifier("client");
        assert!(id.is_err())
    }

    #[test]
    fn parse_client_id_max() {
        // invalid max client id (test string length is 65)
        let id = validate_client_identifier(
            "f0isrs5enif9e4td3r2jcbxoevhz6u1fthn4aforq7ams52jn5m48eiesfht9ckpn",
        );
        assert!(id.is_err())
    }

    #[test]
    fn parse_invalid_id_chars() {
        // invalid id chars
        let id = validate_identifier_chars("channel@01");
        assert!(id.is_err())
    }

    #[test]
    fn validate_chars_empty_id() {
        // validate_identifier_chars allows empty identifiers
        assert!(validate_identifier_chars("").is_ok());
    }

    #[test]
    fn validate_length_empty_id() {
        // validate_identifier_length does not allow empty identifiers
        assert!(validate_identifier_length("", 0, 64).is_err());
    }

    #[test]
    fn validate_min_gt_max_constraints() {
        // validate_identifier_length rejects the id if min > max.
        assert!(validate_identifier_length("foobar", 5, 3).is_err());
    }

    #[test]
    fn parse_invalid_id_path_separator() {
        // invalid id with path separator
        let id = validate_identifier_chars("id/1");
        assert!(id.is_err())
    }

    #[test]
    fn parse_healthy_client_type() {
        let id = validate_client_type("07-tendermint");
        assert!(id.is_ok())
    }

    #[test]
    fn parse_invalid_short_client_type() {
        let id = validate_client_type("<7Char");
        assert!(id.is_err())
    }

    #[test]
    fn parse_invalid_lengthy_client_type() {
        let id = validate_client_type("InvalidClientTypeWithLengthOfClientId>65Char");
        assert!(id.is_err())
    }

    #[rstest]
    #[case::zero_min_length("", 0, 64, false)]
    #[case::empty_prefix("", 1, 64, false)]
    #[case::max_is_low("a", 1, 10, false)]
    #[case::min_greater_than_max("foobar", 5, 3, false)]
    #[case::u64_max_is_too_big("a", 3, 21, false)]
    #[case::u64_min_is_too_small("a", 4, 22, false)]
    #[case::u64_min_max_boundary("a", 3, 22, true)]
    #[case("chainA", 1, 32, true)]
    #[case("chainA", 1, 64, true)]
    fn test_prefix_length_validation(
        #[case] prefix: &str,
        #[case] min: u64,
        #[case] max: u64,
        #[case] success: bool,
    ) {
        let result = validate_prefix_length(prefix, min, max);
        assert_eq!(result.is_ok(), success);
    }

    #[rstest]
    #[case::zero_padded("channel", "001", false)]
    #[case::only_zero("connection", "000", false)]
    #[case::zero("channel", "0", true)]
    #[case::one("connection", "1", true)]
    #[case::n1234("channel", "1234", true)]
    #[case::u64_max("chan", "18446744073709551615", true)]
    #[case::u64_max_plus_1("chan", "18446744073709551616", false)]
    fn test_named_index_validation(#[case] name: &str, #[case] id: &str, #[case] success: bool) {
        let result = validate_named_u64_index(format!("{name}-{id}").as_str(), name);
        assert_eq!(result.is_ok(), success, "{result:?}");
    }
}
