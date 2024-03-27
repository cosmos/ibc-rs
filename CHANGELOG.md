# CHANGELOG

## v0.51.0

*March 26, 2024*

This release introduces a few changes for better customizability. The main one is modularizing ICS-24, ICS-02, and ICS-07 trait implementations. This change empowers developers to write Rust light clients succinctly in a smart-contract context like CosmWasm. Also, the default Tendermint client state verifier is now detached to support custom verifiers, if required.

In addition, this version fixes a bug where the consensus state is incorrectly stored when a header with an older height is submitted.

Furthermore, a set of new host keys is added. This makes `ibc-rs` more consistent with the storage access of `ibc-go`. Also, access to client update information is merged into a single method; instead of individual details.

This release updates the `ibc-proto-rs` dependency to `v0.42.2`. This takes account of the updated `MsgUpdateClient` and deprecates `MsgSubmitMisbehaviour`. Also, `ibc-derive` dependency is updated to `v0.6.1`.
Finally, the minimum supported Rust version is corrected and updated to `1.71.1`.

There are no consensus-breaking changes.

### BREAKING CHANGES

- [ibc-core] Update `MsgUpdateClient` handler to accept misbehaviour reports via
  its `client_message` field
  ([\#835](https://github.com/cosmos/ibc-rs/issues/835))
- [ibc-core-client] Merge client update time and height modification method
  pairs into one, that is replace
  a) client_update_{time,height} by update_meta,
  b) store_update_{time,height} by store_update_meta and
  c) delete_update_{time,height} by delete_update_meta.
  ([\#973](https://github.com/cosmos/ibc-rs/issues/973))
- [ibc] Refactor client relevant APIs for improved modularity and allow
  standalone ICS-02 integration
  ([\#1114](https://github.com/cosmos/ibc-rs/issues/1114))
- [ibc] Increase minimum supported Rust version to 1.71.1
  ([\#1118](https://github.com/cosmos/ibc-rs/issues/1118))
- [ibc] Upgrade `ibc-proto-rs` to `v0.42.2`
  ([\#1125](https://github.com/cosmos/ibc-rs/pull/1125))

### BUG FIXES

- [ibc] Add missing dependencies for some feature flags across multiple crates
  ([\#1059](https://github.com/cosmos/ibc-rs/issues/1059))
- [ibc-client-tendermint-types] Ease frozen Height check in the tendermint
  `ClientState` protobuf deserialization, and consequently include frozen client
  check for client creation path.
  ([\#1061](https://github.com/cosmos/ibc-rs/issues/1061)),
  ([\#1063](https://github.com/cosmos/ibc-rs/pull/1063))
- [ibc-client-tendermint] Use header height for Tendermint consensus state storage
  ([\#1080](https://github.com/cosmos/ibc-rs/issues/1080))
- [ibc] Upgrade `serde_json` to "1.0.1" to address an stack overflow issue
  within the `serde-json-wasm` crate
  ([\#1083](https://github.com/cosmos/ibc-rs/pull/1083))
- [ibc] Resolve potential `base64` dependency resolution issue by bringing it to
  the workspace `Cargo.toml`
  ([\#1084](https://github.com/cosmos/ibc-rs/issues/1084))
- [ibc-client-tendermint-types] Check ics23 proof specs for empty depth range.
  ([\#1100](https://github.com/cosmos/ibc-rs/issues/1100))

### FEATURE

- [ibc-core-host] Add remaining storage paths.
  ([\#1065](https://github.com/cosmos/ibc-rs/issues/1065))
- [ibc-core-host] Add iteration key for cross-compatibility with `ibc-go` used
  for iterating over consensus states
  ([\#1090](https://github.com/cosmos/ibc-rs/issues/1090))

### IMPROVEMENTS

- [ibc-core] Deprecate `ChannelEnd::order_matches` method
  ([\#394](https://github.com/cosmos/ibc-rs/issues/394))
- [ibc-apps] Ease `serde` derive on `ICS-20` and `ICS-721` types
  ([\#1060](https://github.com/cosmos/ibc-rs/pull/1060))
- [ibc-data-types] Refactor `Default` implementations with concrete names
  ([\#1074](https://github.com/cosmos/ibc-rs/issues/1074))
- [ibc-core] Deprecate `MsgSubmitMisbehaviour` in favor of `MsgUpdateClient` for
  submitting misbehaviour reports
  ([\#1077](https://github.com/cosmos/ibc-rs/issues/1077))
- [ibc-core-host] Improve path segment access by exposing path prefixes and
  implementing some convenient parent/leaf methods
  ([\#1089](https://github.com/cosmos/ibc-rs/issues/1089))
- [ibc-client-tendermint] Detach client state verifier such that users have a
  way to utilize custom verifiers
  ([\#1097](https://github.com/cosmos/ibc-rs/pull/1097))
- [ibc-primitives] Use `let-else` over `downcast!()` and remove `utils/macros`
  module as a result ([\#1118](https://github.com/cosmos/ibc-rs/issues/1118))
- [ibc-core] Remove unnecessary shadowing with same value
  ([\#1120](https://github.com/cosmos/ibc-rs/issues/1120))
- [ibc-core-connection] Remove `State::Uninitialized` check while parsing
  `ConnectionEnd` from Protobuf
  ([\#1123](https://github.com/cosmos/ibc-rs/issues/1123))

## v0.50.0

*January 24, 2024*

This release introduces several noteworthy libraries. A standout addition is the
implementation of the ICS-721 NFT transfer application, enabling the transfer of
NFT packets across chains that support this capability.

In addition, It incorporates the ICS-08 Wasm light client data structure and
types. This empowers light client developers to create CosmWasm contracts for
deployment on Cosmos chains compatible with the version of `ibc-go` supporting
ICS-08 Wasm client.

Furthermore, this release addresses the issue with the macro derivation of the
`ClientState` when contexts include generic types, exposes additional convenient
types and serializers through `ibc-primitives` and includes a more flexible
constructor for `MockContext` types within the `ibc-testkit` crate, allowing to
write tests with diverse parameter combinations.

There are no consensus-breaking changes.

### BREAKING CHANGES

- [ibc] Bump `ibc-proto-rs` to v0.41.0
  ([\#1036](https://github.com/cosmos/ibc-rs/pull/1036)).

### BUG FIXES

- [ibc-derive] Refactor `ClientState` macro derivation to handle contexts with
  generic types. ([\#910](https://github.com/cosmos/ibc-rs/issues/910))
- [ibc-derive] Adapt macro derivations to integrate with projects dependent on
  `ibc-core` ([\#999](https://github.com/cosmos/ibc-rs/issues/999)).

### FEATURES

- [ibc-app-nft-transfer] Implement ICS-721 NFT transfer application
  ([\#346](https://github.com/cosmos/ibc-rs/issues/346))
- [ibc-client-wasm-types] Implement ICS-08 Wasm light client domain types
  ([\#1030](https://github.com/cosmos/ibc-rs/issues/1030)).

### IMPROVEMENTS

- [ibc-data-types] Re-export clients' domain type from `ibc-data-types`
  ([\#1041](https://github.com/cosmos/ibc-rs/pull/1041)).
- [ibc-testkit] Deprecate `MockContext::new*` in favor of `MockContextConfig`.
  ([\#1042](https://github.com/cosmos/ibc-rs/issues/1042))
- [ibc-testkit] Remove field access of `MockContext`.
  ([\#1043](https://github.com/cosmos/ibc-rs/issues/1043))
- [ibc-testkit] Deprecate `MockContext::with_client*` in favor of
  `MockContext::with_client_config`.
  ([\#1049](https://github.com/cosmos/ibc-rs/issues/1049))
- [ibc-primitives] Re-export additional google proto types, like `Timestamp`
  and `Duration` for added convenience when developing IBC light clients or
  applications. ([\#1054](https://github.com/cosmos/ibc-rs/pull/1054))
- [ibc-primitives] Relocate `serializers.rs` module to reside within the
  `ibc-primitives` crate extending its utility for a broader range of IBC
  applications. ([\#1055](https://github.com/cosmos/ibc-rs/issues/1055))

## v0.49.1

*January 3, 2024*

This release continues the trend of further decoupling dependencies between the
different ibc-rs sub-crates and modules.

In particular, the `prost` dependency is now only imported in the
`ibc-primitives` crate; note that error variants originating from `prost` have
largely been removed, which is a breaking change. The `bytes` dependency was
also removed. Additionally, `CommitmentProofBytes` can now be accessed without
explicit ownership of the object for which the proof is being queried for.

Some other improvements of note include making the CosmWasm check more rigorous,
streamlining the `Msg` trait and renaming it to `ToProto`, as well as
implementing custom JSON and Borsh `ChainId` deserialization.

There are no consensus-breaking changes.

### BREAKING CHANGES

- `[ibc-app-transfer]` Refactor `send-coins-*()` methods by breaking them down
  into distinct escrow and unescrow methods, enhancing both clarity and
  specificity in functionality.
  ([\#837](https://github.com/cosmos/ibc-rs/issues/837))
- `[ibc-app-transfer]` Add `memo` field to `escrow-coins-*()` and
  `burn-coins-*()` methods, allowing implementors to pass in arbitrary data
  necessary for their use case.
  ([\#839](https://github.com/cosmos/ibc-rs/issues/837))
- `[ibc-core-host-type]` Optimize `IdentifierError` variants and make them
  mutually exclusive. ([\#978](https://github.com/cosmos/ibc-rs/issues/978))
- `[ibc-data-types]` Bump ibc-proto-rs dependency to v0.39.1.
  ([\#993](https://github.com/cosmos/ibc-rs/issues/993))
- `[ibc]` Minimize `prost` dependency by introducing `ToVec` trait
  - Now `prost` is only imported in `ibc-primitives` crate
  - Remove error variants originating from `prost` (Breaking change)
  - Eliminate the need for the `bytes` dependency
 ([\#997](https://github.com/cosmos/ibc-rs/issues/997))
- `[ibc-core-host-types]` Introduce `ClientType::build_client_id` which avoids unnecessary validation.
  ([#1014](https://github.com/cosmos/ibc-rs/issues/1014))
- `[ibc-core-host-types]` Optimise `ClientId::new` to avoid unnecessary validation and temporary
  string allocation. ([#1014](https://github.com/cosmos/ibc-rs/issues/1014))

### FEATURES

- `[ibc-core-commitment-types]` implement `AsRef<Vec<u8>>` and
  `AsRef<[u8]>` for `CommitmentProofBytes` so it’s possible to gain
  access to the proof byte slice without having to own the object.
  ([#1008](https://github.com/cosmos/ibc-rs/pull/1008))

### IMPROVEMENTS

- `[cw-check]` More rigorous CosmWasm check by upgrading dependencies and
  including `std` and `schema` features for `ibc-core`.
  ([\#992](https://github.com/cosmos/ibc-rs/pull/992))
- `[ibc-primitives]` streamline `Msg` trait and rename to `ToProto`
 ([#993](https://github.com/cosmos/ibc-rs/issues/993))
- `[ibc-core-host-types]` Implement custom JSON and Borsh deserialization for `ChainId` ([#996](https://github.com/cosmos/ibc-rs/pull/1013))
- `[ibc-core-client-types]` Add a convenient `Status::verify_is_active` method.
  ([#1005](https://github.com/cosmos/ibc-rs/pull/1005))
- `[ibc-primitives]` Derive `Hash` on `Timestamp` instead of explicit
  implementation ([#1011](https://github.com/cosmos/ibc-rs/pull/1005))
- `[ibc-derive]` Use global paths in generated code by macros to prevent
  namespace conflicts with local modules
  ([#1017](https://github.com/cosmos/ibc-rs/pull/1017))

## v0.48.2

*December 22, 2023*

This patch release resolves two issues. It corrects the packet sequence number
encoding within Timeout message handlers to align with the big-endian format and
addresses a recursive call error during the conversion from connection `State`
to `i32`.

There are no consensus-breaking changes.

### BUG FIXES

- `[ibc-core-host-types]` Encode packet sequence into a big endian bytes.
  ([\#1004](https://github.com/cosmos/ibc-rs/pull/1004))
- `[ibc-core-connection-types]` Fix recursive call in connection `State`
  conversion to `i32` ([\#1010](https://github.com/cosmos/ibc-rs/issues/1010))

## v0.48.1

*November 27, 2023*

This patch release eliminates the `dep:` syntax from the `serde` feature,
addressing potential dependency resolution issue stemming from Rust v1.70.

There are no consensus-breaking changes.

### BUG FIXES

- Fix Cargo test failure with `--no-default-features` flag.
  ([\#770](https://github.com/cosmos/ibc-rs/issues/770))
- Fix dependency resolution by removing the `dep:` syntax in `serde` feature of
  `ibc-app-transfer` crate.
  ([\#987](https://github.com/cosmos/ibc-rs/issues/987))

## v0.48.0

*November 22, 2023*

In this release, we've undertaken a comprehensive overhaul of the **`ibc-rs`**
repository, resulting in a strategic reorganization of the codebase. This
restructuring dissects the implementation of each IBC specification,
categorizing and situating them within relevant libraries. The primary objective
is to elevate `ibc-rs` practicality and enhance user flexibility by providing a
more modular and composable experience.

Users now have the flexibility to choose from a spectrum of dependencies. They
can opt to utilize the entire suite of meta-crates, such as `ibc`, `ibc-core`,
`ibc-clients`, or `ibc-apps`. Alternatively, they can exercise fine-grained
control by selectively importing specific crates. This can involve bringing in
an entire implemented IBC sub-module, like the `ibc-core-client` crate, or
importing only the associated data structures of a module, such as the
`ibc-core-client-types` crate.

Furthermore, this release introduces optimizations centered around construction
and validation of ICS-24 host identifiers, aiming to curtail some heap
allocations, beneficial for resource-constrained hosts.

There are no consensus-breaking changes.

### BREAKING CHANGES

- Move ICS-20 and ICS-27 implementations to the respective part of `ibc-apps`
  and `ibc-clients` crates, as part of the `ibc` crate restructuring effort.
  ([\#716](https://github.com/cosmos/ibc-rs/issues/716))
- Bump `ibc-proto-rs` to v0.38.0
  ([\#949](https://github.com/cosmos/ibc-rs/issues/949))
- Bump minimum supported Rust version to 1.64
  ([\#956](https://github.com/cosmos/ibc-rs/issues/956))
- Restructure `ibc-rs` codebase by organizing it into smaller self-contained,
  modular libraries, enabling the selective import of specific domain types or
  module implementations, either individually or in combination, providing
  enhanced flexibility and ease of use.
  ([\#965](https://github.com/cosmos/ibc-rs/issues/965))

### FEATURES

- Restructure the mock module implementation and separate its codebase into a
  new crate named `ibc-testkit`
  ([\#954](https://github.com/cosmos/ibc-rs/issues/953))
- Provide `Into<String>` for all identifiers types.
  ([\#974](https://github.com/cosmos/ibc-rs/pull/974))

### IMPROVEMENTS

- Re-export essential proto types from the underlying `ibc-*-*-types` crates,
  removing the necessity for a direct dependency on `ibc-proto` in projects
  integrating `ibc-rs` ([\#697](https://github.com/cosmos/ibc-rs/issues/697))
- Rename `{submodule}.rs` with corresponding `{submodule}` directory to
  `{submodule}/mod.rs` ([\#771](https://github.com/cosmos/ibc-rs/issues/771))
- Add From implementation for ICS26 enum types to make it simpler to construct
  the types. ([\#938](https://github.com/cosmos/ibc-rs/pull/938))
- Reduce vector allocations in Commitment computation.
  ([\#939](https://github.com/cosmos/ibc-rs/pull/939))
- Support chain identifiers without the `{chain_name}-{revision_number}` pattern
  of Tendermint chains. ([\#940](https://github.com/cosmos/ibc-rs/issues/940)).
- Remove redundant `String` creation in `validate_prefix_length`
  ([\#943](https://github.com/cosmos/ibc-rs/issues/943)).
- Remove redundant `#[test_log::test]` attributes in test modules
  ([\#948](https://github.com/cosmos/ibc-rs/issues/948))
- Remove the default value and implementation for `PortId`
  ([\#951](https://github.com/cosmos/ibc-rs/issues/951))
- Expose domain message types under the `ics04_channel` as public
  ([\#952](https://github.com/cosmos/ibc-rs/issues/952))
- Enhance dependency management with workspace inheritance
  ([\#955](https://github.com/cosmos/ibc-rs/issues/955))
- Simplify and refactor ICS-24 identifier validation logic.
  ([\#961](https://github.com/cosmos/ibc-rs/issues/961))
- Reduce heap allocation by using `str` instead of `String` places we convert
  domain event attributes to the ABCI event attributes
  ([\#970](https://github.com/cosmos/ibc-rs/issues/970))
- Expose various fields, types and functions in `ibc-rs` as public including:
  - `validate` and `execute` handler functions for all the IBC message types.
  - `TYPE_URL` constants.
  - Any private fields within the domain message types.
  - Any private fields within the Tendermint `ClientState` and `ConsensusState`
  ([\#976](https://github.com/cosmos/ibc-rs/issues/976))

## v0.47.0

*October 19, 2023*

This release adds necessary APIs for featuring consensus state pruning and
implements pertaining logic for Tendermint light clients. This prevents
unlimited store growth. Additionally, we've enhanced ibc-rs compatibility with
no-float environments making Wasm compilation smoother and updated main
dependencies including `prost` to v0.12, `ibc-proto-rs` to v0.37, and
`tendermint-rs` to v0.34, ensuring the latest advancements.

There are no consensus-breaking changes.

### FEATURES

- Implement consensus state pruning for Tendermint light clients. ([\#600](https://github.com/cosmos/ibc-rs/issues/600))

### IMPROVEMENTS

- Add test for expired client status.
  ([\#538](https://github.com/cosmos/ibc-rs/issues/538))

- Fix compilation issue with Wasm envs because of floats. ([\#850](https://github.com/cosmos/ibc-rs/issues/850))
  - Use `serde-json-wasm` dependency instead of `serde-json` for no-floats support
  - Add CI test to include CosmWasm compilation check

- Change `mocks` feature to imply `std` since it requires
  Timestamp::now to work.
  ([\#926](https://github.com/cosmos/ibc-rs/pull/926))
- Return PacketStates instead of paths from packet_commitments and
  packet_acknowledgements. ([\#927](https://github.com/cosmos/ibc-rs/issues/927))
- Remove `AnySchema` as `JsonSchema` derive on `Any` now accessible through
  `ibc-proto-rs`. ([#929](https://github.com/cosmos/ibc-rs/issues/929))

## v0.46.0

*October 12, 2023*

This release introduces vital bug fixes, including removal of an incorrect
validation during a Tendermint client update and the addition of a missing state
update during a successful client upgrade ensuring the inclusion of the host's
height and timestamp in the store.

Additionally, it eliminates the `safe-regex` dependency, and restructures IBC
query implementations under the previous `grpc` feature flag and moves it to a
separate crate called as `ibc-query`.

There are consensus-breaking changes.

### BREAKING CHANGES

- Relocate `*_update_time` and `*_update_height` to the client contexts' traits
  for improved access by light clients
  ([#914](https://github.com/cosmos/ibc-rs/issues/914))

### BUG FIXES

- Remove an incorrect validation during tendermint client update
  ([\#911](https://github.com/cosmos/ibc-rs/issues/911))
- Add missing update in the state, which should include the host's height and
   timestamp when a successful client upgrade take place.
   ([\#913](https://github.com/cosmos/ibc-rs/issues/913))

### IMPROVEMENTS

- Remove `safe-regex` dependency
  ([\#875](https://github.com/cosmos/ibc-rs/issues/875))
- Enhance IBC query methods usability and code organization
  - The implementation of query methods is now publicly accessible as standalone functions.
  - `grpc` feature now lives as a separate crate called as `ibc-query`
  ([#896](https://github.com/cosmos/ibc-rs/issues/896))
- Re-export ibc proto types from `ibc-proto-rs`` for dep

## v0.45.0

*September 20, 2023*

This release introduces a new API under the `grpc` feature flag, which has ibc-rs expose grpc endpoints that the hermes relayer needs. Furthermore, `no_std` support for the `serde` feature has been restored, accompanied by other miscellaneous changes.
There are no consensus-breaking changes.

### BREAKING CHANGES

- Bump tendermint-rs to v0.33.0
  ([#785](https://github.com/cosmos/ibc-rs/issues/785))
- Bump ibc-proto-rs to v0.34.0
  ([#790](https://github.com/cosmos/ibc-rs/issues/790))
- Allow hosts to handle overflow cases in `increase_*_counter` methods by
  returning `Result<(),ContextError>` type.
  ([#857](https://github.com/cosmos/ibc-rs/issues/857))
- logger and event emitter methods return `Result<(), ContextError>` type.
  ([#859](https://github.com/cosmos/ibc-rs/issues/859))
- Bump `ibc-proto-rs` to v0.35.0 along with some other minor dependency updates
  ([#873](https://github.com/cosmos/ibc-rs/issues/873))

### BUG FIXES

- Fix compilation error of v0.41.0 by restoring no_std support for serde
  feature ([#741](https://github.com/cosmos/ibc-rs/issues/741))
- Replace mutable ref with immutable ref in validate handler
  ([\#863](https://github.com/cosmos/ibc-rs/issues/863))

### FEATURES

- Blanket implementation of core gRPC services
  ([\#686](https://github.com/cosmos/ibc-rs/issues/686))

### IMPROVEMENTS

- Switch to domain Tendermint event type instead of proto for the
  `upgrade_client_proposal_handler` return
  ([#838](https://github.com/cosmos/ibc-rs/issues/838))
- Bump ibc-proto to v0.34.1 and borsh to v0.10
  ([#844](https://github.com/cosmos/ibc-rs/issues/844))
- Add borsh derive for `MsgTransfer`
  ([#845](https://github.com/cosmos/ibc-rs/pull/845))
- Add borsh derive for `MsgEnvelope`
  ([#846](https://github.com/cosmos/ibc-rs/pull/846))
- Derive `PartialEq`, `Eq` for `MsgEnvelope`
  ([#847](https://github.com/cosmos/ibc-rs/pull/847))
- Organize imports grouping and granularity using `rustfmt.toml`
  ([#848](https://github.com/cosmos/ibc-rs/issues/848))
- Add `JsonSchema` derive for `MsgEnvelope`
  ([#856](https://github.com/cosmos/ibc-rs/pull/856))
- Remove unused code snippets and move serializer roundtrip test to `serializers.rs`
  ([#869](https://github.com/cosmos/ibc-rs/issues/869))

## v0.44.2

*October 12, 2023*

This release is a critical patch release that introduces a vital fix by removing
an incorrect validation during a Tendermint client update.

There are no consensus-breaking changes.

## v0.44.1

*August 4, 2023*

This release fixes a bug with the `UpdateClient` event where the `header` field was not properly encoded.

There are no consensus-breaking changes.

### BUG FIXES

- Remove traces of deprecated `mocks-no-std` feature
  ([#819](https://github.com/cosmos/ibc-rs/issues/821))
- Utilize encoded bytes from `Any` for the `header` field of `UpdateClient`
  event
  ([#821](https://github.com/cosmos/ibc-rs/issues/821))

## v0.44.0

*August 4, 2023*

The goal with this release was to replace `ClientState::{confirm_not_frozen, expired}()` with `ClientState::status()` ([#536](https://github.com/cosmos/ibc-rs/issues/536)). Updating basecoin-rs with the new changes exposed the shortcomings of having `SendPacket*Context` be supertraits of `TokenTransfer*Context`, which in turned exposed the shortcomings of having `Router` be a supertrait of `ValidationContext`. Hence, we decoupled everything!

There are consensus-breaking changes.

### BREAKING CHANGES

- [ibc-derive] Replace `ClientState::{confirm_not_frozen, expired}()` with `ClientState::status()`
  ([#536](https://github.com/cosmos/ibc-rs/issues/536))
- Decouple `TokenTransfer{Validation,Execution}` from `SendPacket{Validation,Execution}`
  ([#786](https://github.com/cosmos/ibc-rs/issues/786))
- Decouple `Router` from `ValidationContext`
  ([#788](https://github.com/cosmos/ibc-rs/pull/788))
- Simplify Module lookup in the `Router` trait
 ([#802](https://github.com/cosmos/ibc-rs/issues/802))

## v0.43.1

*July 31, 2023*

This release bumps ibc-proto to v0.32.1, resolving issue with token transfer
deserialization for cases with no memo field provided. It also includes various
enhancements and bug fixes, such as reorganized acknowledgement types, enhanced
`ChainId` validation, improved `from_str` height creation, synchronized channel
event namings for consistency.

There are consensus-breaking changes.

### BREAKING CHANGES

- Organize acknowledgement types under the `ics04_channel` module to get
  accessible by any applications.
  ([#717](https://github.com/cosmos/ibc-rs/issues/717))
- Sync field and method namings of ics04 events with the convention
  ([#750](https://github.com/cosmos/ibc-rs/issues/750))
- use ibc_proto::protobuf::Protobuf to replace tendermint_proto::Protobuf
  ([#754](https://github.com/cosmos/ibc-rs/pull/754))
- Enhancements and fixes to `ChainId` impls and validation.
  ([#761](https://github.com/cosmos/ibc-rs/issues/761))
- Use `Vec<u8>` for HeaderAttribute instead of `Any`
  ([#764](https://github.com/cosmos/ibc-rs/issues/764))
- Serde: Schema for Coin/Transfer, Amount is string
  ([#772](https://github.com/cosmos/ibc-rs/issues/772))

### BUG-FIXES

- Tendermint ConsensusState -> Any can crash if out of memory
  ([#747](https://github.com/cosmos/ibc-rs/issues/747))
- `Height::from_str` accepts invalid heights
  ([#752](https://github.com/cosmos/ibc-rs/issues/752))
- Add serde serialization and deserialization to Packet Receipt
  ([#794](https://github.com/cosmos/ibc-rs/pull/794))

### IMPROVEMENTS

- Scale encoding for ICS-20 transfer message
  ([#745](https://github.com/cosmos/ibc-rs/issues/745))
- Add test to ensure `PacketData` keeps proper JSON encoding
  ([#763](https://github.com/cosmos/ibc-rs/issues/763))

## v0.42.0

*July 5, 2023*

This release primarily implements ADR 7. It also includes a number of miscellaneous improvements.

There are no consensus-breaking changes.

### BREAKING CHANGES

- Implement ADR 7, where `ClientState` objects are now statically dispatched instead
   of dynamically dispatched.
([#296](https://github.com/cosmos/ibc-rs/issues/296))
- Revise the `verify_upgrade_client` method to utilize the domain-specific
  `MerkleProof` type
  ([#691](https://github.com/cosmos/ibc-rs/issues/691))
- Revise the `ChainId::new` method so that rather than taking String argument
  it borrows a str.  ([#721](https://github.com/cosmos/ibc-rs/issues/721))
- Modify `MsgUpgradeClient` struct to utilize `CommitmentProofBytes` and
  apply some refinements around upgrade client methods and impls respectively.
  ([#739](https://github.com/cosmos/ibc-rs/issues/739))
- Remove `Router::has_route`
  ([#503](https://github.com/cosmos/ibc-rs/issues/503))

### FEATURES

- Upgrade to tendermint v0.32, ibc-proto-rs v0.32, ics23 v0.10, and get
  prehash_key_before_comparison field available for the `ProofSpec`
  ([#640](https://github.com/cosmos/ibc-rs/issues/640))

### IMPROVEMENT

- Remove Header trait ([#617](https://github.com/cosmos/ibc-rs/issues/617))
- Deny use of `unwrap()` throughout the crate
  ([#655](https://github.com/cosmos/ibc-rs/issues/655))
- `ChainId` should serialize itself without using `tendermint::chain::Id`
  ([#729](https://github.com/cosmos/ibc-rs/issues/729))
- use `FromStr` in client_type functions to construct `ClientType`
  ([#731](https://github.com/cosmos/ibc-rs/pull/731))

## v0.41.0

*May 23, 2023*

This release bumps ibc-proto to v0.30.0 and tendermint to v0.31, and provides utilities for chain upgrades (Tendermint only).

There are consensus-breaking changes.

### BREAKING CHANGES

- Support for upgrade client proposal by featuring helper contexts and domain types 
  ([#420](https://github.com/cosmos/ibc-rs/issues/420))
- Remove unused `ClientState` methods
  ([#681](https://github.com/cosmos/ibc-rs/issues/681))
- Bump ibc-proto to v0.30.0 and tendermint to v0.31
  ([#689](https://github.com/cosmos/ibc-rs/issues/689))

### BUG FIXES

- Encode upgraded client/consensus states for upgrade_client validation using `prost::Message`
  from pros ([#672](https://github.com/cosmos/ibc-rs/issues/672))

### FEATURES

- Timestamp ser and der failed on borsh feature
  ([#687](https://github.com/cosmos/ibc-rs/issues/687))

### IMPROVEMENTS

- Clarify usage of `upgrade_path` for handling upgrade proposals
  ([#141](https://github.com/cosmos/ibc-rs/issues/141))
- Refactor tests for upgrade_client implementation
  ([#385](https://github.com/cosmos/ibc-rs/issues/385))
- Exclude `ClientState::new()` checks from proto ClientState conversion
  ([#671](https://github.com/cosmos/ibc-rs/issues/671))
- Remove redundant #[allow(clippy::too_many_arguments)]
 ([#674](https://github.com/cosmos/ibc-rs/issues/674))
- Token transfer: Make `Amount` type less restrictive
  ([#684](https://github.com/cosmos/ibc-rs/issues/684))

## v0.40.0

*May 8, 2023*

This release primarily consolidated the modules in the ibc-rs crate, removed many legacy items, and documented every item in the crate. This represents a big step towards v1.0. Very few items changed name; most were just moved to elsewhere in the module tree. Perhaps a good heuristic to fix the breaking changes is the remove the faulty `use` statements, and have your editor re-import the item.

There were also a few minor validation checks missing, which we added. These were pretty much the last remaining known ones.

There are consensus-breaking changes.

### BREAKING CHANGES

- Add missing validation checks for all the IBC message types
  ([#233](https://github.com/cosmos/ibc-rs/issues/233))
- Reduce and consolidate the amount of public modules exposed
  ([#235](https://github.com/cosmos/ibc-rs/issues/235))
- Separate validation/execution handlers from context API
  ([#539](https://github.com/cosmos/ibc-rs/issues/539))
- Make `TYPE_URL`s private ([#597](https://github.com/cosmos/ibc-rs/issues/597))

### FEATURES

- Add parity-scale-codec, borsh, serde feature for *Path
  ([#652](https://github.com/cosmos/ibc-rs/issues/652))

### IMPROVEMENTS

- Document every method of `ValidationContext` and `ExecutionContext`
  ([#376](https://github.com/cosmos/ibc-rs/issues/376))

## v0.39.0

*May 2, 2023*

This release primarily adds support for the `memo` field to the token transfer
app (ICS 20). This required updating ibc-proto-rs and tendermint-rs dependencies
as well.

There are consensus-breaking changes.

### BREAKING CHANGES

- Bump ibc-proto to v0.29.0, bump tendermint to v0.30.0, and add `memo` field to
  `PacketData` ([#559](https://github.com/cosmos/ibc-rs/issues/559))
- Add missing `ClientType` and `ClientId` validation checks
  ([#621](https://github.com/cosmos/ibc-rs/issues/621))

### FEATURES

- Define a new `ValidationContext::validate_message_signer` method to allow
  validation of the `signer` field in messages across all handlers.
  ([#619](https://github.com/cosmos/ibc-rs/issues/619))

## v0.38.0

*April 24, 2023*

This release involves splitting the newly defined `MsgUpdateClient` type in
v0.37.0 into distinct IBC message structs: `MsgUpdateClient` and
`MsgSubmitMisbehaviour`. Additionally, we made improvements to the `Version`
validations in connection and channel handshakes, discarded now-unused
`store_client_type` interface, and removed `IbcEventType` to enable each IBC
event variant to define its own set of event types.

There are consensus-breaking changes

### BREAKING CHANGES

- Remove `store_client_type` interface as it is not included in the IBC spec anymore.
  ([#592](https://github.com/cosmos/ibc-rs/issues/592))
- Code clean-up remained from v0.37.0 release
- ([#622](https://github.com/cosmos/ibc-rs/issues/622))
- Remove `IbcEventType` ([#623](https://github.com/cosmos/ibc-rs/issues/623))
- Split `MsgUpdateClient` back into `MsgUpdateClient` and `MsgSubmitMisbehaviour`
  ([#628](https://github.com/cosmos/ibc-rs/issues/628))
- Refactor and fix version validation in connection and channel handshakes
  ([#625](https://github.com/cosmos/ibc-rs/issues/625))

### IMPROVEMENTS

- Make token transfer events compatible with latest ibc-go
  ([#495](https://github.com/cosmos/ibc-rs/pull/495))

## v0.37.0

*April 13, 2023*

This release primarily updates `ClientState` to adopt a better API for client updates and misbehaviour detection, borrowed from ibc-go's ADR 6. In the process of updating the API, a few bugs were found in the tendermint light client and fixed.

There are consensus-breaking changes.

### BREAKING CHANGES

- `ClientState`: Split `check_misbehaviour_and_update_state` 
  and `check_header_and_update_state`
  ([#535](https://github.com/cosmos/ibc-rs/issues/535))
- Improve MsgTransfer struct
  ([#567](https://github.com/cosmos/ibc-rs/issues/567))
- Remove `ics05_port::context::PortReader` ([#580](https://github.com/cosmos/ibc-rs/issues/580))
- Check if `ClientStatePath` is empty during client creation process
  ([#604](https://github.com/cosmos/ibc-rs/issues/604))

### BUG FIXES

- Disallow creation of new Tendermint client state instance with a frozen height
 ([#178](https://github.com/cosmos/ibc-rs/issues/178))
- Emit a message event for SendPacket ([#574](https://github.com/cosmos/ibc-rs/issues/574))
- Properly convert from `Any` to `MsgEnvelope` 
  ([#578](https://github.com/cosmos/ibc-rs/issues/578))
- Tendermint light client: fix missing trusted_validator_set 
  hash check
  ([#583](https://github.com/cosmos/ibc-rs/issues/583))
- Tendermint light client: fix missing `Header.height()` 
  vs `Header.trusted_height` check
  ([#585](https://github.com/cosmos/ibc-rs/issues/585))
- Tendermint light client: ensure that we use the correct
  chain ID in commit verification
  ([#589](https://github.com/cosmos/ibc-rs/issues/589))
- tx_msg: Remove panic in `Msg::get_sign_bytes`
  ([#593](https://github.com/cosmos/ibc-rs/issues/593))
- Tendermint light client: add check that ensure that
  the consensus state timestamps are monotonic, otherwise
  freeze the client
  ([#598](https://github.com/cosmos/ibc-rs/issues/598))
- Tendermint light client: fix how the client's latest
  height is updated
  ([#601](https://github.com/cosmos/ibc-rs/issues/601))

### FEATURES

- Prefixed denom parity scale codec enabled
  ([#577](https://github.com/cosmos/ibc-rs/pull/577))
- Add (de)serialization for `ics04_channel::handler::ModuleExtras`
  ([#581](https://github.com/cosmos/ibc-rs/issues/581))

## v0.36.0

*March 27, 2023*

This release adds the emission a `"message"` event for all handlers, which hermes currently
depends on.

There are no consensus-breaking changes.

### BUG

- Emit a message event for each IBC handling
  ([#563](https://github.com/cosmos/ibc-rs/issues/563))

## v0.35.0

*March 22, 2023*

This release fixes a bug in the packet timeout handler.

This is a consensus-breaking change.

### BUG

- Timeout handler returns an error only when both height and timestamp have not reached yet 
  ([#555](https://github.com/cosmos/ibc-rs/issues/555))

## v0.34.0

*March 17, 2023*

This release fixes a bug in the connection handshake.

This is a consensus-breaking change.

### BUG

- Fix client IDs for the proof verifications in `ConnectionOpenTry` and `ConnectionOpenAck` 
([#550](https://github.com/cosmos/ibc-rs/issues/550))

## v0.33.0

*March 16, 2023*

This release primarily updates the `ClientState` trait.

There are no consensus-breaking changes.

### BREAKING CHANGES

- Replace specific verify_functions inside `ics02_client` with generic
  `verify_membership` and `verify_non_membership` interfaces.
  ([#530](https://github.com/cosmos/ibc-rs/issues/530))
- Replace `ClientState::frozen_height()` and `ClientState::is_frozen()`
  with `ClientState::confirm_frozen()`
  ([#545](https://github.com/cosmos/ibc-rs/issues/545))

### IMPROVEMENT

- Fix `ContextError` Display output 
  ([#547](https://github.com/cosmos/ibc-rs/issues/547))

## v0.32.0

*March 9, 2023*

This release primarily removes the `'static` lifetime bound on the `Module` trait,
and adds some methods to the token transfer validation trait.

There are no consensus-breaking changes.

### BREAKING CHANGES

- Move `verify_delay_passed` process and its associated errors under the
  `ics03_connection` section and reduce entanglements with the
  `ValidationContext`.
  ([#404](https://github.com/cosmos/ibc-rs/issues/404))
- Refactor and privatize Packet/Ack commitment computations for improved security
  and modularity.
  ([#470](https://github.com/cosmos/ibc-rs/issues/470))
- Allow for non-'static bound Modules 
  [#490](https://github.com/cosmos/ibc-rs/issues/490))
- Separate the validation from the execution process for `send/mint/burn_coins`
  operations.
  ([#502](https://github.com/cosmos/ibc-rs/issues/502))
- Refactor naming in the Transfer application to align with the repo naming
  conventions.
  ([#506](https://github.com/cosmos/ibc-rs/issues/506))
- Refactor `is_send/receive_enabled` interfaces within the transfer application
  to `can_send/receive_coins` returning `Result<(), TokenTransferError>` type
  for a better failure handler
  ([#508](https://github.com/cosmos/ibc-rs/issues/508))

### IMPROVEMENTS

- Use `<&str>::deserialize` instead of `String::deserialize` to avoid an extra
  allocation ([#496](https://github.com/cosmos/ibc-rs/issues/496))
- In `test_serialization_roundtrip`, check that round-tripped data is equal
  ([#497](https://github.com/cosmos/ibc-rs/issues/497))

## v0.31.0

*February 28, 2023*

This release contains quality of life improvements.

There are no consensus-breaking changes.

### BREAKING CHANGES

- Remove ibc::handler module ([#478](https://github.com/cosmos/ibc-rs/issues/478))
- Discard the `connection-channels` method under `ValidationContext` since it is
  no longer used by the core handlers. 
  ([#479](https://github.com/cosmos/ibc-rs/issues/479))
- Remove Send + Sync supertraits on the Module trait
  ([#480](https://github.com/cosmos/ibc-rs/issues/480))
- Modify `validate_self_client` error type to return `ContextError` instead of
  `ConnectionError` 
  ([#482](https://github.com/cosmos/ibc-rs/issues/482))

### IMPROVEMENTS

- Fix typos ([\#472](https://github.com/cosmos/ibc-rs/issues/472))

## v0.30.0

*February 24, 2023*

This release contains an overhaul of the `send_packet()` and `send_transfer()` architecture.
The main gain is to separate into `send_packet_{validate,execute}()`, and similarly for 
`send_transfer()`.

There are no consensus-breaking changes.

### BREAKING CHANGES

- Update send_packet/transfer(), and related contexts
  ([#442](https://github.com/cosmos/ibc-rs/issues/442))

## v0.29.0

*February 22, 2023*

This release includes the latest Tendermint-rs v0.29.0 and removes the
`Reader` and `Keeper` API in favor of the new `ValidationContext`/`ExecutionContext` API as the default.
Additionally, unit tests have been updated to work with the new API.

There are consensus-breaking changes.

### BREAKING CHANGES

- Remove Reader and Keeper API
  ([#279](https://github.com/cosmos/ibc-rs/issues/279))
- Refactor `get_*` and `store_*` methods to take `*Path` structs instead
  ([#382](https://github.com/cosmos/ibc-rs/issues/382))
- Make `ValidationContext::host_timestamp()` abstract and remove
  `ValidationContext::pending_host_consensus_state()`
  ([#418](https://github.com/cosmos/ibc-rs/issues/418))

### BUG FIXES

- Mend error variant todo!()s wherever tendermint client calls the
  "consensus_state" method
  ([#403](https://github.com/cosmos/ibc-rs/issues/403))

### FEATURE

- Remove `val_exec_ctx` feature flag
  ([#415](https://github.com/cosmos/ibc-rs/issues/415))

### IMPROVEMENTS

- Make all unit tests test the ValidationContext/ExecutionContext API
  ([#430](https://github.com/cosmos/ibc-rs/issues/430))
- Add an implementation of `validate_self_client` for the mock client
  ([#432](https://github.com/cosmos/ibc-rs/issues/432))
- Add a docstring and rename the `validate_self_client` argument for improved
  code documentation and readability
  ([#434](https://github.com/cosmos/ibc-rs/issues/434))
- Refactor connection handler unit tests to adapt with new Validation/Execution API
  ([#440](https://github.com/cosmos/ibc-rs/issues/440))

## v0.28.0

*February 9, 2023*

With this release, the implementation of the new `ValidationContext`/`ExecutionContext` is complete, although still behind the `val_exec_ctx` feature flag. There were also important bug fixes.

There are consensus-breaking changes.

### BREAKING CHANGES

- Implement `verify_upgrade_and_update_state` method for Tendermint clients
  ([#19](https://github.com/cosmos/ibc-rs/issues/19)).
- Remove support for asynchronous acknowledgements
  ([#361](https://github.com/cosmos/ibc-rs/issues/361))

### BUG FIXES

- Fix acknowledgement returned by the token transfer's onRecvPacket callback
  ([#369](https://github.com/cosmos/ibc-rs/issues/369))
- Mend `ChanOpenConfirm` handler check of expected counterparty state
  ([#396](https://github.com/cosmos/ibc-rs/issues/396))
- Fix issue with the error handling in the `new_check_header_and_update_state`
  method when consensus state is not found
  ([#405](https://github.com/cosmos/ibc-rs/issues/405))
- Fix the caught error by `get_packet_receipt` under `val_exec_ctx` feature when
  the packet receipt is not found
  ([#409](https://github.com/cosmos/ibc-rs/issues/409))

### FEATURE

- Finish implementing `ValidationContext::validate()` and
  `ExecutionContext::execute()` 
  ([#393](https://github.com/cosmos/ibc-rs/issues/393))

### IMPROVEMENTS

- Add tests to verify `AbciEvent` match the expected Ibc events
([#163](https://github.com/cosmos/ibc-rs/issues/163)).
- Add unit tests to cover edge scenarios for counterparty conn & chan ids at init phases
  ([#175](https://github.com/cosmos/ibc-rs/issues/175)).

## v0.27.0

*January 16, 2023*

This release contains a bug fix for the `ChanOpenConfirm` handler and it is strongly recommended to upgrade.

This release contains a consensus-breaking change during the channel opening handshake; it was broken, and now is fixed.

### BUG FIXES

- Fix ChanOpenConfirm handler check of counterparty state
  ([#353](https://github.com/cosmos/ibc-rs/issues/353))

## v0.26.0

*December 14, 2022*

This release contains miscellaneous improvements, focusing mainly on addressing technical debt.

There are no consensus-breaking changes.

### BREAKING CHANGES

- Exclude `ChannelEnd` from `MsgChannelOpenInit` and `MsgChannelOpenTry` and refactor their fields to match the spec
  ([#20](https://github.com/cosmos/ibc-rs/issues/20))
- Simplify Msg trait by removing unnecessary methods.
  ([#218](https://github.com/cosmos/ibc-rs/issues/218))
- Refactor proof handlers to conduct proof verifications inline with the process function 
  and apply naming conventions to packet messages types
  ([#230](https://github.com/cosmos/ibc-rs/issues/230))
- The function parameters in the Reader traits now references,
  while the functions in the Keeper traits take ownership directly.
  ([#304](https://github.com/cosmos/ibc-rs/issues/304))
- Change type of `trusted_validator_set` field in
  `MisbehaviourTrustedValidatorHashMismatch` error variant from `ValidatorSet` to
  `Vec<Validator>` to avoid clippy catches
  ([#309](https://github.com/cosmos/ibc-rs/issues/309))
- The function parameters in the `ValidationContext` trait now use references,
  while the functions in the `ExecutionContext` trait take ownership directly.
  ([#319](https://github.com/cosmos/ibc-rs/issues/319))
- Make internal `process()` `pub(crate)` 
  ([#338](https://github.com/cosmos/ibc-rs/issues/338))

### FEATURES

- Add serialization and deserialization features for codec and borsh to the host
  type in ics24 ([#259](https://github.com/cosmos/ibc-rs/issues/259))
- Add codec and borsh for ics04_channel::msgs::Acknowledgement and
  events::ModuleEvent ([#303](https://github.com/cosmos/ibc-rs/issues/303))
- Add parity-scale-codec and borsh for ibc::events::IbcEvent
  ([#320](https://github.com/cosmos/ibc-rs/issues/320))
- Make the code under mocks features work in no-std
  ([#311](https://github.com/cosmos/ibc-rs/issues/311))
- Make `serde` optional behind the `serde` feature flag
  ([#293](https://github.com/cosmos/ibc-rs/issues/293))

### IMPROVEMENTS

- Refactor unreachable test of conn_open_ack handler
  ([#30](https://github.com/cosmos/ibc-rs/issues/30))
- Remove legacy relayer-specific code and move ics18_relayer under the mock module
  ([#154](https://github.com/cosmos/ibc-rs/issues/154))
- Improve clippy catches and fix lint issues identified by clippy 0.1.67
  ([#309](https://github.com/cosmos/ibc-rs/issues/309))

## v0.25.0

*December 14, 2022*

This release updates the tendermint-rs dependency to v0.28.0 which includes important security improvements. Many other improvements have been made as well, including misbehaviour handling.

A lot of work has also been put towards implementing ADR 5, which is currently unfinished and has been put behind the feature flag `val_exec_ctx`.

The only consensus-breaking changes are the ones related to the fact that we now properly handle misbehaviour messages.

### BREAKING CHANGES

- Implement the IBC misbehaviour handler and misbehaviour handling logic for the Tendermint light client.
  ([#12](https://github.com/cosmos/ibc-rs/issues/12))
- `Ics20Context` no longer requires types implementing it to implement `ChannelReader` and `ChannelKeeper`, and instead depends on the `Ics20ChannelKeeper`
  and `SendPacketReader` traits. Additionally, the `send_packet` handler now requires the calling context to implement `SendPacketReader` and returns
  a `SendPacketResult`.
  ([#182](https://github.com/cosmos/ibc-rs/issues/182))
- Add `ValidationContext` and `ExecutionContext`, and implement for clients (ICS-2)
  ([#240](https://github.com/cosmos/ibc-rs/issues/240))
- Change `host_height`, `host_timestamp` return value to a `Result` in `ClientReader`, `ConnectionReader`, `ChannelReader` and `ValidationContext`
  ([#242](https://github.com/cosmos/ibc-rs/issues/242))
- Rename Ics* names to something more descriptive
  ([#245](https://github.com/cosmos/ibc-rs/issues/245))
- Implement `ValidationContext::validate` and `ExecutionContext::execute` for connections (ICS-3)
  ([#251](https://github.com/cosmos/ibc-rs/issues/251))
- Implement misbehaviour in `ExecutionContext` and `ValidationContext`
  ([#281](https://github.com/cosmos/ibc-rs/issues/281))
- Update `tendermint` dependencies to `v0.28.0`, which contain an important security fix.
([#294](https://github.com/cosmos/ibc-rs/issues/294))

### BUG FIXES

- Set counterparty connection ID to None in `conn_open_init` ([#174](https://github.com/cosmos/ibc-rs/issues/174))
- Verify the message's counterparty connection ID in `conn_open_ack`
  instead of the store's ([#274](https://github.com/cosmos/ibc-rs/issues/274))

### IMPROVEMENTS

- Remove `flex-error` and remove unused error variants([#164](https://github.com/cosmos/ibc-rs/issues/164))
- ConnectionMsg::ConnectionOpen{Try, Ack} should not wrap a Box
  ([#258](https://github.com/cosmos/ibc-rs/issues/258))
- Track code coverage with `cargo-llvm-cov`
  ([#277](https://github.com/cosmos/ibc-rs/issues/277))

## v0.24.0

*December 8, 2022*

This release mainly updates the tendermint-rs dependency to v0.27.0 and includes security improvements.

There are no consensus-breaking changes.

### BREAKING CHANGES

- Update to changes in tendermint-rs 0.27
  ([#260](https://github.com/cosmos/ibc-rs/pulls/260))

### IMPROVEMENTS

- Update `ics23` to v0.9.0, which contains substantial security improvements
  ([#278](https://github.com/cosmos/ibc-rs/issues/278))

## v0.23.0

*November 21, 2022*

This release mainly updates the tendermint-rs dependency to v0.26.0.

There are no consensus-breaking changes.

### BREAKING CHANGES

- Update to tendermint-rs 0.26 and ibc-proto 0.22
  ([#208](https://github.com/cosmos/ibc-rs/issues/208))

### FEATURES

- Add Other Item for Ics02Client,Ics03connection, Ics04Channel Error
  ([#237](https://github.com/cosmos/ibc-rs/issues/237))

## v0.22.0

*November 9, 2022*

This release includes major improvements in making the library compatible with ibc-go v5.0.1. This includes making ibc events compatible and removing the crossing-hellos logic from the connection and channel handshakes.

There are consensus-breaking changes in the connection and channel handshakes. However, there are no consensus-breaking changes for already established channels.

### BREAKING CHANGES

- Make connection events compatible with ibc-go
  ([#145](https://github.com/cosmos/ibc-rs/issues/145))
- Makes channel/packet events compatible with ibc-go
  ([#146](https://github.com/cosmos/ibc-rs/issues/146))
- Remove crossing hellos logic from connection handshake. Breaking changes in 
  connection message types.
  ([#156](https://github.com/cosmos/ibc-rs/issues/156)).
- Remove crossing hellos logic from channel handshake
  ([#157](https://github.com/cosmos/ibc-rs/issues/157))
- call `validate_self_client` in `conn_open_try` and `conn_open_ack`,
  and provide a tendermint implementation for `validate_self_client`
  ([#162](https://github.com/cosmos/ibc-rs/issues/162))
- Refactor channel handlers. Proof calls were inlined, and our handshake
  variable naming convention was applied
  ([#166](https://github.com/cosmos/ibc-rs/issues/166))
- Change `ClientType` to contain a `String` instead of `&'static str`
  ([#206](https://github.com/cosmos/ibc-rs/issues/206))

### BUG FIXES

- Connection consensus state proof verification now properly uses `consensus_height`
  ([#168](https://github.com/cosmos/ibc-rs/issues/168)).
- Allow one-letter chain names in `ChainId::is_epoch_format`
  ([#211](https://github.com/cosmos/ibc-rs/issues/211))
- Don't panic on user input in channel proof verification
  ([#219](https://github.com/cosmos/ibc-rs/issues/219))

### FEATURES

- Add getter functions to SendPacket, ReceivePacket, WriteAcknowledgement,
  AcknowledgePacket, TimeoutPacket to get the elements of the structure
  ([#231](https://github.com/cosmos/ibc-rs/issues/231))

## v0.21.1

*October 27, 2022*

This release fixes a critical vulnerability. It is strongly advised to upgrade.

### BUG FIXES

- No longer panic when packet data is not valid UTF-8
  ([#199](https://github.com/cosmos/ibc-rs/issues/199))

## v0.21.0

*October 24, 2022*

This is a small release that allows new `ClientTypes` to be created, which was missed when implementing ADR 4. The changes are not consensus-breaking.

### BREAKING CHANGES

- Make ClientType allow any string value as opposed to just Tendermint
  ([#188](https://github.com/cosmos/ibc-rs/issues/188))

## v0.20.0

*October 19, 2022*

This is a major release, which implemented [ADR 4](https://github.com/cosmos/ibc-rs/blob/main/docs/architecture/adr-004-light-client-crates-extraction.md), as well as some miscellaneous bug fixes. Please see the corresponding sections for more information.

### BREAKING CHANGES

- Add missing Tendermint `ClientState` checks and make all its fields private.
- Add a `frozen_height` input parameter to `ClientState::new()`.
  ([#22](https://github.com/cosmos/ibc-rs/issues/22)).
- Remove `Display` from `IbcEvent` ([#144](https://github.com/cosmos/ibc-rs/issues/144)).
- Remove `IbcEvent::Empty` ([#144](https://github.com/cosmos/ibc-rs/issues/144)).
- Make `client_state` field required in `MsgConnectionOpenTry` and
  `MsgConnectionOpenAck`. Necessary for correctness according to spec.  
  ([#159](https://github.com/cosmos/ibc-rs/issues/159)).
- Redesign the API to allow light client implementations to be hosted outside the ibc-rs repository. 
  ([#2483](https://github.com/informalsystems/ibc-rs/pull/2483)).

### BUG FIXES

- Make client events compatible with ibc-go v5
  ([#144](https://github.com/cosmos/ibc-rs/issues/144)).
- Delete packet commitment in acknowledge packet handler regardless of channel ordering
  ([#2229](https://github.com/informalsystems/ibc-rs/issues/2229)).

### FEATURES

- Public PrefixedDenom inner type and add as_str func for BaseDenom 
  ([#161](https://github.com/cosmos/ibc-rs/issues/161))

### IMPROVEMENTS

- Derive Hash for ModuleId ([#179](https://github.com/cosmos/ibc-rs/issues/179))
- Improved `core::ics04_channel` APIs, avoiding poor ergonomics of
  reference-to-tuple arguments and inconsistent ownership patterns.
  ([#2603](https://github.com/informalsystems/ibc-rs/pull/2603)).

### DESIGN DECISIONS
- Propose ADR05 for handlers validation and execution separation.
  ([#2582](https://github.com/informalsystems/ibc-rs/pull/2582)).

## v0.19.0

*August 22nd, 2022*

#### BREAKING CHANGES

- Remove `height` attribute from `IbcEvent` and its variants
  ([#2542](https://github.com/informalsystems/ibc-rs/issues/2542))

#### BUG FIXES

- Fix `MsgTimeoutOnClose` to verify the channel proof
  ([#2534](https://github.com/informalsystems/ibc-rs/issues/2534))


## v0.18.0

*August 8th, 2022*

#### IMPROVEMENTS

- Remove Deserialize from IbcEvent and variants
  ([#2481](https://github.com/informalsystems/ibc-rs/issues/2481))


## v0.17.0

*July 27th, 2022*

#### BREAKING CHANGES

- Remove provided `Ics20Reader::get_channel_escrow_address()` implementation and make `cosmos_adr028_escrow_address()` public.
  ([#2387](https://github.com/informalsystems/ibc-rs/issues/2387))

#### BUG FIXES

- Fix serialization for ICS20 packet data structures
  ([#2386](https://github.com/informalsystems/ibc-rs/issues/2386))
- Properly process `WriteAcknowledgement`s on packet callback
  ([#2424](https://github.com/informalsystems/ibc-rs/issues/2424))
- Fix `write_acknowledgement` handler which incorrectly used packet's `source_{port, channel}` as key for storing acks
  ([#2428](https://github.com/informalsystems/ibc-rs/issues/2428))

#### IMPROVEMENTS

- Propose ADR011 for light client extraction
  ([#2356](https://github.com/informalsystems/ibc-rs/pull/2356))


## v0.16.0

*July 7th, 2022*

#### BREAKING CHANGES

- Change `ChannelId` representation to a string, allowing all IDs valid per ICS 024
  ([#2330](https://github.com/informalsystems/ibc-rs/issues/2330)).

#### BUG FIXES

- Fix `recv_packet` handler incorrectly querying `packet_receipt` and `next_sequence_recv` using
  packet's `source_{port, channel}`.
  ([#2293](https://github.com/informalsystems/ibc-rs/issues/2293))
- Permit channel identifiers with length up to 64 characters,
  as per the ICS 024 specification.
  ([#2330](https://github.com/informalsystems/ibc-rs/issues/2330)).

#### IMPROVEMENTS

- Remove the concept of a zero Height
  ([#1009](https://github.com/informalsystems/ibc-rs/issues/1009))
- Complete ICS20 implementation ([#1759](https://github.com/informalsystems/ibc-rs/issues/1759))
- Derive `serde::{Serialize, Deserialize}` for `U256`. ([#2279](https://github.com/informalsystems/ibc-rs/issues/2279))
- Remove unnecessary supertraits requirements from ICS20 traits.
  ([#2280](https://github.com/informalsystems/ibc-rs/pull/2280))


## v0.15.0

*May 23rd, 2022*

### BUG FIXES

- Fix packet commitment calculation to match ibc-go
  ([#2104](https://github.com/informalsystems/ibc-rs/issues/2104))
- Fix incorrect acknowledgement verification
  ([#2114](https://github.com/informalsystems/ibc-rs/issues/2114))
- fix connection id mix-up in connection acknowledgement processing
  ([#2178](https://github.com/informalsystems/ibc-rs/issues/2178))

### IMPROVEMENTS

- Remove object capabilities from the modules
  ([#2159](https://github.com/informalsystems/ibc-rs/issues/2159))


## v0.14.1

*May 2nd, 2022*

> This is a legacy version with no ibc crate changes. 

## v0.14.0

*April 27th, 2022*

### BUG FIXES

- Make all handlers emit an IbcEvent with current host chain height as height parameter value.
  ([#2035](https://github.com/informalsystems/ibc-rs/issues/2035))
- Use the version in the message when handling a MsgConnOpenInit
  ([#2062](https://github.com/informalsystems/ibc-rs/issues/2062))

### IMPROVEMENTS

- Complete ICS26 implementation ([#1758](https://github.com/informalsystems/ibc-rs/issues/1758))
- Improve `ChannelId` validation. ([#2068](https://github.com/informalsystems/ibc-rs/issues/2068))


## v0.13.0
*March 28th, 2022*

### IMPROVEMENTS

- Refactored channels events in ICS 04 module
  ([#718](https://github.com/informalsystems/ibc-rs/issues/718))


## v0.12.0
*February 24th, 2022*

### BUG FIXES

- Fixed the formatting of NotEnoughTimeElapsed and NotEnoughBlocksElapsed
  in Tendermint errors ([#1706](https://github.com/informalsystems/ibc-rs/issues/1706))
- IBC handlers now retrieve the host timestamp from the latest host consensus
  state ([#1770](https://github.com/informalsystems/ibc-rs/issues/1770))

### IMPROVEMENTS

- Added more unit tests to verify Tendermint ClientState
  ([#1706](https://github.com/informalsystems/ibc-rs/issues/1706))
- Define CapabilityReader and CapabilityKeeper traits
  ([#1769](https://github.com/informalsystems/ibc-rs/issues/1769))
- [Relayer Library](https://github.com/informalsystems/hermes/tree/master/crates/relayer)
  - Add two more health checks: tx indexing enabled and historical entries > 0
    ([#1388](https://github.com/informalsystems/ibc-rs/issues/1388))
  - Changed `ConnectionEnd::versions` method to be non-allocating by having it return a `&[Version]` instead of `Vec<Version>`
    ([#1880](https://github.com/informalsystems/ibc-rs/pull/1880))

## v0.11.1
*February 4th, 2022*

> This is a legacy version with no ibc crate changes.


## v0.11.0
*January 27th, 2022*

### BREAKING CHANGES

- Hide `ibc::Timestamp::now()` behind `clock` feature flag ([#1612](https://github.com/informalsystems/ibc-rs/issues/1612))

### BUG FIXES

- Verify the client consensus proof against the client's consensus state root and not the host's state root
  [#1745](https://github.com/informalsystems/ibc-rs/issues/1745)
- Initialize consensus metadata on client creation
  ([#1763](https://github.com/informalsystems/ibc-rs/issues/1763))

### IMPROVEMENTS

  - Extract all `ics24_host::Path` variants into their separate types
    ([#1760](https://github.com/informalsystems/ibc-rs/issues/1760))
  - Disallow empty `CommitmentPrefix` and `CommitmentProofBytes`
    ([#1761](https://github.com/informalsystems/ibc-rs/issues/1761))

## v0.10.0
*January 13th, 2021*

### BREAKING CHANGES

- Add the `frozen_height()` method to the `ClientState` trait (includes breaking changes to the Tendermint `ClientState` API).
  ([#1618](https://github.com/informalsystems/ibc-rs/issues/1618))
- Remove `Timestamp` API that depended on the `chrono` crate:
  ([#1665](https://github.com/informalsystems/ibc-rs/pull/1665)):
  - `Timestamp::from_datetime`; use `From<tendermint::Time>`
  - `Timestamp::as_datetime`, superseded by `Timestamp::into_datetime`

### BUG FIXES

- Delete packet commitment instead of acknowledgement in acknowledgePacket
  [#1573](https://github.com/informalsystems/ibc-rs/issues/1573)
- Set the `counterparty_channel_id` correctly to fix ICS04 [`chanOpenAck` handler verification](https://github.com/informalsystems/ibc-rs/blob/master/modules/src/core/ics04_channel/handler/chan_open_ack.rs)
  ([#1649](https://github.com/informalsystems/ibc-rs/issues/1649))
- Add missing assertion for non-zero trust-level in Tendermint client initialization.
  ([#1697](https://github.com/informalsystems/ibc-rs/issues/1697))
- Fix conversion to Protocol Buffers of `ClientState`'s `frozen_height` field.
  ([#1710](https://github.com/informalsystems/ibc-rs/issues/1710))

### FEATURES

- Implement proof verification for Tendermint client (ICS07).
  ([#1583](https://github.com/informalsystems/ibc-rs/pull/1583))

### IMPROVEMENTS

- More conventional ad-hoc conversion methods on `Timestamp`
  ([#1665](https://github.com/informalsystems/ibc-rs/pull/1665)):
- `Timestamp::nanoseconds` replaces `Timestamp::as_nanoseconds`
- `Timestamp::into_datetime` substitutes `Timestamp::as_datetime`

## v0.9.0, the “Zamfir” release
*November 23rd, 2021*

### BUG FIXES

- Set the connection counterparty in the ICS 003 [`connOpenAck` handler][conn-open-ack-handler]
  ([#1532](https://github.com/informalsystems/ibc-rs/issues/1532))

[conn-open-ack-handler]: https://github.com/informalsystems/ibc-rs/blob/master/modules/src/core/ics03_connection/handler/conn_open_ack.rs

### IMPROVEMENTS

- Derive `PartialEq` and `Eq` on `IbcEvent` and inner types
  ([#1546](https://github.com/informalsystems/ibc-rs/issues/1546))


## v0.8.0
*October 29th, 2021*

### IMPROVEMENTS

- Support for converting `ibc::events::IbcEvent` into `tendermint::abci::Event`
  ([#838](https://github.com/informalsystems/ibc-rs/issues/838))
- Restructure the layout of the `ibc` crate to match `ibc-go`'s [layout](https://github.com/cosmos/ibc-go#contents)
  ([#1436](https://github.com/informalsystems/ibc-rs/issues/1436))
- Implement `FromStr<Path>` to enable string-encoded paths to be converted into Path identifiers
  ([#1460](https://github.com/informalsystems/ibc-rs/issues/1460))


## v0.8.0-pre.1
*October 22nd, 2021*

### BREAKING CHANGES

- The `check_header_and_update_state` method of the `ClientDef`
  trait (ICS02) has been expanded to facilitate ICS07
  ([#1214](https://github.com/informalsystems/ibc-rs/issues/1214))

### FEATURES

- Add ICS07 verification functionality by using `tendermint-light-client`
  ([#1214](https://github.com/informalsystems/ibc-rs/issues/1214))


## v0.7.3
*October 4th, 2021*

> This is a legacy version with no ibc crate changes.


## v0.7.2
*September 24th, 2021*


## v0.7.1
*September 14th, 2021*

### IMPROVEMENTS

- Change all `*Reader` traits to return `Result` instead of `Option` ([#1268])
- Clean up modules' errors ([#1333])

[#1268]: https://github.com/informalsystems/ibc-rs/issues/1268
[#1333]: https://github.com/informalsystems/ibc-rs/issues/1333


## v0.7.0
*August 24th, 2021*

### BUG FIXES

- Set the index of `ibc::ics05_port::capabilities::Capability` ([#1257])

[#1257]: https://github.com/informalsystems/ibc-rs/issues/1257

### IMPROVEMENTS

- Implement `ics02_client::client_consensus::ConsensusState` for `AnyConsensusState` ([#1297])

[#1297]: https://github.com/informalsystems/ibc-rs/issues/1297


## v0.6.2
*August 2nd, 2021*

### BUG FIXES

- Add missing `Protobuf` impl for `ics03_connection::connection::Counterparty` ([#1247])

[#1247]: https://github.com/informalsystems/ibc-rs/issues/1247

### FEATURES

- Use the [`flex-error`](https://docs.rs/flex-error/) crate to define and
handle errors ([#1158])


## v0.6.1
*July 22nd, 2021*

### FEATURES

- Enable `pub` access to verification methods of ICS 03 & 04 ([#1198])
- Add `ics26_routing::handler::decode` function ([#1194])
- Add a pseudo root to `MockConsensusState` ([#1215])

### BUG FIXES

- Fix stack overflow in `MockHeader` implementation ([#1192])
- Align `as_str` and `from_str` behavior in `ClientType` ([#1192])

[#1192]: https://github.com/informalsystems/ibc-rs/issues/1192
[#1194]: https://github.com/informalsystems/ibc-rs/issues/1194
[#1198]: https://github.com/informalsystems/ibc-rs/issues/1198
[#1215]: https://github.com/informalsystems/ibc-rs/issues/1215


## v0.6.0
*July 12th, 2021*

> This is a legacy version with no ibc crate changes.


## v0.5.0
*June 22nd, 2021*

> This is a legacy version with no ibc crate changes.


## v0.4.0
*June 3rd, 2021*

### IMPROVEMENTS

- Started `unwrap` cleanup ([#871])

[#871]: https://github.com/informalsystems/ibc-rs/issues/871


## v0.3.2
*May 21st, 2021*

> This is a legacy version with no ibc crate changes.


## v0.3.1
*May 14h, 2021*

### BUG FIXES

- Process raw `delay_period` field as nanoseconds instead of seconds. ([#927])

[#927]: https://github.com/informalsystems/ibc-rs/issues/927


## v0.3.0
*May 7h, 2021*

### IMPROVEMENTS

- Reinstated `ics23` dependency ([#854])
- Use proper Timestamp type to track time ([#758])

### BUG FIXES

- Fix parsing in `chain_version` when chain identifier has multiple dashes ([#878])

[#758]: https://github.com/informalsystems/ibc-rs/issues/758
[#854]: https://github.com/informalsystems/ibc-rs/issues/854
[#878]: https://github.com/informalsystems/ibc-rs/issues/878


## v0.2.0
*April 14th, 2021*

### FEATURES

- Added handler(s) for sending packets ([#695]), recv. and ack. packets ([#736]), and timeouts ([#362])

### IMPROVEMENTS

- Follow Rust guidelines naming conventions ([#689])
- Per client structure modules ([#740])
- MBT: use modelator crate ([#761])

### BUG FIXES

- Fix overflow bug in ICS03 client consensus height verification method ([#685])
- Allow a conn open ack to succeed in the happy case ([#699])

### BREAKING CHANGES

- `MsgConnectionOpenAck.counterparty_connection_id` is now a `ConnectionId` instead of an `Option<ConnectionId>`([#700])

[#362]: https://github.com/informalsystems/ibc-rs/issues/362
[#685]: https://github.com/informalsystems/ibc-rs/issues/685
[#689]: https://github.com/informalsystems/ibc-rs/issues/689
[#699]: https://github.com/informalsystems/ibc-rs/issues/699
[#700]: https://github.com/informalsystems/ibc-rs/pull/700
[#736]: https://github.com/informalsystems/ibc-rs/issues/736
[#740]: https://github.com/informalsystems/ibc-rs/issues/740
[#761]: https://github.com/informalsystems/ibc-rs/issues/761


## v0.1.1
*February 17, 2021*

### IMPROVEMENTS

- Change event height to ICS height ([#549])

### BUG FIXES

- Fix panic in conn open try when no connection id is provided ([#626])
- Disable MBT tests if the "mocks" feature is not enabled ([#643])

### BREAKING CHANGES

- Implementation of the `ChanOpenAck`, `ChanOpenConfirm`, `ChanCloseInit`, and `ChanCloseConfirm` handlers ([#316])
- Remove dependency on `tendermint-rpc` ([#624])

[#316]: https://github.com/informalsystems/ibc-rs/issues/316
[#549]: https://github.com/informalsystems/ibc-rs/issues/549
[#624]: https://github.com/informalsystems/ibc-rs/issues/624
[#626]: https://github.com/informalsystems/ibc-rs/issues/626
[#643]: https://github.com/informalsystems/ibc-rs/issues/643


## v0.1.0
*February 4, 2021*

### FEATURES

- Add `MsgTimeoutOnClose` message type ([#563])
- Implement `MsgChannelOpenTry` message handler ([#543])

### IMPROVEMENTS

- Clean the `validate_basic` method ([#94])
- `MsgConnectionOpenAck` testing improvements ([#306])

### BUG FIXES:

- Fix for storing `ClientType` upon 'create-client' ([#513])

### BREAKING CHANGES:

- The `ibc::handler::Event` is removed and handlers now produce `ibc::events::IBCEvent`s ([#535])

[#94]: https://github.com/informalsystems/ibc-rs/issues/94
[#306]: https://github.com/informalsystems/ibc-rs/issues/306
[#513]: https://github.com/informalsystems/ibc-rs/issues/513
[#535]: https://github.com/informalsystems/ibc-rs/issues/535
[#543]: https://github.com/informalsystems/ibc-rs/issues/543
[#563]: https://github.com/informalsystems/ibc-rs/issues/563


## v0.0.6
*December 23, 2020*

> This is a legacy version with no ibc crate changes.


## v0.0.5
*December 2, 2020*

### FEATURES

- Implement flexible connection id selection ([#332])
- ICS 4 Domain Types for channel handshakes and packets ([#315], [#95])
- Introduce LightBlock support for MockContext ([#389])


### IMPROVEMENTS

- Split `msgs.rs` of ICS002 in separate modules ([#367])
- Fixed inconsistent versioning for ICS003 and ICS004 ([#97])
- Fixed `get_sign_bytes` method for messages ([#98])
- Homogenize ConnectionReader trait so that all functions return owned objects ([#347])
- Align with tendermint-rs in the domain type definition of `block::Id` ([#338])


[#95]: https://github.com/informalsystems/ibc-rs/issues/95
[#97]: https://github.com/informalsystems/ibc-rs/issues/97
[#98]: https://github.com/informalsystems/ibc-rs/issues/98
[#332]: https://github.com/informalsystems/ibc-rs/issues/332
[#338]: https://github.com/informalsystems/ibc-rs/issues/338
[#347]: https://github.com/informalsystems/ibc-rs/issues/347
[#367]: https://github.com/informalsystems/ibc-rs/issues/367
[#368]: https://github.com/informalsystems/ibc-rs/issues/368
[#389]: https://github.com/informalsystems/ibc-rs/issues/389


## v0.0.4
*October 19, 2020*

### FEATURES:
- ICS03 Ack and Confirm message processors ([#223])
- Routing module minimal implementation for MVP ([#159], [#232])
- Basic relayer functionality: a test with ClientUpdate ping-pong between two mocked chains ([#276])

### IMPROVEMENTS:
- Implemented the `DomainType` trait for IBC proto structures ([#245], [#249]).
- ICS03 connection handshake protocol initial implementation and tests ([#160])
- Add capability to decode from protobuf Any* type into Tendermint and Mock client states
- Cleanup Any* client wrappers related code
- Migrate handlers to newer protobuf definitions ([#226])
- Extend client context mock ([#221])
- Context mock simplifications and cleanup ([#269], [#295], [#296], [#297])
- Split `msgs.rs` in multiple files, implement `From` for all messages ([#253])

### BUG FIXES:
- Removed "Uninitialized" state from connection ([#217])
- Disclosed bugs in ICS3 version negotiation and proposed a fix ([#209], [#213])

[#159]: https://github.com/informalsystems/ibc-rs/issues/159
[#160]: https://github.com/informalsystems/ibc-rs/issues/160
[#207]: https://github.com/informalsystems/ibc-rs/issues/207
[#209]: https://github.com/informalsystems/ibc-rs/issues/209
[#213]: https://github.com/informalsystems/ibc-rs/issues/213
[#217]: https://github.com/informalsystems/ibc-rs/issues/217
[#221]: https://github.com/informalsystems/ibc-rs/issues/221
[#223]: https://github.com/informalsystems/ibc-rs/issues/223
[#226]: https://github.com/informalsystems/ibc-rs/issues/226
[#232]: https://github.com/informalsystems/ibc-rs/issues/232
[#245]: https://github.com/informalsystems/ibc-rs/issues/245
[#249]: https://github.com/informalsystems/ibc-rs/issues/249
[#269]: https://github.com/informalsystems/ibc-rs/issues/269
[#276]: https://github.com/informalsystems/ibc-rs/issues/276
[#295]: https://github.com/informalsystems/ibc-rs/issues/295
[#296]: https://github.com/informalsystems/ibc-rs/issues/296
[#297]: https://github.com/informalsystems/ibc-rs/issues/297


## v0.0.3
*September 1, 2020*

### BREAKING CHANGES:
- Renamed `modules` crate to `ibc` crate. Version number for the new crate is not reset. ([#198])
- `ConnectionId`s are now decoded to `Vec<ConnectionId>` and validated instead of `Vec<String>` ([#185])
- Removed `Connection` and `ConnectionCounterparty` traits ([#193])
- Removed `Channel` and `ChannelCounterparty` traits ([#192])

### FEATURES:
- partial implementation of message handler ([#119], [#194])
- partial implementation of message handler ([#119], [#194])
- Proposal for IBC handler (message processor) architecture ([#119], [#194])
- Documentation for the repository structure ([#1])
- Connection Handshake FSM English description ([#122])

### BUG FIXES:
- Identifiers limit update according to ICS specs ([#168])

[#1]: https://github.com/informalsystems/ibc-rs/issues/1
[#119]: https://github.com/informalsystems/ibc-rs/issues/119
[#122]: https://github.com/informalsystems/ibc-rs/issues/122
[#168]: https://github.com/informalsystems/ibc-rs/issues/168
[#185]: https://github.com/informalsystems/ibc-rs/issues/185
[#192]: https://github.com/informalsystems/ibc-rs/issues/192
[#193]: https://github.com/informalsystems/ibc-rs/issues/193
[#194]: https://github.com/informalsystems/ibc-rs/issues/194
[#198]: https://github.com/informalsystems/ibc-rs/issues/198


## v0.0.2

*August 1, 2020*

### BREAKING CHANGES:

- Refactor queries, paths, and Chain trait to reduce code and use
  protobuf instead of Amino.
        [\#152](https://github.com/informalsystems/ibc-rs/pull/152),
        [\#174](https://github.com/informalsystems/ibc-rs/pull/174),
        [\#155](https://github.com/informalsystems/ibc-rs/pull/155)

### FEATURES:

- Channel closing datagrams in TLA+ [\#141](https://github.com/informalsystems/ibc-rs/pull/141)

### IMPROVEMENTS:

- Implemented better Raw type handling. [\#156](https://github.com/informalsystems/ibc-rs/pull/156)

### BUG FIXES:

- Fixed the identifiers limits according to updated ics spec. [\#189](https://github.com/informalsystems/ibc-rs/pull/189)
- Fix nightly runs. [\#161](https://github.com/informalsystems/ibc-rs/pull/161)
- Fix for incomplete licence terms. [\#153](https://github.com/informalsystems/ibc-rs/pull/153)


## 0.0.1

*July 1st, 2020*

This is the initial prototype release of an IBC relayer and TLA+ specifications.
There are no compatibility guarantees until v0.1.0.

Includes:

- Client state, consensus state, connection, channel queries.
    - Note: deserialization is unimplemented as it has dependency on migration to protobuf for ABCI queries
- IBC Modules partial implementation for datastructures, messages and queries.
- Some English and TLA+ specifications for Connection & Channel Handshake as well as naive relayer algorithm.
