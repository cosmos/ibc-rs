This release continues the trend of further decoupling dependencies between the different `ibc-rs`
sub-crates and modules. 

In particular, the `prost` dependency is now only imported in the `ibc-primitives`
crate; note that error variants originating from `prost` have largely been removed, which is a breaking
change. The `bytes` dependency was also removed. Additionally, `CommitmentProofBytes` can now be 
accessed without explicit ownership of the object which the proof is being queried for. 

Some other notable improvements include making the CosmWasm check more rigorous, streamlining the 
`Msg` trait and renaming it to `ToProto`, as well as implementing custom JSON and Borsh `ChainId`
deserialization. 

There are no consensus-breaking changes.
