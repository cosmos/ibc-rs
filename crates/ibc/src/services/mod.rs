//! Implementation of the gRPC services of core IBC components.
//!
//! The provided structs includes blanket implementation of their corresponding gRPC service traits,
//! if the host implements the following _context_ traits.
//! - [`ValidationContext`](crate::core::ValidationContext)
//! - [`ProvableContext`](crate::core::ProvableContext)
//! - [`QueryContext`](crate::core::QueryContext)
//! - [`UpgradeValidationContext`](crate::hosts::tendermint::upgrade_proposal::UpgradeValidationContext)
//!   - Only for [`ClientQuery::upgraded_client_state`](ibc_proto::ibc::core::client::v1::query_server::Query::upgraded_client_state) and [`ClientQuery::upgraded_client_state`](ibc_proto::ibc::core::client::v1::query_server::Query::upgraded_consensus_state)
//!
//! Example
//! ```rust,ignore
//! use ibc_proto::ibc::core::{
//!     channel::v1::query_server::QueryServer as ChannelQueryServer
//!     client::v1::query_server::QueryServer as ClientQueryServer,
//!     connection::v1::query_server::QueryServer as ConnectionQueryServer,
//! }
//! use ibc::core::{ProvableContext, QueryContext, ValidationContext};
//! use ibc::hosts::tendermint::upgrade_proposal::UpgradeValidationContext;
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
//! let client_service = ClientQueryServer::new(ClientQueryService::new(ibc, upgrade))
//! let connection_service = ConnectionQueryServer::new(ConnectionQueryService::new(ibc))
//! let channel_service = ChannelQueryServer::new(ChannelQueryService::new(ibc))
//!
//! let grpc_server = tonic::transport::Server::builder()
//!       .add_service(client_service)
//!       .add_service(connection_service)
//!       .add_service(channel_service)
//!       .serve(addr);
//! ```

mod channel;
mod client;
mod connection;
mod error;

pub use channel::ChannelQueryService;
pub use client::ClientQueryService;
pub use connection::ConnectionQueryService;
