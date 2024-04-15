- [ibc] Enhance portability of custom `Validation/ExecutionContext` traits under
  ICS-07. This upgrade relocates them, along with the rest of the
  client-relevant context APIs, under ICS-02, with the traits renamed to
  `ExtClientValidationContext` and `ExtClientExecutionContext` for improved
  self-description ([\#1163](https://github.com/cosmos/ibc-rs/issues/1163))
