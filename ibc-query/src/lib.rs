//! Contains a set of utility traits and implementations for querying the state
//! of an `ibc-rs` enabled chain, including implementation of essential IBC
//! query methods and gRPC query services defined in `ibc-proto` crate.
//! Therefore, some ready-to-use Query structs for each layer of the client,
//! connection, and channel have been implemented and exposed by this crate.
//!
//! The provided structs includes blanket implementation of their corresponding
//! gRPC service traits, if the host implements the following _context_ traits:
//! - [`ValidationContext`](ibc::core::host::ValidationContext)
//! - [`ProvableContext`](crate::core::context::ProvableContext)
//! - [`QueryContext`](crate::core::context::QueryContext)
//! - [`UpgradeValidationContext`](ibc::cosmos_host::upgrade_proposal::UpgradeValidationContext)
//!   - Only for
//!     [`ClientQuery::upgraded_client_state`](ibc_proto::ibc::core::client::v1::query_server::Query::upgraded_client_state)
//!     and
//!     [`ClientQuery::upgraded_client_state`](ibc_proto::ibc::core::client::v1::query_server::Query::upgraded_consensus_state)
//!
//! Example
//! ```rust,ignore
//! use ibc_proto::ibc::core::{
//!     channel::v1::query_server::QueryServer as ChannelQueryServer
//!     client::v1::query_server::QueryServer as ClientQueryServer,
//!     connection::v1::query_server::QueryServer as ConnectionQueryServer,
//! }
//! use ibc::core::ValidationContext;
//! use ibc::hosts::tendermint::upgrade_proposal::UpgradeValidationContext;
//! use ibc::services::core::{ProvableContext, QueryContext};
//! use ibc::services::{ChannelQueryService, ClientQueryService, ConnectionQueryService};
//!
//! struct Ibc;
//! impl ValidationContext for Ibc { }
//! impl ProvableContext for Ibc { }
//! impl QueryContext for Ibc { }
//!
//! struct Upgrade;
//! impl UpgradeValidationContext for Upgrade { }
//!
//! let ibc = Ibc::new();
//! let upgrade = Upgrade::new();
//!
//! // `ibc` and `upgrade` must be thread-safe
//!
//! let client_service = ClientQueryServer::new(ClientQueryService::new(ibc.clone(), upgrade))
//! let connection_service = ConnectionQueryServer::new(ConnectionQueryService::new(ibc.clone()))
//! let channel_service = ChannelQueryServer::new(ChannelQueryService::new(ibc))
//!
//! let grpc_server = tonic::transport::Server::builder()
//!       .add_service(client_service)
//!       .add_service(connection_service)
//!       .add_service(channel_service)
//!       .serve(addr);
//! ```
//!

#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![no_std]
#![deny(
    warnings,
    trivial_casts,
    trivial_numeric_casts,
    unused_import_braces,
    unused_qualifications,
    rust_2018_idioms
)]
#![forbid(unsafe_code)]

extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

pub mod core;
pub mod error;
pub mod types;
pub mod utils;
