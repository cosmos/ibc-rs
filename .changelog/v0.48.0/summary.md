In this release, we've undertaken a comprehensive overhaul of the **`ibc-rs`**
repository, resulting in a strategic reorganization of the codebase. This
restructuring dissects the implementation of each IBC specification,
categorizing and situating them within relevant libraries. The primary objective
is to elevate `ibc-rs` practicality and enhance user flexibility by providing a
more modular and composable experience.

Users now have the flexibility to choose from a spectrum of dependencies. They
can opt to utilize the entire suite of meta-crates, such as `ibc`, `ibc-core`,
`ibc-clients`, or `ibc-apps`. Alternatively, they can exercise fine-grained
control by selectively importing specific crates. This can involve bringing in
an entire implemented IBC sub-module, like the `ibc-core-client` crate, or
importing only the associated data structures of a module, such as the
`ibc-core-client-types` crate.

Furthermore, this release introduces optimizations centered around construction
and validation of ICS-24 host identifiers, aiming to curtail some heap
allocations, beneficial for resource-constrained hosts.

There are no consensus-breaking changes.
