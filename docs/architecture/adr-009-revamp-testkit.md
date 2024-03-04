# ADR 009: Revamp IBC integration test framework

## Changelog

- 04-03-2024: Initial draft

## Context

The current framework in the IBC testkit uses
[existing types and injects state or dependency manually][1]. Sometimes, it uses
[semantically wrong data as mock data][2]. Because of this, tests with
customizable steps and fixed fixtures became messy and unmaintainable.

To overcome this, we need to create a proper integration test framework that
allows:

- writing and maintaining tests easily.
- yet covering various success and failure scenarios.

[1]: https://github.com/cosmos/ibc-rs/blob/65d84464842b3620f0bd66a07af30705bfa37761/ibc-testkit/tests/core/ics02_client/update_client.rs#L572-L576
[2]: https://github.com/cosmos/ibc-rs/blob/65d84464842b3620f0bd66a07af30705bfa37761/ibc-testkit/src/testapp/ibc/core/types.rs#L320

## Decision

### Principle

The main goal of this proposal is that writing and maintaining tests should be
easier. The happy tests should be succinct and readable. The happy steps may be
reused as fixtures - but not the sad steps. Because the sad tests should be
descriptive and contain comments about the test steps. (Also, if there are more
than 10 steps, it should be broken down into smaller tests or reuse fixtures)

### Testing traits

Moreover, the test framework should allow testing the implementations of the
traits. Namely, there are two sets of crucial traits - one for different kinds
of clients and the other for different IBC apps.

Testing different implementations of clients is possible using a general `Host`
and implementing its corresponding client traits. This is already done by
[`Mock` and `SyntheticTendermint`][3].

[3]: https://github.com/cosmos/ibc-rs/blob/65d84464842b3620f0bd66a07af30705bfa37761/ibc-testkit/src/hosts/block.rs#L32-L36

In the Tendermint client, we still don't test membership checks as we are not
using a Merkle tree in our test framework. In the Mock client, we return success
for these checks. This allowed us to write tests for Connection, Channel and
Packets but clearly, they don't cover the failure cases.

For the IBC apps, we are using existing `ics-20` and `ics-721`. But we may add a
mock one to cover more scenarios.

### Architecture

To revamp the test framework, we need to unify the existing methods, traits and
structs using a clear structure. The following is the proposed structure:

```rs
struct Chain<H: Host, R: Router, S: Store> {
    // host meta data
    host: H,

    // host chain history
    blocks: Vec<H::Block>,

    // chain storage with commitment vector support
    store: S,

    // deployed ibc module or contract
    ibc: Ibc<R, S>
}

impl<H, S> Chain<H: Host, S: Store> {
    fn new(chain_id: H::ChainId) -> Self;

    fn dispatch(msg: MsgEnvelope) -> Result<(), Error> {
        dispatch(self.ibc.context, self.ibc.router, msg)
    }
    ...

    fn advance_height(&self) {
        self.store.commit();
        let root_hash = self.store.root_hash();
        self.blocks.push(self.host.generate_block(root_hash));
    }
}

struct Relayer;

impl Relayer {
    fn bootstrap_client<LH, LS, RH, RS>(ctx_a: Chain<LH, LS>, ctx_b: Chain<RH, RS>);

    fn bootstrap_connection<LH, LS>(ctx: Chain<LH, LS>, client_id: String);
    fn bootstrap_channel<LH, LS>(ctx: Chain<LH, LS>, connection_id: String);


    fn client_create_msg(ctx: Chain<LH, LS>, height: Height) -> MsgEnvelope;
    fn client_update_msg(ctx: Chain<LH, LS>, height: Height) -> MsgEnvelope;
    ..

    fn connection_open_init_msg(ctx: Chain<LH, LS>, client_id: String) -> MsgEnvelope;
    // uses IBC events
    fn connection_open_try_msg(ctx_a: Chain<LH, LS>, ctx_b: Chain<RH, RS>) -> MsgEnvelope;
    ..

    // similarly for channels and packets
}

struct Ibc<R: Router, S: Store> {
    context: IbcStore<S>,
    router: R
}

pub trait Host {
    type Block;
    fn chain_id(&self) -> String;
    fn generate_block(&self, hash: Vec<u8>) -> Self::Block;
    fn generate_client_state(&self) -> Self::Block;
    fn generate_consensus_state(&self) -> Self::Block;
}

pub trait Store {
    type Error;
    fn get(&self, key: &[u8]) -> Result<Vec<u8>, Self::Error>;
    fn set(&mut self, key: &[u8], value: Vec<u8>) -> Result<Vec<u8>, Self::Error>
    fn commit(&mut self) -> Result<(), Self::Error>;
    fn root_hash(&self) -> Result<Vec<u8>, Self::Error>;
    fn get_proof(&self, key: &[u8]) -> Result<Vec<u8>, Self::Error>;
}
```

The idea is to maintain multiple `Chain` instances and use `Relayer` to
facilitate the IBC among them. The `Chain` uses `blocks` and `store` to model
chain history and storage. `Chain::advance_height` is used to commit changes to
store and generate a new block with a new height and commitment root.
`Chain::dispatch` is used to submit messages to the IBC module and make changes
to the storage.

We should use emitted IBC events and log messages in our tests too. We should
have utility functions to parse and assert these in our tests.

## Status

Proposed

## Consequences

This ADR pays the technical debt of the existing testing framework.

### Positive

The future tests will be more readable and maintainable. The test framework
becomes modular and heavily leverages Rust's trait system. Even the `ibc-rs`
users may benefit from this framework to test their implementations of `ibc-rs`
traits.

### Negative

This requires a significant refactoring of the existing tests. Since this may
take some time, the parallel development on `main` branch may conflict with this
work.

## Future

The IBC standard is being implemented in different languages. We should
investigate a DSL to write the IBC test scenarios. This way we add portability
to our tests and check compatibility with other implementations, such as ibc-go.
