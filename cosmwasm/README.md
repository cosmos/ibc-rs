<div align="center">
    <h1>CosmWasm Integration</h1>
</div>

This collection is designed to offer libraries that facilitate the
implementation of various `ibc-rs` core, clients and applications as CosmWasm
contracts. Currently, the available packages are:

## IBC Clients

- [ibc-client-cw](./ibc-clients/cw-context): Provides utilities and a generic `Context` object
  to streamline the implementation of any ibc-rs-powered light clients as
  CosmWasm contracts.
  - To utilize the CosmWasm contracts developed with this library, hosting
    environments must support the CosmWasm module and be using the version of
    `ibc-go` that supports the `08-wasm` proxy light client.

> [!CAUTION]
> The `ibc-client-cw` is currently in development and should not be
  deployed for production use. Users are advised to exercise caution and test
  thoroughly in non-production environments.

- [ibc-client-tendermint-cw](./ibc-clients/ics07-tendermint): CosmWasm Contract

> [!TIP]
> The pre-compiled CosmWasm contract for `ibc-client-tendermint-cw` is available
> as Github workflow artifacts at
> [_Actions_](https://github.com/cosmos/ibc-rs/actions/workflows/upload-cw-clients.yaml)
> tab. They can be downloaded
> [during a Github workflow](https://github.com/cosmos/ibc-rs/blob/1098f252c04152812f026520e28e323f3bc0507e/.github/workflows/upload-cw-clients.yaml#L87-L96)
> using `actions/download-artifact@v4` action.
