use alloc::fmt::Debug;
use core::time::Duration;

use basecoin_store::context::ProvableStore;
use bon::builder;
use ibc::core::client::context::client_state::ClientStateValidation;
use ibc::core::client::types::Height;
use ibc::core::primitives::prelude::*;
use ibc::core::primitives::Timestamp;

use crate::context::StoreGenericTestContext;
use crate::hosts::{HostClientState, TestBlock, TestHost};
use crate::testapp::ibc::core::router::MockRouter;
use crate::testapp::ibc::core::types::{MockIbcStore, DEFAULT_BLOCK_TIME_SECS};
use crate::utils::year_2023;

/// Returns a dummy [`StoreGenericTestContext`], for testing purposes only!
#[builder(finish_fn = build)]
pub fn dummy_store_generic_test_context<S, H>(
    #[builder(default)] host: H,
    #[builder(default = Duration::from_secs(DEFAULT_BLOCK_TIME_SECS))] block_time: Duration,
    #[builder(default = year_2023())] latest_timestamp: Timestamp,
    #[builder(default)] block_params_history: Vec<H::BlockParams>,
    #[builder(default = Height::new(0, 5).expect("Never fails"))] latest_height: Height,
) -> StoreGenericTestContext<S, H>
where
    S: ProvableStore + Debug + Default,
    H: TestHost,
    HostClientState<H>: ClientStateValidation<MockIbcStore<S>>,
{
    assert_ne!(
        latest_height.revision_height(),
        0,
        "The chain must have a non-zero revision_height"
    );

    // timestamp at height 1
    let genesis_timestamp = (latest_timestamp
        - (block_time * u32::try_from(latest_height.revision_height() - 1).expect("no overflow")))
    .expect("no underflow");

    let mut context = StoreGenericTestContext {
        multi_store: Default::default(),
        host,
        ibc_store: MockIbcStore::new(latest_height.revision_number(), Default::default()),
        ibc_router: MockRouter::new_with_transfer(),
    };

    // store is at height 0; no block

    context.advance_genesis_height(genesis_timestamp, &Default::default());

    // store is at height 1; one block

    context = context.advance_block_up_to_height(
        latest_height
            .sub(block_params_history.len() as u64)
            .expect("no error"),
    );

    for block_params in block_params_history {
        context.advance_block_height_with_params(block_time, &block_params);
    }

    assert_eq!(
        context.host.latest_block().height(),
        latest_height,
        "The latest height in the host must match the latest height in the context"
    );

    assert_eq!(
        context.host.latest_block().timestamp(),
        latest_timestamp,
        "The latest timestamp in the host must match the latest timestamp in the context"
    );

    context
}
