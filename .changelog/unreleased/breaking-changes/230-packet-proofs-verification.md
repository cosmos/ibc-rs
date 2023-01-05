- Refactor proof handlers to conduct proof verifications inline with the process function
  ([#230](https://github.com/cosmos/ibc-rs/issues/230))
  * Remove `ics04_channel/handler/verify` module
  * Remove `proofs` module and move respective proofs into each of the packet message structs
  * Apply naming conventions to packet messages types
