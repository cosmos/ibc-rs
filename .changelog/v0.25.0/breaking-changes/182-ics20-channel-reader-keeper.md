- `Ics20Context` no longer requires types implementing it to implement `ChannelReader` and `ChannelKeeper`, and instead depends on the `Ics20ChannelKeeper`
  and `SendPacketReader` traits. Additionally, the `send_packet` handler now requires the calling context to implement `SendPacketReader` and returns
  a `SendPacketResult`.
  ([#182](https://github.com/cosmos/ibc-rs/issues/182))