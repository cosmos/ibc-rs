This release adds necessary APIs for featuring consensus state pruning and
implements pertaining logic for Tendermint light clients. This prevents
unlimited store growth. Additionally, we've enhanced ibc-rs compatibility with
no-float environments making Wasm compilation smoother and updated main
dependencies including `prost` to v0.12, `ibc-proto-rs` to v0.37, and
`tendermint-rs` to v0.34, ensuring the latest advancements.

There are no consensus-breaking changes.
