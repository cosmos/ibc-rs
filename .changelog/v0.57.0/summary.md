
This release introduces improvements to better support the **Packet Forward
Middleware**, including asynchronous packet acknowledgments and enhanced
contextual parsing of sender and receiver instances in ICS-20. The update
removes the reliance on `TryFrom<Signer>` for parsing, improving flexibility in
transaction handling.

Additionally, the "arbitrary" feature flag now enables the implementation of the
`Arbitrary` trait, enhancing testing capabilities. Furthermore, `Serde` support
has been added for `Height` without `revision_number`, facilitating better
interoperability with CosmWasm light clients operating on the `08-wasm` module
of `ibc-go`.

There are no consensus-breaking changes in this release.
