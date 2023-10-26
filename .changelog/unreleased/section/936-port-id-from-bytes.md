- Add `PortId::from_bytes` method for constructing PortId from bytes
  slice caller doesn’t know is valid UTF-8.  Useful when dealing with
  binary protocols.  ([\#936](https://github.com/cosmos/ibc-rs/pull/936))
