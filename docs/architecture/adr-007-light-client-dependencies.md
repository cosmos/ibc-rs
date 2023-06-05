# ADR 007: LIGHT CLIENT DEPENDENCIES

## Context

This ADR is meant to address the main limitation of our current light client API, first introduced in [ADR 4] and [later improved] to adopt some of the ideas present in ibc-go's [ADR 6]. Implementing some `ClientState` methods require additional information from the host. For example, the Tendermint client's implementation of `ClientState::verify_client_message` needs [access to the host timestamp] to properly perform a message's verification. We solved this problem by [giving a reference] to a `ValidationContext` and `ExecutionContext`, since most methods are already made available by these traits. However, this solution has some limitations:

1. Not all methods needed by every future light client is present in `ValidationContext` or `ExecutionContext`. For example, if a light client X finds that it would need access to some resource X, currently the only way to solve this is to submit a PR on the ibc-rs repository that adds a method `get_resource_Y()` to `ValidationContext`.
    + This means that every host will need to implement `get_resource_Y()`, even if they don't use light client X.
    + It clutters up `ValidationContext` and `ExecutionContext`.
2. We found that some methods only needed by the Tendermint light client made their way into `ValidationContext`.
    + `next_consensus_state()` and `prev_consensus_state()` are not used in the core handlers; they're only there because of the Tendermint light client.
3. It gives more power to light clients than they really need
    + By giving the light clients access to `ValidationContext` and `ExecutionContext`, we're effectively giving them the same capabilities as the core handlers.
    + Although our current model is that all code is trusted (including light clients we didn't write), restraining the capabilities we give to light clients at the very least eliminates a class of bugs (e.g. calling the wrong method), and serves as documentation for exactly what the light client will need.

This ADR is all about fixing this issue; namely, to enable light clients to impose a `Context` trait for the host to implement.

[ADR 4]: ../architecture/adr-004-light-client-crates-extraction.md
[later improved]: https://github.com/cosmos/ibc-rs/pull/584
[ADR 6]: https://github.com/cosmos/ibc-go/blob/main/docs/architecture/adr-006-02-client-refactor.md
[access to the host timestamp]: https://github.com/cosmos/ibc-rs/blob/3e2566b3102af3fb6185cdc158cff818ec605535/crates/ibc/src/clients/ics07_tendermint/client_state/update_client.rs#L70
[giving a reference]: https://github.com/cosmos/ibc-rs/blob/3e2566b3102af3fb6185cdc158cff818ec605535/crates/ibc/src/core/ics02_client/client_state.rs#L72

## Decision

The primary change is that we will no longer use dynamic dispatch. Namely, we will remove all occurances of `dyn ValidationContext`, `Box<dyn ConsensusState>`, etc. This is because our solution will be centered around generics, and our traits will no longer be trait object safe.

The `ClientState` trait is split into 4 traits: `ClientStateBase`, `ClientStateInitializer<SupportedConsensusStates>`

+ What `SupportedConsensusStates` is

### Light client implementation

In this section, we will discuss the general pattern that light clients will use.


## Consequences

> This section describes the consequences, after applying the decision. All consequences should be summarized here, not just the "positive" ones.

### Positive

### Negative
+ If 2 light clients need the same (or very similar) methods, then the host will need to reimplement the same method multiple times
    + Although mitigatable by implementing once in a function and delegating all trait methods to that implementation, it is at the very least additional boilerplate


### Neutral

## References

> Are there any relevant PR comments, issues that led up to this, or articles referenced for why we made the given design choice? If so link them here!

* [Main issue](https://github.com/cosmos/ibc-rs/issues/296)
