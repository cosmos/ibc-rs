- Remove IdentifierError::Empty and ContainSeparator variants as they
  are special cases of other existing errors.
  ([\#942](https://github.com/cosmos/ibc-rs/pull/942))
- Remove IdentifierError::InvalidLength::length field since it can be
  deduced from the id.
  ([\#942](https://github.com/cosmos/ibc-rs/pull/942))
