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
    host: Option<H>,
    block_time: Option<Duration>,
    latest_timestamp: Option<Timestamp>,
    block_params_history: Option<Vec<H::BlockParams>>,
    latest_height: Option<Height>,
) -> StoreGenericTestContext<S, H>
where
    S: ProvableStore + Debug + Default,
    H: TestHost,
    HostClientState<H>: ClientStateValidation<MockIbcStore<S>>,
{
    let host = host.unwrap_or_default();
    let block_time = block_time.unwrap_or_else(|| Duration::from_secs(DEFAULT_BLOCK_TIME_SECS));
    let latest_timestamp = latest_timestamp.unwrap_or_else(year_2023);
    let block_params_history = block_params_history.unwrap_or_default();
    let latest_height = latest_height.unwrap_or_else(|| Height::new(0, 5).expect("Never fails"));

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
