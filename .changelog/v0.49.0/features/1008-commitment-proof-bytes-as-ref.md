- `[ibc-core-commitment-types`] implement `AsRef<Vec<u8>>` and
  `AsRef<[u8]>` for `CommitmentProofBytes` so it’s possible to gain
  access to the proof byte slice without having to own the object.
  ([#1008](https://github.com/cosmos/ibc-rs/pull/1008))
