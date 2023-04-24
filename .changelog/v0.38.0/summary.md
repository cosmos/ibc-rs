This release involves splitting back the newly defiend `MsgUpdateClient` type in
v0.37.0 into two distinct IBC message structs, namely `MsgUpdateClient` and
`MsgSubmitMisbehaviour`. Additionally, we have made improvements to the
`Version` validations in connection and channel handshakes, carried out some
clean-ups, discarded now-unused `store_client_type` interface, and removed
`IbcEventType` to enable each IBC event variant to define its own set of event
types.

There are consensus-breaking changes
