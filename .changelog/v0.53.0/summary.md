This release overhauls the `ibc-testkit` crate such that it is now able to:

- simulate more realistic IBC workflows by utilizing real IBC and relayer
  interfaces (as opposed to mocked versions)
- validate code paths that were not easily testable beforehand, such as Merkle
  proof generation
- compose tests in a much more succinct and readable fashion

Note that the drastic changes made to `ibc-testkit`'s structs and types are
breaking changes.

For more information and background context on the changes to `ibc-testkit` and
the rationale behind the overhaul, please refer to [ADR 009][adr-009].

This release also includes a fix to the proof verification logic for
`PacketTimeout`s, which was verifying an incorrect field. It also bumps the
minimum-supported version of `ibc-proto` to 0.44, and the version of
`tendermint` to 0.36. Note that the minimum-supported Rust version was reverted
back to 1.71.1.

[adr-009]: https://github.com/cosmos/ibc-rs/blob/main/docs/architecture/adr-009-revamp-testkit.md
