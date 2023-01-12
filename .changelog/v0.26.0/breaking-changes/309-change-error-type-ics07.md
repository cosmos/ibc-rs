- Change type of `trusted_validator_set` field in
  `MisbehaviourTrustedValidatorHashMismatch` error variant from `ValidatorSet` to
  `Vec<Validator>` to avoid clippy catches
  ([#309](https://github.com/cosmos/ibc-rs/issues/309))