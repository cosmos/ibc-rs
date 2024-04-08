- [ibc] Enable the use of custom hashing functions by making `MerkleProof`
  validation methods generic over `HostFunctionsProvider` and using
  `hash_with<H>()` instead of `hash()` wherever validators' hash is computed.
  ([\#1147](https://github.com/cosmos/ibc-rs/issues/1147))
