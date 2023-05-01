use crate::prelude::*;

use super::error::ValidationError as Error;

/// Path separator (ie. forward slash '/')
const PATH_SEPARATOR: char = '/';
const VALID_SPECIAL_CHARS: &str = "._+-#[]<>";

/// Default validator function for identifiers.
///
/// A valid identifier only contain valid characters, and be of a given min and
/// max length as specified in the
/// [`ICS-24`](https://github.com/cosmos/ibc/tree/main/spec/core/ics-024-host-requirements#paths-identifiers-separators)]
/// spec.
pub fn validate_identifier(id: &str, min: usize, max: usize) -> Result<(), Error> {
    assert!(max >= min);

    // Check identifier is not empty
    if id.is_empty() {
        return Err(Error::Empty);
    }

    // Check identifier does not contain path separators
    if id.contains(PATH_SEPARATOR) {
        return Err(Error::ContainSeparator { id: id.into() });
    }

    // Check identifier length is between given min/max
    if id.len() < min || id.len() > max {
        return Err(Error::InvalidLength {
            id: id.into(),
            length: id.len(),
            min,
            max,
        });
    }

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

/// Checks if the prefix can form a valid identifier with the given min/max identifier's length.
fn validate_prefix_epoch_format(
    prefix: &str,
    min_id_length: usize,
    max_id_length: usize,
) -> Result<(), Error> {
    // Checks if the prefix is not blank
    if prefix.is_empty() {
        return Err(Error::Empty)?;
    }

    // Checks if the prefix forms a valid identifier when used with `0`
    validate_identifier(
        &format!("{prefix}-{}", u64::MIN),
        min_id_length,
        max_id_length,
    )?;

    // Checks if the prefix forms a valid identifier when used with `u64::MAX`
    validate_identifier(
        &format!("{prefix}-{}", u64::MAX),
        min_id_length,
        max_id_length,
    )?;

    Ok(())
}

/// Default validator function for the Client types.
pub fn validate_client_type(id: &str) -> Result<(), Error> {
    validate_prefix_epoch_format(id, 9, 64)
}

/// Default validator function for Client identifiers.
///
/// A valid client identifier must be between 9-64 characters as specified in
/// the ICS-24 spec.
pub fn validate_client_identifier(id: &str) -> Result<(), Error> {
    validate_identifier(id, 9, 64)
}

/// Default validator function for Connection identifiers.
///
/// A valid connection identifier must be between 10-64 characters as specified
/// in the ICS-24 spec.
pub fn validate_connection_identifier(id: &str) -> Result<(), Error> {
    validate_identifier(id, 10, 64)
}

/// Default validator function for Port identifiers.
///
/// A valid port identifier must be between 2-128 characters as specified in the
/// ICS-24 spec.
pub fn validate_port_identifier(id: &str) -> Result<(), Error> {
    validate_identifier(id, 2, 128)
}

/// Default validator function for Channel identifiers.
///
/// A valid channel identifier must be between 8-64 characters as specified in
/// the ICS-24 spec.
pub fn validate_channel_identifier(id: &str) -> Result<(), Error> {
    validate_identifier(id, 8, 64)
}

#[cfg(test)]
mod tests {
    use crate::core::ics24_host::validate::{
        validate_channel_identifier, validate_client_identifier, validate_client_type,
        validate_connection_identifier, validate_identifier, validate_port_identifier,
    };
    use test_log::test;

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
        let id = validate_identifier("channel@01", 1, 10);
        assert!(id.is_err())
    }

    #[test]
    fn parse_invalid_id_empty() {
        // invalid id empty
        let id = validate_identifier("", 1, 10);
        assert!(id.is_err())
    }

    #[test]
    fn parse_invalid_id_path_separator() {
        // invalid id with path separator
        let id = validate_identifier("id/1", 1, 10);
        assert!(id.is_err())
    }

    #[test]
    fn parse_healthy_client_type() {
        let id = validate_client_type("07-tendermint");
        assert!(id.is_ok())
    }

    #[test]
    fn parse_short_client_type() {
        let id = validate_client_type("<7Char");
        assert!(id.is_err())
    }

    #[test]
    fn parse_lengthy_client_type() {
        let id = validate_client_type("InvalidClientTypeWithLengthOfClientId>65Char");
        assert!(id.is_err())
    }
}
