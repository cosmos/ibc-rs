# ADR 11: Refactoring `ibc-rs`'s Error Handling Architecture

## Changelog

- 2024-09-20: Draft Proposed

## Status

Proposed

## Context

As a library whose main use cases involve integrating with external host code
bases, ibc-rs's error handling architecture should strive to achieve the
following goals:

1. Empower host developers to respond to ibc-rs-originating errors that affect
   host logic.
2. Assist ibc-rs core developers' debugging efforts by providing detailed, human-readable
   error reports and/or stack traces.

These two aims highlight the need to clearly distinguish between *host*-level
errors and *protocol*-level errors. Developers working at the host level often
do not need to concern themselves with lower-level protocol errors; even if they
do, those errors should not be exposed to host developers at the fine-grained
level of granularity that is more appropriate for protocol-level developers.

As it currently stands, ibc-rs exposes a top-level `ContextError` type that is
returned by all validation and execution context methods, which are those that
need to be implemented by hosts as part of the ibc-rs integration process. This
`ContextError` type encapsulates all of the protocol-level errors that ibc-rs
exposes. As a result, the onus is being placed on host developers to decide
whether these errors are relevant or not. In other words, the distinction
between host- and protocol-level errors is not reflected in ibc-rs's error
handling methodology.

## Proposal

### Some Changes to `ContextError`

ibc-rs's top-level error type, `ContextError`, will be renamed to `HandlerError`
in an effort to improve the semantics of this error type. `HandlerError` will be
returned only by the top- level handler entrypoints of ibc-rs, namely the
[dispatch], [validate], and [execute] methods.

```diff
- pub enum ContextError {
+ pub enum HandlerError {
    /// ICS02 Client error: {0}
    Client(ClientError),
    /// ICS03 Connection error: {0}
    Connection(ConnectionError),
    /// ICS04 Channel error: {0}
    Channel(ChannelError),
-   /// ICS04 Packet error: {0}
-   Packet(PacketError),
    /// ICS26 Router error: {0}
    Router(RouterError),
}
```

In addition, the error variant that contained the `PacketError` type will be
removed. This is because we're opting to move towards only having a single error
type for each ICS module, removing the discrepancy that the ICS04 module exposes
two distinct error types. The `PacketError` type will be merged into the
`ChannelError` type.

### The New `HostError` Type

In light of the stated rationale of making it clear when an error originates
from host logic vs ibc-rs's internal logic, we propose adding a new `HostError`
type that will live as a variant in each of the module-level error types, as
well as in the application-level error types, `TokenTransferError` and
`NftTransferError`. Initially, we had only a single `Host(HostError)` variant
that existed in the `HandlerError`type, but it became clear that this wasn't the
correct place in which to expose host-level errors. This is because host errors
can crop up within any of the core ibc-rs modules.

We introduce the following concrete `HostError` type in the `ics24-host` crate:

```rust
pub enum HostError {
    /// invalid state: {description}
    InvalidState { description: String },
    /// missing state: {description}
    MissingState { description: String },
    /// failed to update store: {description}
    FailedToStore { description: String },
    /// failed to retrieve from store: {description}
    FailedToRetrieve { description: String },
    /// other error: {description}
    Other { description: String },
}
```

This initial definition offers fairly generic error variants that nonetheless
aim to capture most of the error use-cases faced by hosts. One such notable
use-case is fetching/retrieving data from the host's storage.

Host errors can occur within any of the core ibc-rs modules. Thus, we'll be
adding `HostError` variants to each of the module-level error types where
appropriate: `ClientError` in ICS02, `ConnectionError` in ICS03, and
`ChannelError` in ICS04. Note that as of now a `HostError` variant is not being
added to the `RouterError` type in ICS26 as it is not used by hosts, i.e., it
does not expose its own handlers.

The main areas where `HostError`s are now being returned are mainly in
`ValidationContext` and `ExecutionContext` trait methods. These traits are the
ones that hosts need to implement as part of the process of integrating ibc-rs.

As an example, consider the [consensus_state] method under the
`ClientValidationContext` trait. This method used to return a `ContextError`.
One place where it is called is in the `upgrade_client` handler:

```rust
pub fn validate<Ctx>(ctx: &Ctx, msg: MsgUpgradeClient) -> Result<(), ContextError>
    ...
    let old_consensus_state = client_val_ctx
        .consensus_state(&old_cons_state_path)
        .map_err(|_| ClientError::MissingConsensusState {
            client_id,
            height: old_client_state.latest_height(),
        })?;
```

The `ContextError` of the `consensus_state` method was being mapped onto a
`ClientError`, which was then mapped *back* into a `ContextError`; certainly an
inefficient round-trip. In the case that an error did occur in the
`consensus_state` method, it would not have been clear to the user that the
error originated from a host context.

This `validate` function will now be changed to return a `ClientError`. Coupled
with the `consensus_state` method now returning a `HostError`, this call can now
be made much more cleanly:

```rust
pub fn validate<Ctx>(ctx: &Ctx, msg: MsgUpgradeClient) -> Result<(), ClientError>
    ...
    let old_consensus_state = client_val_ctx
        .consensus_state(&old_cons_state_path)?;
```

`HostError`s cleanly map to `ClientError`s (along with any other of the module
errors in ibc-core). In the same vein, `ClientError`s map cleanly to
`HandlerError`s: with these changes, the granularity of where and how errors are
defined in ibc-rs makes a lot more sense. Plus, the error will now carry the
relevant host context!

### Defining a New `DecodingError` Type

Another significant change that will be made as part of refactoring ibc-rs's
error handling is the introduction of a new `DecodingError` type. The purpose of
this type is to serve as the single error type returned by every raw `TryFrom`
conversion in ibc-rs. A major chunk of ibc-rs's logic deals with these sorts of
conversions. As it stands, depending on what type was being converted into,
different module-level errors were used as the error type.

For example, a `TryFrom<Any> for AnyClientState` impl would return a
`ClientError`, while a `TryFrom<Vec<RawProofSpec>> for ProofSpecs` impl would
return a `CommitmentError`. As a result, there were many conversion-related
error variants scattered across all the different error types, which led to a
lot of duplication and redundancy. In essence, these methods are all dealing
with decoding and/or deserialization in some form or another.

The introduction of the `DecodingError` type seeks to consolidate these
duplications and redundancies:

```rust
pub enum DecodingError {
    /// identifier error: {0}
    Identifier(IdentifierError),
    /// base64 decoding error: {0}
    Base64(Base64Error),
    /// utf-8 String decoding error: {0}
    StringUtf8(FromUtf8Error),
    /// utf-8 str decoding error: {0}
    StrUtf8(Utf8Error),
    /// protobuf decoding error: {0}
    Protobuf(ProtoError),
    /// prost decoding error: {0}
    Prost(ProstError),
    /// invalid hash bytes: {description}
    InvalidHash { description: String },
    /// invalid JSON data: {description}
    InvalidJson { description: String },
    /// invalid raw data: {description}
    InvalidRawData { description: String },
    /// missing raw data: {description}
    MissingRawData { description: String },
    /// mismatched resource name: expected `{expected}`, actual `{actual}`
    MismatchedResourceName { expected: String, actual: String },
    /// unknown type URL `{0}`
    UnknownTypeUrl(String),
}
```

This type captures most of external decoding- and parsing-related errors that
crop up in ibc-rs, as well as variants for encoding internal decoding issues.

Deploying this error type across all of ibc-rs's conversions will have a marked
effect on the number of outstanding variants held by the module-level errors,
allowing us to reduce a significant number of variants and reduce these module
error to something much more akin to its essence.

### Positives

The changes introduced in this ADR represent a clear net positive effect on the
usability and maintainability of ibc-rs as a whole. The overall hierarchy and
semantics of the error system make a lot more sense than its prior incarnation.
Module-level errors are much more simplified and streamlined. Additionally, host
contexts can be attached to ibc-core errors, making it much clearer where an
error originated from.

### Negatives

With all that said, these changes of course come with some downsides. A major
one would be the generality of the error types that were introduced: it's not
clear whether we captured the right level of granularity with them. Especially
with the `HostError` type, it's not clear whether the way this type is laid out
is sufficient for hosts, or whether they would prefer something more bespoke and
tailored to their particular needs.

Most of the new error variants introduced also require String allocations, which
is ideal; this is a tradeoff between generality of error variants and
specificity. Introducing more specific error variants would help cut down on the
number of String allocations, but would contribute to bloating and redundancy
within ibc-rs's error types.

Lastly, the new error types and variants do not come with guard rails to help
steer ibc-rs's contributors towards following the conventions laid out in their
usage. It is very easy to abuse String-allocating variants for errors that they
may not actually be appropriate for. The main guard against this comes in the
form of PR review, which is not ideal.

## Notes on Error Handling Conventions

### Error Classifications

This section details the conventions surrounding how error types, their
variants, and the error messages contained within variants, are formatted and
structured. It serves as a guide for how to parse and read ibc-rs's error
messages.

The naming convention of variants follows a few distinct classifications,
illustrated by the following example code snippet:

```rust
/// The possible classes of errors that error variants can encapsulate
enum ErrorClassifications {
    /// nested error: {0}
    NestedError(SomeError),
    /// something is missing
    Missing,
    /// already exists
    Duplicate,
    /// something is invalid
    Invalid,
    /// mismatched thing: expected `{expected}`, actual `{actual}`
    Mismatched { expected: String, actual: String },
    /// failed to do something
    FailedTo,
    /// unknown resource
    Unknown,
    /// insufficient value
    Insufficient,
}
```

A few exceptions to these classifications exist, but these classifications
capture almost all of the error variants that exist within ibc-rs. New error
variants defined going forward should fall into one of the above
classifications.

### Structuring Variants and Error Messages

We'll start with conventions that apply to all variants, regardless of their
classification. Error variants themselves should start with a capital letter,
while error messages should all start with a lower-case letter.
String-interpolated values within error messages should all be surrounded by
backticks, in order to signify that it is an interpolated value. The exception
to this is nested errors that are appended to the end of an error message: these
interpolated values are __not__ surrounded by backticks. Instead, they should
all follow a colon in the error message itself; thus, colons indicate that the
following is a nested error message. Lastly, error messages should all start
with the classification of its respective error variant, i.e., an error message
for an `Invalid` error class should start with "invalid...", while an error
message for a `Missing` error class should start with "missing...", etc.

When it comes to the `NestedError` and `Mismatched` classifications, the
structure and formatting do not deviate. `NestedError`s are always newtype
wrappers around a contained error. The sole purpose of these variants is to
provide a means of converting the contained error to the containing error type.
Thus, every `NestedError` variant should be accompanied by a `From<SomeError>
for ContainingError` impl. The naming scheme for `NestedError` variants should
include the name of the contained error, minus the word "Error" itself. The
error message then should clearly delineate the type of the contained error, the
fact that the message is referring to a lower-level error, and the contents of
the lower-level error. An example looks like this:

```rust
    /// timestamp error: {0}
    Timestamp(TimestampError),
```

Note the colon followed by the interpolation of the nested error: it should __not__ be
surrounded by backticks.

The `Mismatched` classification is used for situations where an expected
instance of a type is known, along with the instance that was actually found.
These are both included in the error under the `expected` and `actual` fields.
The error message for this class should always include "expected ``{expected}``,
actual ``{actual}``"; this statement should be precluded by "mismatched
[TYPE]:". The type should be spelled in the singular and followed by a colon.
The `expected` and `actual` interpolated values in the error message should be
surrounded by ``{}`` backticks and then braces.

The rest of the error classes are a bit more freeform in how they are
structured. They could take the form of unit structs that do not contain any
values, serving mainly to surface a specific error message and nothing more.

```rust
    /// missing attribute key
    MissingAttributeKey,
    /// missing attribute value
    MissingAttributeValue,
```

These two variants each highlight the fact that a very particular resource is
missing.

Single-element unit structs are used primarily to surface an invalid or
duplicate type, opting to not provide any additional context other than what the
error message itself provides.

```rust
    /// invalid status `{0}`
    InvalidStatus(Status),
    /// duplicate client state `{0}`
    DuplicateClientState(ClientId),
```

The most common form of variant is a one-element struct with a `description`
field. This serves to allow injecting more context at the call site of the error
in the form of a String. This is the most general variant: care should be taken
to not allow these sorts of variants to be abused for unintended purposes. Note
that, like variants that contain nested errors, descriptions should be
interpolated in the error message without enclosing backticks.

```rust
    /// failed to verify header: {description}
    FailedToVerifyHeader { description: String },
    /// invalid hash: {description}
    InvalidHash { description: String },
```

## Future Work

In light of the stated downsides, a natural follow-up of this work would be to
generalize ibc-rs's error handling architecture to allow hosts to introduce
their own bespoke error types. This would allow host developers to define more
precise error types and variants that can better signal what the root cause of
an error might be. This would improve upon the rather generic error variants
that are exposed through ibc-rs's own `HostError` definition.

The downside is that this would add additional work on top of the already considerable amount
of work that it takes to integrate ibc-rs into host chains.

## References

- [#1319][issue-1319]: Consolidating duplicated error variants into the `DecodingError` type
- [#1320][issue-1320]: Defining the `HostError` type
- [#1339][issue-1339]: Merging `PacketError` into `ChannelError`
- [#1340][issue-1340]: Adding `HostError` variants to each module-level error
- [#1346][issue-1346]: Cleaning up generic `String` error variants

[dispatch]: https://github.com/cosmos/ibc-rs/blob/4aecaece9bda3c0f4a3b6a8379d73bd7eddc2cc4/ibc-core/ics25-handler/src/entrypoint.rs#L35
[validate]: https://github.com/cosmos/ibc-rs/blob/4aecaece9bda3c0f4a3b6a8379d73bd7eddc2cc4/ibc-core/ics25-handler/src/entrypoint.rs#L54
[execute]: https://github.com/cosmos/ibc-rs/blob/4aecaece9bda3c0f4a3b6a8379d73bd7eddc2cc4/ibc-core/ics25-handler/src/entrypoint.rs#L130
[issue-1319]: https://github.com/cosmos/ibc-rs/issues/1319
[issue-1320]: https://github.com/cosmos/ibc-rs/issues/1320
[issue-1339]: https://github.com/cosmos/ibc-rs/issues/1339
[issue-1340]: https://github.com/cosmos/ibc-rs/issues/1340
[issue-1346]: https://github.com/cosmos/ibc-rs/issues/1346
