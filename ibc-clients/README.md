# IBC Clients

This top-level crate re-exports Inter-Blockchain Communication (IBC)
implementations of light clients. It serves as a meta-crate, simplifying the
process of importing and integrating various IBC clients into your blockchain.
IBC is a distributed protocol that enables communication between distinct
sovereign blockchains and IBC light clients track the consensus states and proof
specs of external blockchains, which are required to properly verify proofs
against the client's consensus state.

The structure within the `ibc-clients` crate is designed to provide flexibility
for external users. It allows you to utilize the own `ibc-clients` crate or
selectively import specific libraries, whether you need a certain IBC client
implementation (e.g. `ibc-client-tendermint` crate) or only its associated data
structures (e.g. `ibc-core-tendermint-types`). This versatility empowers hosts,
including chain integrators, relayers, or any IBC tooling projects, to build
their solutions on top of the layers that best suit their requirements.

## Sub-Crates

Currently, the `ibc-clients` crate contains the implementation of the following
IBC light clients:

### ICS-07: Tendermint Light Client

- [ibc-client-tendermint-types](./ics07-tendermint/types): Data Structures
- [ibc-client-tendermint](./ics07-tendermint): Implementation
- [ibc-client-tendermint-cw](./ics07-tendermint/cw-contract): CosmWasm Contract

> [!TIP]
> The pre-compiled CosmWasm contract for `ibc-client-tendermint-cw` is available
> as Github workflow artifacts at
> [_Actions_](https://github.com/cosmos/ibc-rs/actions/workflows/upload-cw-clients.yaml)
> tab. They can be downloaded
> [during a Github workflow](https://github.com/cosmos/ibc-rs/blob/1098f252c04152812f026520e28e323f3bc0507e/.github/workflows/upload-cw-clients.yaml#L87-L96)
> using `actions/download-artifact@v4` action.

### ICS-08: WASM Proxy Light Client

- [ibc-client-wasm-types](./ics08-wasm/types)

### CosmWasm Integration

- [ibc-client-cw](./cw-context): Types and Utilities for CosmWasm Integration
  - To utilize the CosmWasm contracts developed with this library, hosting
    environments must support the CosmWasm module and be using the version of
    `ibc-go` that supports the `08-wasm` proxy light client.

> [!CAUTION]
> The `ibc-client-cw` is currently in development and should not be
  deployed for production use. Users are advised to exercise caution and test
  thoroughly in non-production environments.

## Third-party Clients

Here, we list IBC third-party clients that are compatible with `ibc-rs`. You
should always audit the implementation of any third-party crate. If you have a
client that you'd like to be added to this list, please open a PR!

- [ICS 6: Solomachine](https://github.com/octopus-network/ics06-solomachine) by
  Octopus Network

## Contributing

IBC is specified in English in the [cosmos/ibc
repo](https://github.com/cosmos/ibc). Any protocol changes or clarifications
should be contributed there.

If you're interested in contributing, please take a look at the
[CONTRIBUTING](./../CONTRIBUTING.md) guidelines. We welcome and appreciate
community contributions!
