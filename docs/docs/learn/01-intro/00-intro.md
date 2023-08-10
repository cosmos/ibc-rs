---
slug: /
title: Introduction
displayed_sidebar: learnSidebar
sidebar_position: 0
---

# Welcome to the IBC-rs Documentation

IBC-rs is an open-source project implementing Inter-Blockchain Communication
(IBC) in Rust. It is developed primarily by [Informal Systems
Inc](https://informal.systems/) and aims to bring the power of IBC protocol to a
wider range of blockchains, developers, and operators by enabling new business
models across the blockchain industry, ultimately paving the way for a truly
interconnected blockchain ecosystem.

## Getting Started

To learn about Inter-Blockchain Communication (IBC), following resources provide
a good starting point to review terminology, concepts, and specifications:

* [**IBC Website**](https://cosmos.network/ibc/) - The official IBC website.
* [**IBC Specification**](https://github.com/cosmos/ibc) - The IBC specification
  repository.

## Supported Interchain Standards

IBC-rs is designed to serve the complete suite of IBC protocol standards. As
such, it supports all the core specifications essential for TAO (Transfer,
Authentication, Ordering), spanning from ICS-02 to ICS-05, as well as ICS-24 and
ICS-26. However, a few
[divergences](https://github.com/cosmos/ibc-rs/tree/main/crates/ibc#divergence-from-the-interchain-standards-ics)
from the interchain standards (ICS) are introduced to accommodate the
implementation.

* **IBC Light Clients**
  * [**ICS-7: Tendermint**](./../learn/clients/tendermint)
* **IBC Applications**
  * [**ICS-20: Fungible Token Transfer**](./../learn/apps/token-transfer)
  * [**ICS-27: Interchain Accounts**](./../learn/apps/interchain-accounts) (Work in progress)

## Latest Status

Check out our [project board](https://github.com/orgs/cosmos/projects/27) for
  the latest status of the IBC-rs implementation.

## Explore Implementation

To get a better understanding of IBC-rs implementation, check out the following sections:

* [**Overview**](./../learn/overview/overview) - Provides an overview of IBC-rs and
  how it implements the IBC protocol.
* [**ADRs**](./../../developers/06-architecture/README.md) - Learn about the
  architectural design decisions.
* [**FAQ Page**](https://github.com/cosmos/ibc-rs/wiki/FAQ) - Browse frequently asked
  questions about IBC-rs.

## How to Integrate IBC

* [**Integration**](./../../developers/integration/overview) - Learn how to
  integrate IBC module and relayer into your blockchain.

## Related Projects

Check out the docs of projects relate to the IBC-rs:

* [**CometBFT**](https://docs.cometbft.com) - The leading BFT engine for
  building blockchains, powering Cosmos SDK.
* [**Hermes**](https://hermes.informal.systems) - A Rust implementation of IBC
  relayer.
* [**Basecoin**](https://github.com/informalsystems/basecoin-rs) - A basic Rust
  implementation of the CometBFT ABCI application, primarily serving as our
  testing stand.
* [**IBC-Go**](https://ibc.cosmos.network/) - A Go implementation of the IBC
  protocol.

## Help & Support

* [**Migration Guide**](./../../developers/migrations/guideline) - Learn how to
  migrate to a newer stable version of IBC-rs.
* [**GitHub Discussions**](https://github.com/cosmos/ibc-rs/discussions) - Ask
  questions and discuss IBC-rs development on GitHub.
* [**Want to
  Contribute?**](./developers/intro/contributing) -
  Learn how to contribute to IBC-rs development.
* [**Found an Issue?**](https://github.com/cosmos/ibc-rs/edit/main/docs/docs/DOC_README.md) - Help us
  improve the documentation by suggesting edits on GitHub.
