- Use `<&str>::deserialize` instead of `String::deserialize` to avoid an extra
  allocation ([#496](https://github.com/cosmos/ibc-rs/issues/496))