- Remove IdentifierError::Empty and ContainSeparator variants as they
  are special cases of other existing errors.
  ([\#978](https://github.com/cosmos/ibc-rs/issues/978))
- Remove IdentifierError::InvalidLength::length field since it can be
  deduced from the id.
  ([\#978](https://github.com/cosmos/ibc-rs/issues/978))
