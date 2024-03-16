# IBC Query

## Overview

This crate offers a comprehensive set of utility types, traits, and functions
designed for integrating either a gRPC query server or implementing RPC methods
in hosts. It specifically facilitates querying the state of the IBC core client,
connection, and channel layers of a chain enabled with `ibc-rs`.

## Features

- Provides essential utility request/response domain types and their conversions
to the proto types for efficient integration.
- Provides convenient query objects with pre-implemented gRPC query services.
- Offers convenient objects on which query service has been implemented and
- Includes convenient `QueryContext` and `ProvableContext` traits that extend
  the capabilities of an implemented IBC module, enabling the retrieval of state
  from the chain.
- Derives `serde` for all the domain types to facilitate (de)serialization of
  the domain types.
