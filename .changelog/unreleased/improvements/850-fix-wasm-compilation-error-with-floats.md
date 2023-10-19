- Fix compilation issue with Wasm envs because of floats
  - Use `serde-json-wasm` dependency instead of `serde-json` for no-floats support
  - Add CI test to include CosmWasm compilation check
([\#850](https://github.com/cosmos/ibc-rs/issues/850))
