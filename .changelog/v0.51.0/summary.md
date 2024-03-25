This release introduces a few changes for better customizability. The main one is modularizing ICS-24, ICS-02, and ICS-07 trait implementations. This change empowers developers to write Rust light clients succinctly in a smart-contract context like CosmWasm. Also, the default Tendermint client state verifier is now detached to support custom verifiers, if required.

In addition, this version fixes a bug where the consensus state is incorrectly stored when a header with an older height is submitted.

Furthermore, a set of new host keys is added. This makes `ibc-rs` more consistent with the storage access of `ibc-go`. Also, access to client update information is merged into a single method; instead of individual details.

This release updates the `ibc-proto-rs` dependency to `v0.42.2`. This takes account of the updated `MsgUpdateClient` and deprecates `MsgSubmitMisbehaviour`.

Finally, the minimum supported Rust version is corrected and updated to `1.71.1`.

There are no consensus-breaking changes.
