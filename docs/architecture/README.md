# Architecture Decision Records (ADR)

This is a location to record all high-level architecture decisions in the IBC-RS project.

You can read more about the ADR concept in this [blog post](https://product.reverb.com/documenting-architecture-decisions-the-reverb-way-a3563bb24bd0#.78xhdix6t).

An ADR should provide:

- Context on the relevant goals and the current state
- Proposed changes to achieve the goals
- Summary of pros and cons
- References
- Changelog

Note the distinction between an ADR and a spec. The ADR provides the context, intuition, reasoning, and
justification for a change in architecture, or for the architecture of something
new. The spec is much more compressed and streamlined summary of everything as
it is or should be.

If recorded decisions turned out to be lacking, convene a discussion, record the new decisions here, and then modify the code to match.

Note the context/background should be written in the present tense.

To suggest an ADR, please make use of the [ADR template](./adr-template.md) provided.

## Table of Contents

| ADR \#                                                  | Description                                  | Status   |
| ------------------------------------------------------- | -------------------------------------------- | -------- |
| [001](./adr-001-handler-implementation.md)              | IBC handlers implementation                  | Accepted |
| [002](./adr-002-error.md)                               | Error Management                             | Accepted |
| [003](./adr-003-ics20-implementation.md)                | ICS20 implementation                         | Accepted |
| [004](./adr-004-light-client-crates-extraction.md)      | Light client crates extraction               | Accepted |
| [005](./adr-005-handlers-redesign.md)                   | Handlers validation and execution separation | Accepted |
| [006](./adr-006-upgrade-client-implementation.md)       | Chain and client upgradability               | Accepted |
| [007](./adr-007-light-client-contexts.md)               | Light client contexts                        | Accepted |
| [008](./adr-008-restructure-ibc-crate.md)               | Restructure `ibc` crate                      | Accepted |
| [010](./adr-010-enable-standalone-ics02-integration.md) | Enable standalone `ics-02` integration       | Accepted |
