use alloc::fmt::Debug;
use core::time::Duration;

use basecoin_store::context::ProvableStore;
use ibc::core::client::context::client_state::ClientStateValidation;
use ibc::core::client::types::Height;
use ibc::core::primitives::prelude::*;
use ibc::core::primitives::Timestamp;
use typed_builder::TypedBuilder;

use crate::context::MockGenericContext;
use crate::hosts::{HostClientState, TestBlock, TestHost};
use crate::testapp::ibc::core::router::MockRouter;
use crate::testapp::ibc::core::types::{MockIbcStore, DEFAULT_BLOCK_TIME_SECS};
use crate::utils::year_2023;

/// Configuration of the `MockContext` type for generating dummy contexts.
#[derive(Debug, TypedBuilder)]
#[builder(build_method(into))]
pub struct MockContextConfig<H>
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

impl<S, H> From<MockContextConfig<H>> for MockGenericContext<S, H>
where
    S: ProvableStore + Debug + Default,
    H: TestHost,
    HostClientState<H>: ClientStateValidation<MockIbcStore<S>>,
{
    fn from(params: MockContextConfig<H>) -> Self {
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
            main_store: Default::default(),
            host: params.host,
            ibc_store: MockIbcStore::new(
                params.latest_height.revision_number(),
                Default::default(),
            ),
            ibc_router: MockRouter::new_with_transfer(),
        };

        // store is a height 0; no block

        context.generate_genesis_block(genesis_timestamp, &Default::default());

        // store is a height 1; one block

        context = context.advance_block_up_to(
            params
                .latest_height
                .sub(params.block_params_history.len() as u64)
                .expect("no error"),
        );

        for block_params in params.block_params_history {
            context.advance_with_block_params(params.block_time, &block_params);
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
