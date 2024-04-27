- [ibc-client-tendermint-types] Box header fields inside of Misbehaviour type so
  that the type is smaller (i.e. trade size of the type for heap memory).  This
  prevents stack overflows on systems with small stack (e.g. Solana).
  ([\#1145](https://github.com/cosmos/ibc-rs/pull/1145))
