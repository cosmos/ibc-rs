# `ibc-apps`

This crate is top-level library re-exports implemented Inter-Blockchain
Communication (IBC) applications in Rust serves as a centralized hub,
simplifying the process of importing and integrating various IBC applications
into your blockchain. IBC is a distributed protocol that enables communication
between distinct sovereign blockchains and IBC applications is the part of the
protocol that abstracts away the core transport and authentication layers and
focuses solely on business logics.

The structure within the `ibc-apps` crate is designed to provide flexibility for
external users. It allows you to utilize the own `ibc-apps` crate
comprehensively or selectively import specific libraries, whether you need a
certain IBC application (e.g. `ibc-app-transfer` crate) or only their associated
data structures (e.g. `ibc-app-transfer-types`). This versatility empowers
hosts, including chain integrators, relayers, or any IBC tooling projects, to
build their solutions on top of the layers that best suit their requirements.

## Libraries

Currently, the `ibc-apps` crate contains the implementation of the following IBC
applications:

### ICS-20: Fungible Token Transfer Application

- [ibc-app-transfer](crates/ibc-apps/ics20-transfer)
- [ibc-app-transfer-types](crates/ibc-apps/ics20-transfer/types)

## Contributing

IBC is specified in English in the [cosmos/ibc repo][ibc]. Any
protocol changes or clarifications should be contributed there.

If you're interested in contributing, please comment on an issue or open a new
one!

See also [CONTRIBUTING.md](./../../CONTRIBUTING.md).

## Resources

- [IBC Website][ibc-homepage]
- [IBC Specification][ibc]
- [IBC Go implementation][ibc-go]

[//]: # (general links)
[ibc]: https://github.com/cosmos/ibc
[ibc-go]: https://github.com/cosmos/ibc-go
[ibc-homepage]: https://cosmos.network/ibc
