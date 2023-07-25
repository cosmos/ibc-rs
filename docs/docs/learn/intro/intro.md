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
ICS-26. However, few
[divergences](https://github.com/cosmos/ibc-rs/tree/main/crates/ibc#divergence-from-the-interchain-standards-ics)
from the interchain standards (ICS) are introduced to accommodate the
implementation.

* **IBC Light Clients**
  * [**ICS-20: Tendermint**](./../clients/tendermint.md)
* **IBC Applications**
  * [**ICS-20: Fungible Token Transfer**](./../apps/token-transfer.md)
  * [**ICS-27: Interchain Accounts**](./../apps/interchain-accounts.md) (Work in progress)

## Latest Status

Check out our [project board](https://github.com/orgs/cosmos/projects/27) for
  the latest status of the IBC-rs implementation.

## Explore Implementation

To get a better understanding of IBC-rs implementation, check out the following sections:

* [**Overview**](./../learn/overview/overview) - Provides and overview of IBC-rs
  and learn about the basics of implementation.
* [**ADRs**](./../../developers/architecture/README.md) - Learn about the
  architectural decisions.
* [**FAQ Page**](https://github.com/cosmos/ibc-rs/wiki/FAQ) - Frequently asked
  questions about IBC-rs.

## How to Integrate

* [**Integration**](./../../developers/integration/integration.md) - Learn how to
  integrate IBC-rs into your project.

## Explore the Stack

Check out the docs of projects relate to the IBC-rs stack:

* [**CometBFT**](https://docs.cometbft.com) - The leading BFT engine for
  building blockchains, powering Cosmos SDK.
* [**Hermes**](https://hermes.informal.systems) - A Rust implementation of IBC
  relayer.
* [**Basecoin**](https://github.com/informalsystems/basecoin-rs) - A basic Rust
  implementation of the CometBFT ABCI application, primarily serving as our
  testing platform.
* [**IBC-Go**](https://ibc.cosmos.network/) - A Go implementation of the IBC
  protocol.

## Security Notice

There are no established security practices specifically for IBC-rs. Please
direct your security reports to the [IBC-go
repository](https://github.com/cosmos/ibc-go/security/policy).

**Note**: The [disclosure
  log](https://github.com/informalsystems/hermes/blob/master/docs/disclosure-log.md) documents
  problems we have uncovered while specifying and implementing IBC-rs. Some of
  the cases recorded there might have been fixed.

## Help & Support

* [**Migration Guide**](./developers/migrations/guideline) - Learn how to
  migrate to a newer stable version of IBC-rs.
* [**GitHub Discussions**](https://github.com/cosmos/ibc-rs/discussions) - Ask
  questions and discuss IBC-rs development on GitHub.
* [**Want to
  Contribute?**](https://github.com/cosmos/ibc-rs/blob/main/CONTRIBUTING.md) -
  Learn how to contribute to IBC-rs development.
* [**Found an Issue?**](https://github.com/cosmos/ibc-rs/edit/main/docs/docs/DOC_README.md) - Help us
  improve the documentation by suggesting edits on GitHub.
