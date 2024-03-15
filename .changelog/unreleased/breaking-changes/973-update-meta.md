- [ibc-core-client] Merge client update time and height modification method
  pairs into one, that is replace
  a) client_update_{time,height} by update_meta,
  b) store_update_{time,height} by store_update_meta and
  c) delete_update_{time,height} by delete_update_meta.
  ([\#973](https://github.com/cosmos/ibc-rs/issues/973))
