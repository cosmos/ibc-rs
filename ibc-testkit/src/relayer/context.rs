use ibc::core::client::context::ClientValidationContext;
use ibc::core::client::types::Height;
use ibc::core::handler::types::error::ContextError;
use ibc::core::host::types::identifiers::ClientId;
use ibc::core::host::ValidationContext;
use ibc::core::primitives::prelude::*;
use ibc::core::primitives::Signer;

use crate::testapp::ibc::clients::AnyClientState;
use crate::testapp::ibc::core::types::MockContext;
/// Trait capturing all dependencies (i.e., the context) which algorithms in ICS18 require to
/// relay packets between chains. This trait comprises the dependencies towards a single chain.
/// Most of the functions in this represent wrappers over the ABCI interface.
/// This trait mimics the `Chain` trait, but at a lower level of abstraction (no networking, header
/// types, light client, RPC client, etc.)
pub trait RelayerContext {
    /// Returns the latest height of the chain.
    fn query_latest_height(&self) -> Result<Height, ContextError>;

    /// Returns this client state for the given `client_id` on this chain.
    /// Wrapper over the `/abci_query?path=..` endpoint.
    fn query_client_full_state(&self, client_id: &ClientId) -> Option<AnyClientState>;

    /// Temporary solution. Similar to `CosmosSDKChain::key_and_signer()` but simpler.
    fn signer(&self) -> Signer;
}

impl RelayerContext for MockContext {
    fn query_latest_height(&self) -> Result<Height, ContextError> {
        ValidationContext::host_height(self)
    }

    fn query_client_full_state(&self, client_id: &ClientId) -> Option<AnyClientState> {
        // Forward call to Ics2.
        self.client_state(client_id).ok()
    }

    fn signer(&self) -> Signer {
        "0CDA3F47EF3C4906693B170EF650EB968C5F4B2C"
            .to_string()
            .into()
    }
}

#[cfg(test)]
mod tests {
    use ibc::clients::tendermint::types::client_type as tm_client_type;
    use ibc::core::client::context::client_state::ClientStateCommon;
    use ibc::core::client::types::msgs::{ClientMsg, MsgUpdateClient};
    use ibc::core::client::types::Height;
    use ibc::core::handler::types::msgs::MsgEnvelope;
    use ibc::core::host::types::identifiers::ChainId;
    use ibc::core::primitives::prelude::*;
    use tracing::debug;

    use super::RelayerContext;
    use crate::fixtures::core::context::MockContextConfig;
    use crate::hosts::block::{HostBlock, HostType};
    use crate::relayer::context::ClientId;
    use crate::relayer::error::RelayerError;
    use crate::testapp::ibc::clients::mock::client_state::client_type as mock_client_type;
    use crate::testapp::ibc::core::router::MockRouter;
    use crate::testapp::ibc::core::types::MockClientConfig;

    /// Builds a `ClientMsg::UpdateClient` for a client with id `client_id` running on the `dest`
    /// context, assuming that the latest header on the source context is `src_header`.
    pub(crate) fn build_client_update_datagram<Ctx>(
        dest: &Ctx,
        client_id: &ClientId,
        src_header: &HostBlock,
    ) -> Result<ClientMsg, RelayerError>
    where
        Ctx: RelayerContext,
    {
        // Check if client for ibc0 on ibc1 has been updated to latest height:
        // - query client state on destination chain
        let dest_client_state = dest.query_client_full_state(client_id).ok_or_else(|| {
            RelayerError::ClientStateNotFound {
                client_id: client_id.clone(),
            }
        })?;

        let dest_client_latest_height = dest_client_state.latest_height();

        if src_header.height() == dest_client_latest_height {
            return Err(RelayerError::ClientAlreadyUpToDate {
                client_id: client_id.clone(),
                source_height: src_header.height(),
                destination_height: dest_client_latest_height,
            });
        };

        if dest_client_latest_height > src_header.height() {
            return Err(RelayerError::ClientAtHigherHeight {
                client_id: client_id.clone(),
                source_height: src_header.height(),
                destination_height: dest_client_latest_height,
            });
        };

        // Client on destination chain can be updated.
        Ok(ClientMsg::UpdateClient(MsgUpdateClient {
            client_id: client_id.clone(),
            client_message: (*src_header).clone().into(),
            signer: dest.signer(),
        }))
    }

    #[test]
    /// Serves to test both ICS-26 `dispatch` & `build_client_update_datagram` functions.
    /// Implements a "ping pong" of client update messages, so that two chains repeatedly
    /// process a client update message and update their height in succession.
    fn client_update_ping_pong() {
        let chain_a_start_height = Height::new(1, 11).unwrap();
        let chain_b_start_height = Height::new(1, 20).unwrap();
        let client_on_b_for_a_height = Height::new(1, 10).unwrap(); // Should be smaller than `chain_a_start_height`
        let client_on_a_for_b_height = Height::new(1, 20).unwrap(); // Should be smaller than `chain_b_start_height`
        let num_iterations = 4;

        let client_on_a_for_b = tm_client_type().build_client_id(0);
        let client_on_b_for_a = mock_client_type().build_client_id(0);

        let chain_id_a = ChainId::new("mockgaiaA-1").unwrap();
        let chain_id_b = ChainId::new("mockgaiaB-1").unwrap();

        // Create two mock contexts, one for each chain.
        let mut ctx_a = MockContextConfig::builder()
            .host_id(chain_id_a.clone())
            .latest_height(chain_a_start_height)
            .build()
            .with_client_config(
                MockClientConfig::builder()
                    .client_chain_id(chain_id_b.clone())
                    .client_id(client_on_a_for_b.clone())
                    .latest_height(client_on_a_for_b_height)
                    .client_type(tm_client_type()) // The target host chain (B) is synthetic TM.
                    .build(),
            );
        // dummy; not actually used in client updates
        let mut router_a = MockRouter::new_with_transfer();

        let mut ctx_b = MockContextConfig::builder()
            .host_id(chain_id_b)
            .host_type(HostType::SyntheticTendermint)
            .latest_height(chain_b_start_height)
            .build()
            .with_client_config(
                MockClientConfig::builder()
                    .client_chain_id(chain_id_a)
                    .client_id(client_on_b_for_a.clone())
                    .latest_height(client_on_b_for_a_height)
                    .build(),
            );
        // dummy; not actually used in client updates
        let mut router_b = MockRouter::new_with_transfer();

        for _i in 0..num_iterations {
            // Update client on chain B to latest height of A.
            // - create the client update message with the latest header from A
            let a_latest_header = ctx_a.query_latest_header().unwrap();
            let client_msg_b_res =
                build_client_update_datagram(&ctx_b, &client_on_b_for_a, &a_latest_header);

            assert!(
                client_msg_b_res.is_ok(),
                "create_client_update failed for context destination {ctx_b:?}, error: {client_msg_b_res:?}",
            );

            let client_msg_b = client_msg_b_res.unwrap();

            // - send the message to B. We bypass ICS18 interface and call directly into
            // MockContext `recv` method (to avoid additional serialization steps).
            let dispatch_res_b = ctx_b.deliver(&mut router_b, MsgEnvelope::Client(client_msg_b));
            let validation_res = ctx_b.validate();
            assert!(
                validation_res.is_ok(),
                "context validation failed with error {validation_res:?} for context {ctx_b:?}",
            );

            // Check if the update succeeded.
            assert!(
                dispatch_res_b.is_ok(),
                "Dispatch failed for host chain b with error: {dispatch_res_b:?}"
            );
            let client_height_b = ctx_b
                .query_client_full_state(&client_on_b_for_a)
                .unwrap()
                .latest_height();
            assert_eq!(client_height_b, ctx_a.query_latest_height().unwrap());

            // Update client on chain A to latest height of B.
            // - create the client update message with the latest header from B
            // The test uses LightClientBlock that does not store the trusted height
            let mut b_latest_header = ctx_b.query_latest_header().unwrap();

            let th = b_latest_header.height();
            b_latest_header.set_trusted_height(th.decrement().unwrap());

            let client_msg_a_res =
                build_client_update_datagram(&ctx_a, &client_on_a_for_b, &b_latest_header);

            assert!(
                client_msg_a_res.is_ok(),
                "create_client_update failed for context destination {ctx_a:?}, error: {client_msg_a_res:?}",
            );

            let client_msg_a = client_msg_a_res.unwrap();

            debug!("client_msg_a = {:?}", client_msg_a);

            // - send the message to A
            let dispatch_res_a = ctx_a.deliver(&mut router_a, MsgEnvelope::Client(client_msg_a));
            let validation_res = ctx_a.validate();
            assert!(
                validation_res.is_ok(),
                "context validation failed with error {validation_res:?} for context {ctx_a:?}",
            );

            // Check if the update succeeded.
            assert!(
                dispatch_res_a.is_ok(),
                "Dispatch failed for host chain a with error: {dispatch_res_a:?}"
            );
            let client_height_a = ctx_a
                .query_client_full_state(&client_on_a_for_b)
                .unwrap()
                .latest_height();
            assert_eq!(client_height_a, ctx_b.query_latest_height().unwrap());
        }
    }
}
