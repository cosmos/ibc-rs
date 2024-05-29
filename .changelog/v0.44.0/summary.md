The goal with this release was to replace `ClientState::{confirm_not_frozen, expired}()` with `ClientState::status()` ([#536](https://github.com/cosmos/ibc-rs/issues/536)). Updating basecoin-rs with the new changes exposed the shortcomings of having `SendPacket*Context` be supertraits of `TokenTransfer*Context`, which in turned exposed the shortcomings of having `Router` be a supertrait of `ValidationContext`. Hence, we decoupled everything!

There are consensus-breaking changes.
