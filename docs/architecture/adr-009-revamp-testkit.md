# ADR 009: Revamp IBC integration test framework

## Changelog

- 04-03-2024: Initial draft

## Context

The current framework in the IBC testkit uses
[existing types and injects state or dependency
manually](https://github.com/cosmos/ibc-rs/blob/v0.51.0/ibc-testkit/tests/core/ics02_client/update_client.rs#L574-L578).
Sometimes, it uses
[semantically wrong data as mock data](https://github.com/cosmos/ibc-rs/blob/v0.51.0/ibc-testkit/src/testapp/ibc/core/types.rs#L320).
Because of this, tests with customizable steps and fixed fixtures became ad-hoc
and unmaintainable.

To overcome this, we need to improve our test framework that allows:

- testing different implementations (traits).
- succinct tests (useful `util` methods).
- improving test coverage (i.e. validating Merkle proof generation).
- integration tests exercising the IBC workflow (relayer-like interface)

## Decision

The main goal of this proposal is to create a test framework that is modular and
closer to the real blockchain environment. This should also make the existing
tests succinct and readable. Instead of bootstrapping the mock data, we should
use valid steps to generate it - so that we know the exact steps to reach a
state to reproduce in a real environment.

To achieve this, we have broken down the proposal into sub-proposals:

### 1. Adopt a Merkle store for the test framework

The current framework uses `HashMap` and `HashSet` to store data. This works for
many test scenarios, but it fails to test proof-sensitive scenarios. Because of
this, we don't have any connection, channel handshake, or packet relay tests
that cover the Tendermint light client.

We generalize
[`MockContext`](https://github.com/cosmos/ibc-rs/blob/v0.51.0/ibc-testkit/src/testapp/ibc/core/types.rs#L103)
to use a Merkle store which is used for IBC Context's store. For concrete or
default implementations, we can use the IAVL Merkle store implementation from
`informalsystems/basecoin-rs`.

### 2. Modularize the host environment

Currently, we are using `Mock` and `SyntheticTendermint` variants of
[`HostType`](https://github.com/cosmos/ibc-rs/blob/v0.51.0/ibc-testkit/src/hosts/block.rs#L33-L36)
enum as host environments. To manage these two different environments, we also
introduced
[`HostBlocks`](https://github.com/cosmos/ibc-rs/blob/v0.51.0/ibc-testkit/src/hosts/block.rs#L72-75)
for encapsulating the possible host variants that need to be covered by
`ibc-testkit`.

However, this creates friction in the case when we need to add new host
variants. It creates the same problem that the `ibc-derive` crate was designed
to solve for `ClientState` and `ConsensusState` types, namely: dispatching
methods to underlying variants of a top-level enum. But, a concrete
`MockContext` will always have a unique `HostType` and its corresponding
`HostBlocks`. So we can refactor `HostTypes` and `HockBlocks` with a generic
`TestHost` trait that maintains its corresponding types e.g. `Block` types, via
associated types. Finally, we generalize the `MockContext` once more to use this
`TestHost` trait instead of a concrete enum variant.

This `TestHost` trait should be responsible for generating blocks, headers,
client, and consensus states specific to that host environment.

```rs
/// TestHost is a trait that defines the interface for a host blockchain.
pub trait TestHost: Default + Debug + Sized {
    /// The type of block produced by the host.
    type Block: TestBlock;

    /// The type of client state produced by the host.
    type ClientState: Into<AnyClientState> + Debug;

    /// The type of block parameters to produce a block.
    type BlockParams: Debug + Default;

    /// The type of light client parameters to produce a light client state.
    type LightClientParams: Debug + Default;

    /// The history of blocks produced by the host chain.
    fn history(&self) -> &VecDeque<Self::Block>;

    /// Triggers the advancing of the host chain by extending the history of blocks (or headers).
    fn advance_block(
        &mut self,
        commitment_root: Vec<u8>,
        block_time: Duration,
        params: &Self::BlockParams,
    );

    /// Generate a block at the given height and timestamp, using the provided parameters.
    fn generate_block(
        &self,
        commitment_root: Vec<u8>,
        height: u64,
        timestamp: Timestamp,
        params: &Self::BlockParams,
    ) -> Self::Block;

    /// Generate a client state using the block at the given height and the provided parameters.
    fn generate_client_state(
        &self,
        latest_height: &Height,
        params: &Self::LightClientParams,
    ) -> Self::ClientState;
}

/// TestBlock is a trait that defines the interface for a block produced by a host blockchain.
pub trait TestBlock: Clone + Debug {
    /// The type of header that can be extracted from the block.
    type Header: TestHeader;

    /// The height of the block.
    fn height(&self) -> Height;

    /// The timestamp of the block.
    fn timestamp(&self) -> Timestamp;

    /// Extract the header from the block.
    fn into_header(self) -> Self::Header;
}

/// TestHeader is a trait that defines the interface for a header
/// submitted by relayer from the host blockchain.
pub trait TestHeader: Clone + Debug + Into<Any> {
    /// The type of consensus state can be extracted from the header.
    type ConsensusState: ConsensusState + Into<AnyConsensusState> + From<Self> + Clone + Debug;

    /// The height of the block, as recorded in the header.
    fn height(&self) -> Height;

    /// The timestamp of the block, as recorded in the header.
    fn timestamp(&self) -> Timestamp;

    /// Extract the consensus state from the header.
    fn into_consensus_state(self) -> Self::ConsensusState;
}
```

### 3. Decoupling IbcContext and Host environment

Currently, `MockContext` implements the top-level
[validation](https://github.com/cosmos/ibc-rs/blob/v0.51.0/ibc-core/ics25-handler/src/entrypoint.rs#L45)
and
[execution](https://github.com/cosmos/ibc-rs/blob/v0.51.0/ibc-core/ics25-handler/src/entrypoint.rs#L112)
contexts of `ibc-rs`, as opposed to the more granular contexts of each of the
individual handlers. It contains other host-specific data e.g. `host_chain_id`,
`block_time` - that are not directly relevant to the IBC context. If we think of
`MockContext` as a real blockchain context, the `MockContext` represents the
top- level runtime; it contains `MockIbcStore`, which is a more appropriate
candidate to implement the validation and execution contexts than the
`MockContext` itself.

With this, the `MockContext` contains two decoupled parts - the host and the IBC
module.

### 4. Chain-like interface for `MockContext`

With the above changes, we can refactor the `MockContext` to have
blockchain-like interfaces.

The `MockContext` should have `end_block`, `produce_block`, and `begin_block` to
mimic real blockchain environments, such as the Cosmos-SDK.

```rs
impl<S, H> MockContext<S, H>
where
    S: ProvableStore + Debug,
    H: TestHost,
{
    pub fn ibc_store_mut(&mut self) -> &mut MockIbcStore<S>;
    pub fn host_mut(&mut self) -> &mut H;

    pub fn generate_genesis_block(&mut self, genesis_time: Timestamp);
    pub fn begin_block(&mut self);
    pub fn end_block(&mut self);
    pub fn produce_block(&mut self, block_time: Duration);
}
```

### 5. ICS23 compatible proof generation

With the new proof generation capabilities, we can now test the Tendermint light
clients. But we need our proofs to be ICS23 compatible. ICS23 expects the IBC
store root to be committed at a commitment prefix at a top-level store in the
host environment.

For this, we add an extra store in `MockContext` where the `MockIbcStore`
commits its storage root at its
[commitment prefix](https://github.com/cosmos/ibc-rs/blob/v0.51.0/ibc-testkit/src/testapp/ibc/core/core_ctx.rs#L127-L129)
key.

So the `MockContext` is finalized as:

```rs
pub struct MockGenericContext<S, H>
where
    S: ProvableStore + Debug,
    H: TestHost
{
    pub main_store: S,
    pub host: H,
    pub ibc_store: MockIbcStore<S>,
}
```

Now the `MockIbcStore` can generate proofs that contain the proofs in its store
and commitment prefix. But it has to know the proofs of its commitment prefix of
the previous heights.

So we add an extra store in `MockIbcStore` to store the proofs from previous
heights. This is similar to storing `HostConsensusState`s of previous heights.

```rs
#[derive(Debug)]
pub struct MockIbcStore<S>
where
    S: ProvableStore + Debug,
{
    ...
    /// Map of host consensus states.
    pub host_consensus_states: Arc<Mutex<BTreeMap<u64, AnyConsensusState>>>,
    /// Map of proofs of ibc commitment prefix.
    pub ibc_commiment_proofs: Arc<Mutex<BTreeMap<u64, CommitmentProof>>>,
}
```

The storing of the IBC store root at the IBC commitment prefix happens in the
end block. The storing of proofs and host consensus states happens in the
`begin_block` of the `MockContext`.

### 6. Integration Tests via `RelayerContext`

With all the above changes, we can now write an integration test that tests the
IBC workflow - client creation, connection handshake, channel handshake, and
packet relaying for any host environment that implements `TestHost`.

This can be done by reading the
[IBC events](https://github.com/cosmos/ibc-rs/blob/v0.51.0/ibc-testkit/src/testapp/ibc/core/types.rs#L95)
from `MockIbcStore` and creating and sending the IBC messages via
[`MockContext::deliver`](https://github.com/cosmos/ibc-rs/blob/v0.51.0/ibc-testkit/src/testapp/ibc/core/types.rs#L696).

### Miscellaneous

To achieve blockchain-like interfaces, we removed `max_history_size` and
`host_chain_id` from `MockContext`.

- `max_history_size`: We generate all the blocks till a block height. This gives
  us reproducibility. If we need to prune some older block data, we use a
  dedicated `prune_block_till` to prune older blocks. This makes our tests more
  descriptive about the assumption of the test scenarios.
- `host_chain_id`: The IBC runtime does not depend on `host_chain_id` directly.
  The `TestHost` trait implementation is responsible for generating the blocks
  with the necessary data.

Also to minimize verbosity while writing tests (as Rust doesn't support default
arguments to function parameters), we want to use some parameter builders. For
that, we can use the [`TypedBuilder`](https://crates.io/crates/typed-builder)
crate.

## Status

Proposed

## Consequences

This ADR pays the technical debt of the existing testing framework.

### Positive

Future tests will be more readable and maintainable. The test framework becomes
modular and leverages Rust's trait system. `ibc-rs` users may benefit from this
framework, which allows them to test their host implementations of `ibc-rs`
components.

### Negative

This requires a significant refactoring of the existing tests. Since this may
take some time, the parallel development on the `main` branch may conflict with
this work.

## References

This work is being tracked at
[cosmos/ibc-rs#1109](https://github.com/cosmos/ibc-rs/pull/1109).

The following provides the concrete implementations of the proposed changes:

#### MockIbcStore

The modified `MockIbcStore` with Merkle store lives at
[`testapp/ibc/core/types.rs`](https://github.com/cosmos/ibc-rs/blob/feat/refactor-testkit/ibc-testkit/src/testapp/ibc/core/types.rs#L43-L96).

#### TestHost

The Rust trait lives at
[`hosts/mod.rs`](https://github.com/cosmos/ibc-rs/blob/feat/refactor-testkit/ibc-testkit/src/hosts/mod.rs#L27).
The `Mock` and `Tendermint` host implementations live in
[`hosts/mock.rs`](https://github.com/cosmos/ibc-rs/blob/feat/refactor-testkit/ibc-testkit/src/hosts/mock.rs#L30)
and
[`hosts/tendermint.rs`](https://github.com/cosmos/ibc-rs/blob/feat/refactor-testkit/ibc-testkit/src/hosts/tendermint.rs#L42)
respectively.

#### MockGenericContext

[`MockGenericContext`](https://github.com/cosmos/ibc-rs/blob/feat/refactor-testkit/ibc-testkit/src/context.rs#L34-L52)
is actually what is described as `MockContext` in the ADR. For simplicity, we
defined `MockContext` to
[have a concrete store](https://github.com/cosmos/ibc-rs/blob/feat/refactor-testkit/ibc-testkit/src/context.rs#L54-L55)
implementation.

```rs
pub type MockStore = RevertibleStore<GrowingStore<InMemoryStore>>;
pub type MockContext<H> = MockGenericContext<MockStore, H>;
```
