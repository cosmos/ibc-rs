# ADR 11: Refactor `ibc-rs`'s Error Handling Architecture

## Changelog

- 2024-**-**: Draft Proposed

## Status

Proposed

## Context

As a library whose main use cases involve integrating with external host code bases, ibc-rs's
error handling architecture should strive to achieve the following goals:

1. Empower host developers to respond to ibc-rs-originating errors that affect host logic.
2. Assist ibc-rs core developers' debugging efforts by providing detailed, human-readable
   error reports and/or stack traces.

These two aims highlight the need to clearly distinguish between *host*-level errors and
*protocol*-level errors. Developers working at the host level often do not need to concern
themselves with lower-level protocol errors; even if they do, those errors should not be
exposed to host developers at the fine-grained level of granularity that is more appropriate
for protocol-level developers.

As it currently stands, ibc-rs exposes a top-level `ContextError` type that is returned
by all validation and execution context methods, which are those that need to be implemented
by hosts as part of the ibc-rs integration process. This `ContextError` type encapsulates all
of the protocol-level errors that ibc-rs exposes. As a result, the onus is being placed on
host developers to decide whether these errors are relevant or not. In other words, the
distinction between host- and protocol-level errors is not reflected in ibc-rs's error
handling methodology.

### Proposal

#### Some changes to `ContextError`

```diff
- pub enum ContextError {
+ pub enum HandlerError {
    /// ICS02 Client error: `{0}`
    Client(ClientError),
    /// ICS03 Connection error: `{0}`
    Connection(ConnectionError),
    /// ICS04 Channel error: `{0}`
    Channel(ChannelError),
-   /// ICS04 Packet error: `{0}`
-   Packet(PacketError),
    /// ICS26 Router error: `{0}`
    Router(RouterError),
}
```

#### The new `HostError` type

In light of the rationale stated above, we propose adding a new `HostError` type that makes
clear that an error is originating from a host context, as opposed to originating from within
the ibc-rs library. We introduce the following concrete `HostError` type in the ics24-host
crate:

```rust
pub enum HostError {
    /// invalid state: `{description}`
    InvalidState { description: String },
    /// missing state: `{description}`
    MissingState { description: String },
    /// failed to update store: `{description}`
    FailedToStore { description: String },
    /// failed to retrieve from store: `{description}`
    FailedToRetrieve { description: String },
    /// other error: `{description}`
    Other { description: String },
}
```

This initial definition offers fairly generic error variants that nonetheless aim to capture
most of the error use-cases faced by hosts. One such notable use-case is fetching/retrieving
data from the host's storage. 

Host errors can occur within any of the core ibc-rs modules. Thus, we'll be adding `HostError`
variants to each of the module-level error types where appropriate: `ClientError` in ICS02, 
`ConnectionError` in ICS03, and `ChannelError` in ICS04. Note that as of now a `HostError`
variant is not being added to the `RouterError` type in ICS26 as it is not used by hosts, i.e.,
it does not expose its own handlers. 

[Add context here about some of the error calls that changed to `HostError`]

#### Defining a new `DecodingError` type


### Positives

### Negatives


## Future Work

A natural follow-up of this work would be to generalize ibc-rs's error handling architecture
to allow hosts to introduce their own bespoke error types. This would allow host developers
to define more precise error types and variants that can better signal what the root cause
of an error might be. This would improve upon the rather generic error variants that are
exposed through ibc-rs's own `HostError` definition. 

The downside is that this would add additional work on top of the already considerable amount 
of work that it takes to integrate ibc-rs into host chains.
