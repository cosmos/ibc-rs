# IBC Applications

This crate is a top-level library that re-exports implementations of
Inter-Blockchain Communication (IBC) applications. It serves as a meta-crate,
simplifying the process of importing and integrating various IBC applications
into your blockchain. IBC is a distributed protocol that enables communication
between distinct sovereign blockchains. IBC applications abstract away the core
transport, authentication, and ordering (TAO) layers, letting blockchain app
developers focus solely on implementing business logic.

The structure within the `ibc-apps` crate is designed to provide flexibility for
external users. It allows users to either utilize the entire `ibc-apps` crate,
or selectively import specific sub-crates, whether they need a certain IBC
application (e.g. `ibc-app-transfer` crate) or only its associated data
structures (e.g. `ibc-app-transfer-types`). This versatility empowers hosts,
including chain integrators, relayers, or any IBC tooling projects, to build
their solutions on top of the layers that best suit their requirements.

## Sub-Crates

The `ibc-apps` crate contains the implementation of the following IBC
applications:

### ICS-20: Fungible Token Transfer Application

- [ibc-app-transfer](./../ibc-apps/ics20-transfer)
- [ibc-app-transfer-types](./../ibc-apps/ics20-transfer/types)

### ICS-721: Non-Fungible Token Transfer Application

- [ibc-app-nft-transfer](./../ibc-apps/ics721-nft-transfer)
- [ibc-app-nft-transfer-types](./../ibc-apps/ics721-nft-transfer/types)

## Contributing

IBC is specified in English in the [cosmos/ibc
repo](https://github.com/cosmos/ibc). Any protocol changes or clarifications
should be contributed there.

If you're interested in contributing, please take a look at the
[CONTRIBUTING](./../CONTRIBUTING.md) guidelines. We welcome and appreciate
community contributions!
