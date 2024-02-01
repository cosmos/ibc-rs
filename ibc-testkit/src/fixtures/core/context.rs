use alloc::sync::Arc;
use core::cmp::min;
use core::ops::{Add, Sub};
use core::time::Duration;

use ibc::core::client::types::Height;
use ibc::core::host::types::identifiers::ChainId;
use ibc::core::primitives::prelude::*;
use ibc::core::primitives::Timestamp;
use parking_lot::Mutex;
use tendermint_testgen::Validator as TestgenValidator;
use typed_builder::TypedBuilder;

use crate::hosts::block::{HostBlock, HostType};
use crate::testapp::ibc::core::types::{MockContext, MockIbcStore, DEFAULT_BLOCK_TIME_SECS};

/// Configuration of the `MockContext` type for generating dummy contexts.
#[derive(Debug, TypedBuilder)]
#[builder(build_method(into = MockContext))]
pub struct MockContextConfig {
    #[builder(default = HostType::Mock)]
    host_type: HostType,

    #[builder(default = ChainId::new("mockgaia-0").expect("Never fails"))]
    host_id: ChainId,

    #[builder(default = Duration::from_secs(DEFAULT_BLOCK_TIME_SECS))]
    block_time: Duration,

    // may panic if validator_set_history size is less than max_history_size + 1
    #[builder(default = 5)]
    max_history_size: u64,

    #[builder(default, setter(strip_option))]
    validator_set_history: Option<Vec<Vec<TestgenValidator>>>,

    #[builder(default = Height::new(0, 5).expect("Never fails"))]
    latest_height: Height,

    #[builder(default = Timestamp::now())]
    latest_timestamp: Timestamp,
}

impl From<MockContextConfig> for MockContext {
    fn from(params: MockContextConfig) -> Self {
        assert_ne!(
            params.max_history_size, 0,
            "The chain must have a non-zero max_history_size"
        );

        assert_ne!(
            params.latest_height.revision_height(),
            0,
            "The chain must have a non-zero revision_height"
        );

        // Compute the number of blocks to store.
        let n = min(
            params.max_history_size,
            params.latest_height.revision_height(),
        );

        assert_eq!(
            params.host_id.revision_number(),
            params.latest_height.revision_number(),
            "The version in the chain identifier must match the version in the latest height"
        );

        let next_block_timestamp = params
            .latest_timestamp
            .add(params.block_time)
            .expect("Never fails");

        let history = if let Some(validator_set_history) = params.validator_set_history {
            (0..n)
                .rev()
                .map(|i| {
                    // generate blocks with timestamps -> N, N - BT, N - 2BT, ...
                    // where N = now(), BT = block_time
                    HostBlock::generate_block_with_validators(
                        params.host_id.clone(),
                        params.host_type,
                        params
                            .latest_height
                            .sub(i)
                            .expect("Never fails")
                            .revision_height(),
                        next_block_timestamp
                            .sub(params.block_time * ((i + 1) as u32))
                            .expect("Never fails"),
                        &validator_set_history[(n - i) as usize - 1],
                        &validator_set_history[(n - i) as usize],
                    )
                })
                .collect()
        } else {
            (0..n)
                .rev()
                .map(|i| {
                    // generate blocks with timestamps -> N, N - BT, N - 2BT, ...
                    // where N = now(), BT = block_time
                    HostBlock::generate_block(
                        params.host_id.clone(),
                        params.host_type,
                        params
                            .latest_height
                            .sub(i)
                            .expect("Never fails")
                            .revision_height(),
                        next_block_timestamp
                            .sub(params.block_time * ((i + 1) as u32))
                            .expect("Never fails"),
                    )
                })
                .collect()
        };

        MockContext {
            host_chain_type: params.host_type,
            host_chain_id: params.host_id.clone(),
            max_history_size: params.max_history_size,
            history,
            block_time: params.block_time,
            ibc_store: Arc::new(Mutex::new(MockIbcStore::default())),
        }
    }
}
