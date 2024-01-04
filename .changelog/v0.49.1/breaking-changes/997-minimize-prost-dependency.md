- `[ibc]` Minimize `prost` dependency by introducing `ToVec` trait
  - Now `prost` is only imported in `ibc-primitives` crate
  - Remove error variants originating from `prost` (Breaking change)
  - Eliminate the need for the `bytes` dependency
 ([\#997](https://github.com/cosmos/ibc-rs/issues/997))
