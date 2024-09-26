This release brings major improvements to error handling in `ibc-rs`, giving
hosting environments better control over errors and make the process of
debugging easier for the developers. A key enhancement is the clearer
distinction between host-sourced errors and those propagated by `ibc-rs`,
effectively separating host-level from protocol-level errors. Therefore, a
noticeable update is the renaming of the previous `ContextError` to
`HandlerError`, which now exclusively manages errors from IBC handlers. In
parallel, a new `HostError` has been introduced to handle errors originating
from hosts, particularly those from validation and execution contexts.
Additionally, error definitions within `ibc-rs` have been unified, reducing the
granularity of error variants. For more details, please refer to
[ADR-011](./docs/architecture/adr-11-refactor-errors.md).

In addition, it introduces various fixes and enhancements. Notably, helper
traits with default implementations have been added to simplify the conversion
between host time types and `Timestamp`. Consequently, the `ibc-primitives`
crate has been fully decoupled from the `tendermint` dependency.

It’s also worth noting that the `cosmwasm` workspace has been relocated to its
own repository, now available under
[cosmwasm-ibc](https://github.com/informalsystems/cosmwasm-ibc).

There are no consensus-breaking changes in this release.
