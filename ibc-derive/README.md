# IBC Derive

This crate provides a set of procedural macros for deriving IBC-related traits
defined in the [`ibc-rs`](https://github.com/cosmos/ibc-rs) repository, reducing
the amount of boilerplate code required to implement them. It contains macro
implementations for the following traits:

- [ClientState](./../ibc-core/ics02-client/context/src/client_state.rs)
- [ConsensusState](./../ibc-core/ics02-client/context/src/consensus_state.rs)
