This release involves splitting the newly defined `MsgUpdateClient` type in
v0.37.0 into distinct IBC message structs: `MsgUpdateClient` and
`MsgSubmitMisbehaviour`. Additionally, we made improvements to the `Version`
validations in connection and channel handshakes, discarded now-unused
`store_client_type` interface, and removed `IbcEventType` to enable each IBC
event variant to define its own set of event types.

There are consensus-breaking changes
