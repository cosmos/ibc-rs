This release adds some exciting changes, improvements, and new features to ibc-rs.
First and foremost, support for the IBC protocol's [client recovery][client-recovery]
mechanism has been implemented, which provides a route for frozen and expired IBC clients
to be re-instated following a successful governance vote. In addition, a new crate,
`ibc-client-cw`, facilitates CosmWasm contract creation for light clients built using
`ibc-rs`. Lastly, the ics07 tendermint light client has also been packaged and included
as a CosmWasm contract.

This release also includes a myriad of other bug-fixes and improvements,
such as enhancing the portability of ibc-rs's Validation and Execution Context traits,
as well as fixing an incompatibility with how ibc-rs parses `PrefixDenom`s compared
to ibc-go, among many others.

The minimum-supported Rust version has been updated to `1.72`. `ibc-proto` has been
bumped to `0.43`. `tendermint` has been bumped to `0.35`. `ibc-derive` has been
bumped to `0.7`.

There are no consensus-breaking changes as part of this release.
