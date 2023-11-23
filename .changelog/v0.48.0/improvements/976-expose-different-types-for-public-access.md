- Expose various fields, types and functions in `ibc-rs` as public including:
  - `validate` and `execute` handler functions for all the IBC message types.
  - `TYPE_URL` constants.
  - Any private fields within the domain message types.
  - Any private fields within the Tendermint `ClientState` and `ConsensusState`
  ([\#976](https://github.com/cosmos/ibc-rs/issues/976))
