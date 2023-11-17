# `ibc-core`

This crate is top-level library re-exports implemented Inter-Blockchain
Communication (IBC) core transport and authentication layers in Rust serves as a
centralized hub, simplifying the process of importing and integrating various
IBC core components into your blockchain. IBC is a distributed protocol that
enables communication between distinct sovereign blockchains and IBC
applications is the part of the protocol that abstracts away the core transport
and authentication layers and focuses solely on business logics.

The structure within the `ibc-core` crate is designed to provide flexibility for
external users. It allows you to utilize the own `ibc-core` crate
comprehensively or selectively import specific libraries, whether you need a
certain IBC application (e.g. `ibc-core-client` crate) or only their associated
data structures (e.g. `ibc-core-client-types`). This versatility empowers
hosts, including chain integrators, relayers, or any IBC tooling projects, to
build their solutions on top of the layers that best suit their requirements.

## Libraries

Currently, the `ibc-core` crate contains the implementation of the following IBC
core libraries:

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
