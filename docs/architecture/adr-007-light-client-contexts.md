# ADR 007: LIGHT CLIENT CONTEXTS

## Context

This ADR is meant to address the main limitation of our current light client API, first introduced in [ADR 4] and [later improved] to adopt some of the ideas present in ibc-go's [ADR 6]. Implementing some `ClientState` methods require additional information from the host. For example, the Tendermint client's implementation of `ClientState::verify_client_message` needs [access to the host timestamp] to properly perform a message's verification. Previously, we solved this problem by [giving a reference] to a `ValidationContext` and `ExecutionContext`, since most methods are already made available by these traits. However, this solution has some limitations:

1. Not all methods needed by every future light client is present in `ValidationContext` or `ExecutionContext`. For example, if a light client X finds that it would need access to some resource Y, currently the only way to solve this is to submit a PR on the ibc-rs repository that adds a method `get_resource_Y()` to `ValidationContext`.
    + This means that every host will need to implement `get_resource_Y()`, even if they don't use light client X.
    + It clutters up `ValidationContext` and `ExecutionContext`.
2. We found that some methods only needed by the Tendermint light client made their way into `ValidationContext`.
    + `next_consensus_state()` and `prev_consensus_state()` are not used in the core handlers; they're only there because of the Tendermint light client.
3. It gives more power to light clients than they really need
    + By giving the light clients access to `ValidationContext` and `ExecutionContext`, we're effectively giving them the same capabilities as the core handlers.
    + Although our current model is that all code is trusted (including light clients we didn't write), restraining the capabilities we give to light clients at the very least eliminates a class of bugs (e.g. calling the wrong method), and serves as documentation for exactly which methods the light client needs.

This ADR is all about fixing this issue; namely, to enable light clients to define their own `ValidationContext` and `ExecutionContext` traits for the host to implement.

[ADR 4]: ../architecture/adr-004-light-client-crates-extraction.md
[later improved]: https://github.com/cosmos/ibc-rs/pull/584
[ADR 6]: https://github.com/cosmos/ibc-go/blob/main/docs/architecture/adr-006-02-client-refactor.md
[access to the host timestamp]: https://github.com/cosmos/ibc-rs/blob/3e2566b3102af3fb6185cdc158cff818ec605535/crates/ibc/src/clients/ics07_tendermint/client_state/update_client.rs#L70
[giving a reference]: https://github.com/cosmos/ibc-rs/blob/3e2566b3102af3fb6185cdc158cff818ec605535/crates/ibc/src/core/ics02_client/client_state.rs#L72

## Decision


### Changes to `ClientState`

The `ClientState` functionality is split into 3 traits:
+ `ClientStateCommon`,
+ `ClientStateValidation<ClientValidationContext>`, and
+ `ClientStateExecution<ClientExecutionContext>`

Then, `ClientState` is defined as

```rust
pub trait ClientState<ClientValidationContext, E: ClientExecutionContext>:
    ClientStateCommon
    + ClientStateValidation<ClientValidationContext>
    + ClientStateExecution<E>
    // + ...
{
}
```

A blanket implementation implements `ClientState` when these 3 traits are implemented on a given type. For details as to why `ClientState` was split into 3 traits, see the section "Why are there 3 `ClientState` traits?".

The `ClientStateValidation` and `ClientStateExecution` traits are the most important ones, as they are the ones that enable light clients to define `Context` traits for the host to implement.

#### `ClientStateValidation`

Say the implementation of a light client needs a `get_resource_Y()` method from the host in `ClientState::verify_client_message()`. The implementor would first define a trait for the host to implement.

```rust
trait MyClientValidationContext {
    fn get_resource_Y(&self) -> Y;
}
```

Then, they would implement the `ClientStateValidation<ClientValidationContext>` trait *conditioned on* `ClientValidationContext` having `MyClientValidationContext` as supertrait.

```rust
impl<ClientValidationContext> ClientStateValidation<ClientValidationContext> for MyClientState
where
    ClientValidationContext: MyClientValidationContext,
{
    fn verify_client_message(
        &self,
        ctx: &ClientValidationContext,
        // ...
    ) -> Result<(), ClientError> {
        // `get_resource_Y()` accessible through `ctx`
    }

    // ...
}
```

This is the core idea of this ADR. Everything else is a consequence of wanting to make this work.

#### `ClientStateExecution`

`ClientStateExecution` is defined a little differently from `ClientStateValidation`.

```rust
pub trait ClientStateExecution<E>
where
    E: ClientExecutionContext,
{ ... }
```

where `ClientExecutionContext` is defined as (simplified)

```rust
pub trait ClientExecutionContext: Sized {
    // ... a few associated types

    /// Called upon successful client creation and update
    fn store_client_state(
        ...
    ) -> Result<(), ContextError>;

    /// Called upon successful client creation and update
    fn store_consensus_state(
        ...
    ) -> Result<(), ContextError>;
}
```

Under our current architecture (inspired from ibc-go's [ADR 6]), clients have the responsibility to store the `ClientState` and `ConsensusState`. Hence, `ClientExecutionContext` defines a uniform interface that clients can use to store their `ClientState` and `ConsensusState`. It also means that the host only needs to implement these methods once, as opposed to once per client. Note that clients who don't store consensus states (e.g. solomachine) can simply leave the implementation of `store_consensus_state()` empty (or return an error, whichever is most appropriate).

### Changes to `ValidationContext` and `ExecutionContext`

The `ClientState` changes described above induce some changes on `ValidationContext` and `ExecutionContext`.

`ValidationContext` is now defined as:

```rust
pub trait ValidationContext: Router {
    type ClientValidationContext;
    type ClientExecutionContext;
    /// Enum that can contain a `ConsensusState` object of any supported light client
    type AnyConsensusState: ConsensusState<EncodeError = ContextError>;
    /// Enum that can contain a `ClientState` object of any supported light client
    type AnyClientState: ClientState<
        Self::AnyConsensusState,
        Self::ClientValidationContext,
        Self::ClientExecutionContext,
    >;

    // ...
}
```

`AnyConsensusState` and `AnyClientState` are expected to be enums that hold the consensus states and client states of all supported light clients. For example,

```rust
enum AnyConsensusState {
    Tendermint(TmConsensusState),
    Near(NearConsensusState),
    // ...
}

enum AnyClientState {
    Tendermint(TmClientState),
    Near(NearClientState),
    // ...
}
```

`ClientValidationContext` and `ClientExecutionContext` correspond to the same types described in the previous section. The host must ensure that these 2 types implement the Tendermint and Near "`ValidationContext` and `ExecutionContext` traits" (as discussed in the previous section). For example,

```rust
struct MyClientValidationContext;

// Here, `TmClientValidationContext` is a Tendermint's `ValidationContext`, meaning that it contains all the methods
// that the Tendermint client requires from the host in order to perform message validation.
impl TmClientValidationContext for MyClientValidationContext {
    // ...
}

impl NearClientValidationContext for MyClientValidationContext {
    // ...
}

// Code for `ClientExecutionContext` is analogous
```

### `ClientState` and `ConsensusState` convenience derive macros
Notice that `ValidationContext::AnyClientState` needs to implement `ClientState`, and `ValidationContext::AnyConsensusState` needs to implement `ConsensusState`. Given that `AnyClientState` and `AnyConsensusState` are enums that wrap types that *must* implement `ClientState` or `ConsensusState` (respectively), implementing these traits is gruesome boilerplate:

```rust
impl ClientStateCommon for AnyClientState {
    fn client_type(&self) -> ClientType {
        match self {
            Tendermint(cs) => cs.client_type(),
            Near(cs) => cs.client_type()
        }
    }

    // ...
}
```

To relieve users of such torture, we provide derive macros that do just that:

```rust
#[derive(ConsensusState)]
enum AnyConsensusState {
    Tendermint(TmConsensusState),
    Near(NearConsensusState),
    // ...
}

#[derive(ClientState)]
#[validation(MyClientValidationContext)]
#[execution(MyClientExecutionContext)]
enum AnyClientState {
    Tendermint(TmClientState),
    Near(NearClientState),
    // ...
}
```

## FAQs

### Why are there 3 `ClientState` traits?

The `ClientState` trait is defined as

```rust
trait ClientState<ClientValidationContext, ClientExecutionContext>
```

The problem with defining all methods directly under `ClientState` is that it would force users to use fully qualified notation to call any method.

This arises from the fact that no method uses both generic parameters. [This playground] provides an explanatory example. Hence, our solution is to have all methods in a trait use every generic parameter of the trait to avoid this problem.

[This playground]: https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=da65c22f1532cecc9f92a2b7cb2d1360

### Why did you write custom `ClientState` and `ConsensusState` derive macros? Why not use `enum_dispatch` or `enum_delegate`?
We ended up having to write our own custom derive macros because existing crates that offer similar functionality had shortcomings that prevented us from using them:

+ `enum_dispatch`: the trait `ClientState` and the enum that implements `ClientState` need to be defined in the same crate
+ `enum_delegate` (v0.2.*): was designed to remove the above restriction. However, generic traits are not supported.
    + we investigated [turning the generic types] of `ClientState` into associated types. However we were hit by the other limitation of `enum_delegate`: `ClientState` cannot have any supertrait.

[turning the generic types]: https://github.com/cosmos/ibc-rs/issues/296#issuecomment-1540630517
## Consequences

### Positive
+ All light clients can now be implemented in their crates without ever needing to modify ibc-rs
+ Removes trait object downcasting in light client implementations
    + downcasting fails at runtime; these errors are now compile-time

### Negative
+ Increased complexity.
+ Harder to document.
    + Specifically, we do not write any trait bounds on the `Client{Validation, Execution}Context` generic parameters. The effective trait bounds are spread across all light client implementations that a given host uses.


### Neutral
+ Our light client traits are no longer trait-object safe. Hence, for example, all uses of `Box<dyn ConsensusState>` are replaced by the analogous `ValidationContext::AnyConsensusState`.

## Future work

In the methods `ClientState::{verify_client_message, check_for_misbehaviour, update_state, update_state_on_misbehaviour}`, the `client_message` argument is still of type `ibc_proto::google::protobuf::Any` (i.e. still serialized). Ideally, we would have it be well-typed and unserialized. Since there are many ways to do this, and this was slightly tangential to this work, we left it as future work.

## References

* [Main issue](https://github.com/cosmos/ibc-rs/issues/296)
