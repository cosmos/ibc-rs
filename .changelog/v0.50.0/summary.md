This release introduces several noteworthy libraries. A standout addition is the
implementation of the ICS-721 NFT transfer application, enabling the transfer of
NFT packets across chains that support this capability.

In addition, It incorporates the ICS-08 Wasm light client data structure and
types. This empowers light client developers to create CosmWasm contracts for
deployment on Cosmos chains compatible with the version of `ibc-go` supporting
ICS-08 Wasm client.

Moreover, it exposes additional convenient types and serializers through
`ibc-primitives` and includes a more flexible constructor for `MockContext`
types within the `ibc-testkit` crate, allows for testing with diverse parameter
combinations.

There are no consensus-breaking changes.
