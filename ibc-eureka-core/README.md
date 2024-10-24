# IBC Core

This is the top-level library that re-exports the Inter-Blockchain Communication
(IBC) core modules as a meta-crate. It serves to simplify the process of
importing and integrating various IBC core modules into your blockchain.

IBC is a distributed protocol that enables communication between distinct
sovereign blockchains and IBC core is the part of the protocol that handles the
transport, authentication, and ordering (TAO) of data packets.

The structure within the `ibc-eureka-core` crate is designed to provide flexibility for
external users. You can choose to utilize the entire `ibc-eureka-core` crate, or
selectively import specific libraries. From there, you also have the flexibility
of bringing in an entire sub-module (e.g. the `ibc-eureka-core-client` crate), or only
a module's associated data structures (e.g. `ibc-eureka-core-client-types`).

This versatility empowers hosts, including chain integrators, relayers, or any
IBC tooling projects, to build their solution on top of the layers that best
suit their particular requirements.

## Sub-Crates

Currently, the `ibc-eureka-core` crate contains the implementation of the following IBC
core specifications:

### ICS-02: Client Semantics

- [ibc-eureka-core-client](./../ibc-eureka-core/ics02-client)
- [ibc-eureka-core-client-context](./../ibc-eureka-core/ics02-client/context)
- [ibc-eureka-core-client-types](./../ibc-eureka-core/ics02-client/types)

### ICS-03: Connection Semantics

- [ibc-eureka-core-connection](./../ibc-eureka-core/ics03-connection)
- [ibc-eureka-core-connection-types](./../ibc-eureka-core/ics03-connection/types)

### ICS-04: Channel and Packet Semantics

- [ibc-eureka-core-channel](./../ibc-eureka-core/ics04-channel)
- [ibc-eureka-core-channel-types](./../ibc-eureka-core/ics04-channel/types)

### ICS-24: Host Requirements

- [ibc-eureka-core-host](./../ibc-eureka-core/ics24-host)
- [ibc-eureka-core-host-cosmos](./../ibc-eureka-core/ics24-host/cosmos)
- [ibc-eureka-core-host-types](./../ibc-eureka-core/ics24-host/types)

### ICS-25: Handler Interface

- [ibc-eureka-core-handler](./../ibc-eureka-core/ics25-handler)
- [ibc-eureka-core-handler-types](./../ibc-eureka-core/ics25-handler/types)

### ICS-26: Routing Module

- [ibc-eureka-core-routing](./../ibc-eureka-core/ics26-routing)
- [ibc-eureka-core-routing-types](./../ibc-eureka-core/ics26-routing/types)

## Divergence from the Interchain Standards (ICS)

This crate diverges from the [ICS specification](https://github.com/cosmos/ibc)
in a number of ways. See below for more details.

### Module system: no support for untrusted modules

ICS-24 (Host Requirements) gives the [following
requirement](https://github.com/cosmos/ibc/blob/master/spec/core/ics-024-host-requirements/README.md#module-system)
about the module system that the host state machine must support:

> The host state machine must support a module system, whereby self-contained,
> potentially mutually distrusted packages of code can safely execute on the
> same ledger [...].

**This crate currently does not support mutually distrusted packages**. That is,
modules on the host state machine are assumed to be fully trusted. In practice,
this means that every module has either been written by the host state machine
developers, or fully vetted by them.

### Port system: No object capability system

ICS-05 (Port Allocation) requires the host system to support either
object-capability reference or source authentication for modules.

> In the former object-capability case, the IBC handler must have the ability to
> generate object-capabilities, unique, opaque references which can be passed to
> a module and will not be duplicable by other modules. [...] In the latter
> source authentication case, the IBC handler must have the ability to securely
> read the source identifier of the calling module, a unique string for each
> module in the host state machine, which cannot be altered by the module or
> faked by another module.

**This crate currently requires neither of the host system**. Since modules are
assumed to be trusted, there is no need for this object capability system that
protects resources for potentially malicious modules.

For more background on this, see [this issue](https://github.com/informalsystems/ibc-rs/issues/2159).

### Port system: transferring and releasing a port

ICS-05 (Port Allocation) requires the IBC handler to permit [transferring
ownership of a
port](https://github.com/cosmos/ibc/tree/master/spec/core/ics-005-port-allocation#transferring-ownership-of-a-port)
and [releasing a
port](https://github.com/cosmos/ibc/tree/master/spec/core/ics-005-port-allocation#releasing-a-port).

We currently support neither.

### Asynchronous acknowledgements

The standard gives the ability for modules to [acknowledge packets
asynchronously](https://github.com/cosmos/ibc/tree/main/spec/core/ics-004-channel-and-packet-semantics#writing-acknowledgements).
This allows modules to receive the packet, but only applying the changes at a
later time (after which they would write the acknowledgement).

We currently force applications to process the packets as part of
`onRecvPacket()`. If you need asynchronous acknowledgements for your
application, please open an issue.

Note that this still makes us 100% compatible with `ibc-go`.

## Contributing

IBC is specified in English in the [cosmos/ibc
repo](https://github.com/cosmos/ibc). Any protocol changes or clarifications
should be contributed there.

If you're interested in contributing, please take a look at the
[CONTRIBUTING](./../CONTRIBUTING.md) guidelines. We welcome and appreciate
community contributions!
