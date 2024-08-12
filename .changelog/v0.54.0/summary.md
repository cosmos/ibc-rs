This release includes a number of breaking changes, including separating the
packet timeout timestamp from the host `Timestamp` via defining a new bespoke
`TimeoutTimestamp` type. The `cosmwasm` feature flag has been removed. CosmWasm
implementations have also been migrated into their own separate workspace under
the `cosmwasm` directory in an effort to simplify dependency management.

Notable bug fixes include:

- Correctly preventing expired client creation
- Expiring clients when the elapsed time from a trusted header matches the
  trusting period
- Allowing user-defined upgrade paths for client upgrades rather than just
  defaulting to `UPGRADED_IBC_STATE`
- Preventing `Timestamp::nanoseconds` from panicking by disallowing negative
  values from `tendermint::Time`

Lastly, this release adds some new features, including allowing proof
verification methods to accept custom paths, thus allowing light client
developers to introduce custom path serialization logic into their applications.
CosmWasm response types have been refactored to match the `08-wasm` client API.

This release bumps the MSRV of ibc-rs to 1.72.1. `prost` has been bumped to 0.13.1.
`ibc-proto` has been bumped to 0.47.0. `tendermint` dependencies have been bumped
to 0.38.0. The `cosmwasm` dependency has also been bumped to 2.1.0.
