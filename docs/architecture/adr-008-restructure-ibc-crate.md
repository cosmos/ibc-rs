# ADR 008: RESTRUCTURE `ibc` CRATE

## Context

The current state of the **`ibc`** crate exhibits a mix of different
implementation layers. From a module-based perspective, it encompasses essential
elements like ibc core, clients, applications, along with testing facilities.
However, an architectural view reveals a fusion of diverse layers of type
definitions, interfaces (APIs), and handler functions, resulting in a mix of
dependencies and features that may lead to potential conflicts or unnecessary
imports for users.

As of [this pull request](https://github.com/cosmos/ibc-rs/pull/954), we've
separated our mock testing kit into the standalone **`ibc-testkit`** library.
This decoupling from the main **`ibc`** crate sets the stage for the objectives
of this ADR.

The primary goals here are twofold: firstly, to reduce interdependence within
the codebase among various components such as ibc core, clients, and
applications, and secondly, to improve the overall usability of the `ibc-rs`
implementation. The overarching aim is to empower users to selectively import
specific IBC layers, mitigating potential conflicts related to dependencies or
features that may arise in the context of a monolithic library and letting
`ibc-rs` be used in the following scenarios:

1. **Selective Module Import**
    - Users cannot import only the necessary components/modules for their
      projects. For instance, importing only the **`ics07_tendermint`**
      implementation is impractical.
2. **Selective Types Import**
    - Relayers, like Hermes, or any off-chain consumers cannot import their
      desired layer of implementation like ibc types without pulling in
      unnecessary dependencies into their project.
3. **Smoother IBC Core Integration with Hosts**
    - Integrating ibc core with host chains without introducing light client or
      app dependencies is currently not straightforward, impeding smooth
      integration.
4. **Easier Development of CosmWasm Contracts**
    - For developing a CosmWasm tendermint light client, we ideally should only
      be dependent on implementation under the **`ics07_tendermint`** and also
      be importing relevant parts from the **`ibc-core-client`** layer without
      pulling in all the ibc codebase and dependencies.

This ADR aims to enhance both the usability and practicality of `ibc-rs` by
restructuring the codebase and organizing it under multiple sub-libraries, as
stated in the [decision](#decision) section. This will make different parts of
`ibc-rs` accessible to users, positioning it as a more comprehensive, one-stop
solution catering to diverse user groups, whether for on-chain or off-chain use
cases.

## Decision

For the library organization, the first stage of separation is to split the
codebase so that each IBC application, client, and core implementation is
decoupled from one another. The top-level libraries and the naming schema would
look as follows:

```markdown
.
├── ibc -> Primarily re-exports sub-libraries
├── ibc-core 
│   ├── ibc-core-client (contains the implementation + Re-exports types)
│   ├── ibc-core-connection
│   ├── ibc-core-channel
│   ├── ibc-core-commitment
│   └── ibc-core-host
│       └── .
├── ibc-clients
│   ├── ibc-client-tendermint
│   ├── ibc-client-tendermint-cw
│   └── .
├── ibc-apps
│   ├── ibc-app-transfer
│   ├── ibc-app-ica
│   │   └── .
│   └── .
├── ibc-primitives
├── ibc-testkit (previously mock module + `test-utils` feature)
├── ibc-query
└── ibc-derive
```

With this restructure, the main `ibc` crate primarily re-exports types,
interfaces, and implementations of all the sub-libraries. Therefore, if someone
only wants to depend on the `ibc` crate without caring about this granularity,
they can do so.

Afterward, we split off data structure (domain types) of each IBC layer into a
separate sub-library under a `types` folder, still maintained under the
directory of that relevant component/module. As an example, the
`ibc-core-client` crate’s tree and the naming schema would look like this:

```markdown
ibc-core
└── ibc-core-client (dir: ibc-core/ics02-client)
    └── ibc-core-client-types (dir: ibc-core/ics02-client/types)
        ├── msgs
        ├── events
        └── .
```

This way, the main crate of each IBC module contains all the necessary APIs and
implementations to integrate with host chains, along with re-exporting the
sub-library types. This allows projects to selectively import types (e.g.
`ibc-core-client-types`), often required by off-chain users such as relayers. Or
to pick the library containing the entire implementation of that particular
module (e.g. `ibc-core-client`), typically more convenient for host chains or
smart contract developers to integrate with on their end.

Once the restructuring is complete, the **directory tree** of the repo would
look as follows:

```markdown
ibc
ibc-core
├── ics02-client
|   ├── src
|   ├── types
|   |   ├── src
|   |   └── Cargo.toml
|   └── Cargo.toml
├── ics03-connection
|   └── .
├── ics04-channel
|   └── .
├── ics23-commitment
|   └── .
├── ics24-host
|   └── .
├── src
├── Cargo.toml
└── README.md
ibc-clients
├── ics07-tendermint
├── ics08-wasm
└── .
ibc-apps
├── ics20-transfer
├── ics27-ica
└── .
ibc-primitives
ibc-testkit
ibc-query
ibc-derive
```

In the refactored codebase, there will be several `*-types` crates that rely on
the `ibc-proto` library. This library acts as an upstream crate, facilitating
the conversion to and from proto types. Each `*-types` crate also re-exports
crucial proto types for added user convenience. This approach ensures a seamless
experience for users downstream, sparing them the necessity of directly
including the `ibc-proto` dependency in their projects. Consequently, the
`*-types` crates serve as comprehensive, battery-included libraries.

To implement this ADR efficiently and for more organization, we use the
workspace inheritance feature and will add a top-level README for main library
groups like `ibc-core`, `ibc-clients`, etc serving as a guide for users to
understand the purpose and structure of their sub libraries.

Later, it is crucial to come up with a Github action to automate and simplify
the release process as well.

## **Status**

Accepted

## **Consequences**

We should acknowledge this restructuring, while a significant step forward, will
not completely address all existing design couplings. Subsequent improvements in
implementation logic will be necessary to completely decouple ibc core, clients,
and applications from each other and make the entire logic as chain-agnostic as
possible. For instance, currently, our `IbcEvent` type depends on the Tendermint
events in their conversion, which can only be addressed once this restructuring
is complete. There may be other mix-ups as well, but the new repository
structure significantly simplifies their handling and ensures `ibc-rs` evolves
into a more adaptable, modular, and composable implementation that can serve
various use cases.

### **Positive**

- Opens up a range of new use cases for `ibc-rs`
- Facilitates moving toward more chain-agnostic and flexible design and interfaces
- Simplifies development on top of each layer of `ibc-rs` implementation

### **Negative**

- Multiple libraries are more challenging to maintain
- Enforces current users to update a large number of their import paths from
  `ibc` crates
