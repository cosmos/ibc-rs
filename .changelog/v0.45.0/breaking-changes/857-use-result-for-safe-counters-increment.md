- Allow hosts to handle overflow cases in `increase_*_counter` methods by
  returning `Result<(),HandlerError>` type.
  ([#857](https://github.com/cosmos/ibc-rs/issues/857))
