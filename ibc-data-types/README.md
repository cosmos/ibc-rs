# IBC Types

This crate serves as a central hub for re-exporting the implemented
Inter-Blockchain Communication (IBC) data structures. It simplifies the
integration of various IBC domain types into your project. IBC is a distributed
protocol facilitating communication between independent sovereign blockchains
and The IBC data structures within this crate abstract various IBC
specifications, offering a convenient means to encode and decode IBC messages to
and from proto types exposed by
[`ibc-proto`](https://github.com/cosmos/ibc-proto-rs) crate. Additionally, it
supports parsing events to and from ABCI event types.

## Sub-Crates

This crate organizes data structures into three main modules: `core`, `clients`,
and `apps`. Each category further re-exports its respective sub data structures,
providing a clear and modular path for easy navigation and usage:

### Core

| <div style="width:300px">Specification</div> | Crate |
| -------------------------------------------- | ------ |
| ICS-02: Client Semantics                     | [ibc-core-client-types](./../ibc-core/ics02-client/types) |
| ICS-03: Connection Semantics                 | [ibc-core-connection-types](./../ibc-core/ics03-connection/types) |
| ICS-04: Channel and Packet Semantics         | [ibc-core-channel-types](./../ibc-core/ics04-channel/types) |
| ICS-24: Host Requirements                    | [ibc-core-host-types](./../ibc-core/ics24-host/types) |
| ICS-25: Handler Interface                    | [ibc-core-handler-types](./../ibc-core/ics25-handler/types) |
| ICS-26: Routing Module                       | [ibc-core-routing-types](./../ibc-core/ics26-routing/types) |

### Clients

| <div style="width:300px">Specification</div> | Crate |
| -------------------------------------------- | ------ |
| ICS-07: Tendermint Client                    | [ibc-client-tendermint-types](./../ibc-clients/ics07-tendermint/types) |

### Apps

| <div style="width:300px">Specification</div> | Crate |
| -------------------------------------------- | ------ |
| ICS-20: Fungible Token Transfer              | [ibc-app-transfer-types](./../ibc-apps/ics20-transfer/types) |

## Contributing

IBC is specified in English in the [cosmos/ibc
repo](https://github.com/cosmos/ibc). Any protocol changes or clarifications
should be contributed there.

If you're interested in contributing, please take a look at the
[CONTRIBUTING](./../CONTRIBUTING.md) guidelines. We welcome and appreciate
community contributions!
