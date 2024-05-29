# CosmWasm Check

This crate contains a simple CosmWasm contract, which incorporates `ibc-rs`, to ensure the compatibility of `ibc-rs` with the CosmWasm environment.

CosmWasm is `std` targeting `wasm32-unknown-unknown` with `stable` rust which provides precompiled extensions `alloc/core/std` which has only features available in `wasmd` runner and passed `cosmwasm-check` (precompile extensions check and cosmwasm-check are not same set).
