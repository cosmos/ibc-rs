# CHANGELOG

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
- Set the `counterparty_channel_id` correctly to fix ICS04 [`chanOpenAck` handler verification](https://github.com/cosmos/ibc-rs/blob/v0.10.0/modules/src/core/ics04_channel/handler/chan_open_ack.rs)
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

[conn-open-ack-handler]: https://github.com/cosmos/ibc-rs/blob/v0.9.0/modules/src/core/ics03_connection/handler/conn_open_ack.rs

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
