//! [`ClientQueryService`](ClientQueryService) takes generics `I` and `U` to store `ibc_context` and `upgrade_context` that implement [`QueryContext`](QueryContext) and [`UpgradeValidationContext`](UpgradeValidationContext) respectively.
//! `I` must be a type where writes from one thread are readable from another.
//! This means using `Arc<Mutex<_>>` or `Arc<RwLock<_>>` in most cases.

use ibc::core::host::{ClientStateRef, ConsensusStateRef};
use ibc::core::primitives::prelude::*;
use ibc::cosmos_host::upgrade_proposal::{
    AnyUpgradedClientState, AnyUpgradedConsensusState, UpgradeValidationContext,
};
use ibc_proto::google::protobuf::Any;
use ibc_proto::ibc::core::client::v1::query_server::Query as ClientQuery;
use ibc_proto::ibc::core::client::v1::{
    QueryClientParamsRequest, QueryClientParamsResponse, QueryClientStateRequest,
    QueryClientStateResponse, QueryClientStatesRequest, QueryClientStatesResponse,
    QueryClientStatusRequest, QueryClientStatusResponse, QueryConsensusStateHeightsRequest,
    QueryConsensusStateHeightsResponse, QueryConsensusStateRequest, QueryConsensusStateResponse,
    QueryConsensusStatesRequest, QueryConsensusStatesResponse, QueryUpgradedClientStateRequest,
    QueryUpgradedClientStateResponse, QueryUpgradedConsensusStateRequest,
    QueryUpgradedConsensusStateResponse,
};
use tonic::{Request, Response, Status};

use super::{
    query_client_state, query_client_states, query_client_status, query_consensus_state,
    query_consensus_state_heights, query_consensus_states, query_upgraded_client_state,
    query_upgraded_consensus_state,
};
use crate::core::context::QueryContext;

// TODO(rano): currently the services don't support pagination, so we return all the results.

/// Generics `I` and `U` must be a type where writes from one thread are readable from another.
/// This means using `Arc<Mutex<_>>` or `Arc<RwLock<_>>` in most cases.
pub struct ClientQueryService<I, U>
where
    I: QueryContext + Send + Sync + 'static,
    U: UpgradeValidationContext + Send + Sync + 'static,
    ClientStateRef<I>: Into<Any>,
    AnyUpgradedClientState<U>: Into<Any>,
    AnyUpgradedConsensusState<U>: Into<Any>,
{
    ibc_context: I,
    upgrade_context: U,
}

impl<I, U> ClientQueryService<I, U>
where
    I: QueryContext + Send + Sync + 'static,
    U: UpgradeValidationContext + Send + Sync + 'static,
    ClientStateRef<I>: Into<Any>,
    AnyUpgradedClientState<U>: Into<Any>,
    AnyUpgradedConsensusState<U>: Into<Any>,
{
    /// Parameters `ibc_context` and `upgrade_context` must be a type where writes from one thread are readable from another.
    /// This means using `Arc<Mutex<_>>` or `Arc<RwLock<_>>` in most cases.
    pub fn new(ibc_context: I, upgrade_context: U) -> Self {
        Self {
            ibc_context,
            upgrade_context,
        }
    }
}

#[tonic::async_trait]
impl<I, U> ClientQuery for ClientQueryService<I, U>
where
    I: QueryContext + Send + Sync + 'static,
    U: UpgradeValidationContext + Send + Sync + 'static,
    ClientStateRef<I>: Into<Any>,
    ConsensusStateRef<I>: Into<Any>,
    AnyUpgradedClientState<U>: Into<Any>,
    AnyUpgradedConsensusState<U>: Into<Any>,
{
    async fn client_state(
        &self,
        request: Request<QueryClientStateRequest>,
    ) -> Result<Response<QueryClientStateResponse>, Status> {
        let response = query_client_state(&self.ibc_context, request.get_ref())?;

        Ok(Response::new(response))
    }

    async fn client_states(
        &self,
        request: Request<QueryClientStatesRequest>,
    ) -> Result<Response<QueryClientStatesResponse>, Status> {
        let response = query_client_states(&self.ibc_context, request.get_ref())?;

        Ok(Response::new(response))
    }

    async fn consensus_state(
        &self,
        request: Request<QueryConsensusStateRequest>,
    ) -> Result<Response<QueryConsensusStateResponse>, Status> {
        let response = query_consensus_state(&self.ibc_context, request.get_ref())?;

        Ok(Response::new(response))
    }

    async fn consensus_states(
        &self,
        request: Request<QueryConsensusStatesRequest>,
    ) -> Result<Response<QueryConsensusStatesResponse>, Status> {
        let response = query_consensus_states(&self.ibc_context, request.get_ref())?;

        Ok(Response::new(response))
    }

    async fn consensus_state_heights(
        &self,
        request: Request<QueryConsensusStateHeightsRequest>,
    ) -> Result<Response<QueryConsensusStateHeightsResponse>, Status> {
        let response = query_consensus_state_heights(&self.ibc_context, request.get_ref())?;

        Ok(Response::new(response))
    }

    async fn client_status(
        &self,
        request: Request<QueryClientStatusRequest>,
    ) -> Result<Response<QueryClientStatusResponse>, Status> {
        let response = query_client_status(&self.ibc_context, request.get_ref())?;

        Ok(Response::new(response))
    }

    async fn client_params(
        &self,
        _request: Request<QueryClientParamsRequest>,
    ) -> Result<Response<QueryClientParamsResponse>, Status> {
        Err(Status::unimplemented(
            "Querying ClientParams is not supported yet",
        ))
    }

    async fn upgraded_client_state(
        &self,
        request: Request<QueryUpgradedClientStateRequest>,
    ) -> Result<Response<QueryUpgradedClientStateResponse>, Status> {
        let response = query_upgraded_client_state(&self.upgrade_context, request.get_ref())?;

        Ok(Response::new(response))
    }

    async fn upgraded_consensus_state(
        &self,
        request: Request<QueryUpgradedConsensusStateRequest>,
    ) -> Result<Response<QueryUpgradedConsensusStateResponse>, Status> {
        let response = query_upgraded_consensus_state(&self.upgrade_context, request.get_ref())?;

        Ok(Response::new(response))
    }
}
