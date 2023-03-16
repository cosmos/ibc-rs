- Refactor `is_send/receive_enabled` interfaces within the transfer application
  to `can_send/receive_coins` returning `Result<(), TokenTransferError>` type
  for a better failure handler
  ([#508](https://github.com/cosmos/ibc-rs/issues/508))