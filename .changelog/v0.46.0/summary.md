This release introduces vital bug fixes, including removal of an incorrect
validation during a Tendermint client update and the addition of a missing state
update during a successful client upgrade ensuring the inclusion of the host's
height and timestamp in the store.

Additionally, it eliminates the `safe-regex` dependency, and restructures IBC
query implementations under the previous `grpc` feature flag and moves it to a
separate crate called as `ibc-query`.

There are consensus-breaking changes.
