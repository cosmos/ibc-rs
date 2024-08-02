use alloc::fmt::Debug;
use core::time::Duration;

use basecoin_store::context::ProvableStore;
use ibc::core::client::context::client_state::{ClientStateExecution, ClientStateValidation};
use ibc::core::client::context::consensus_state::ConsensusState;
use ibc::core::client::context::ClientExecutionContext;
use ibc::core::client::types::error::ClientError;
use ibc::core::client::types::Height;
use ibc::core::primitives::prelude::*;
use ibc::core::primitives::Timestamp;
use ibc::primitives::proto::Any;
use typed_builder::TypedBuilder;

use crate::context::StoreGenericTestContext;
use crate::hosts::{HostClientState, HostConsensusState, TestBlock, TestHost};
use crate::testapp::ibc::core::router::MockRouter;
use crate::testapp::ibc::core::types::{MockIbcStore, DEFAULT_BLOCK_TIME_SECS};
use crate::utils::year_2023;

/// Configuration of the [`StoreGenericTestContext`] type for generating dummy contexts.
#[derive(Debug, TypedBuilder)]
#[builder(build_method(into))]
pub struct TestContextConfig<H>
where
    H: TestHost,
{
    #[builder(default)]
    host: H,

    #[builder(default = Duration::from_secs(DEFAULT_BLOCK_TIME_SECS))]
    block_time: Duration,

    #[builder(default = year_2023())]
    latest_timestamp: Timestamp,

    #[builder(default)]
    block_params_history: Vec<H::BlockParams>,

    #[builder(default = Height::new(0, 5).expect("Never fails"))]
    latest_height: Height,
}

impl<S, H, ACL, ACS> From<TestContextConfig<H>> for StoreGenericTestContext<S, H, ACL, ACS>
where
    S: ProvableStore + Debug + Default,
    H: TestHost,
    S: ProvableStore + Debug,
    ACL: From<HostClientState<H>> + ClientStateExecution<MockIbcStore<S, ACL, ACS>> + Clone,
    ACS: From<HostConsensusState<H>> + ConsensusState + Clone,
    HostClientState<H>: ClientStateValidation<MockIbcStore<S, ACL, ACS>>,
    MockIbcStore<S, ACL, ACS>:
        ClientExecutionContext<ClientStateMut = ACL, ConsensusStateRef = ACS>,
    ClientError: From<<ACL as TryFrom<Any>>::Error>,
{
    fn from(params: TestContextConfig<H>) -> Self {
        assert_ne!(
            params.latest_height.revision_height(),
            0,
            "The chain must have a non-zero revision_height"
        );

        // timestamp at height 1
        let genesis_timestamp = (params.latest_timestamp
            - (params.block_time
                * u32::try_from(params.latest_height.revision_height() - 1).expect("no overflow")))
        .expect("no underflow");

        let mut context = Self {
            multi_store: Default::default(),
            host: params.host,
            ibc_store: MockIbcStore::new(
                params.latest_height.revision_number(),
                Default::default(),
            ),
            ibc_router: MockRouter::new_with_transfer(),
        };

        // store is at height 0; no block

        context.advance_genesis_height(genesis_timestamp, &Default::default());

        // store is at height 1; one block

        context = context.advance_block_up_to_height(
            params
                .latest_height
                .sub(params.block_params_history.len() as u64)
                .expect("no error"),
        );

        for block_params in params.block_params_history {
            context.advance_block_height_with_params(params.block_time, &block_params);
        }

        assert_eq!(
            context.host.latest_block().height(),
            params.latest_height,
            "The latest height in the host must match the latest height in the context"
        );

        assert_eq!(
            context.host.latest_block().timestamp(),
            params.latest_timestamp,
            "The latest timestamp in the host must match the latest timestamp in the context"
        );

        context
    }
}
