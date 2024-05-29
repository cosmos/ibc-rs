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
- Derives `serde` and `schema` for all the domain types enabling easy
  (de)serialization. This feature is particularly beneficial for JSON RPC
  implementations.

## Remarks

- At present, the Protobuf representation of request types does not include
  support for querying at a specific height. Consequently, the current state of
  `ibc-query` allows conversion from protos as a compatible direction but does
  not support conversion into protos due to the absence of the `query_height`
  fields.

- Currently `ibc-query` does not support pagination. If pagination is a
  requirement for your project, please open an issue and provide details about
  your usage.
