use alloc::sync::Arc;
use core::cmp::min;
use core::ops::{Add, Sub};
use core::time::Duration;

use ibc::core::ics04_channel::packet::{Packet, Sequence};
use ibc::core::ics04_channel::timeout::TimeoutHeight;
use ibc::core::ics24_host::identifier::{ChainId, ChannelId, PortId};
use ibc::core::timestamp::Timestamp;
use ibc::prelude::*;
use ibc::Height;
use parking_lot::Mutex;
use tendermint_testgen::Validator as TestgenValidator;
use typed_builder::TypedBuilder;

use super::types::DEFAULT_BLOCK_TIME_SECS;
use crate::hosts::block::{HostBlock, HostType};
use crate::testapp::ibc::core::types::{MockContext, MockIbcStore};

/// Configuration for a `MockContext` type.
#[derive(Debug, TypedBuilder)]
#[builder(build_method(into = MockContext))]
pub struct MockContextConfig {
    #[builder(default = HostType::Mock)]
    host_type: HostType,

    host_id: ChainId,

    #[builder(default = Duration::from_secs(DEFAULT_BLOCK_TIME_SECS))]
    block_time: Duration,

    // may panic if validator_set_history size is less than max_history_size + 1
    #[builder(default = 5)]
    max_history_size: u64,

    #[builder(default, setter(strip_option))]
    validator_set_history: Option<Vec<Vec<TestgenValidator>>>,

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
            events: Vec::new(),
            logs: Vec::new(),
        }
    }
}

/// Configuration for a `PacketData` type.
#[derive(TypedBuilder, Debug)]
#[builder(build_method(into = Packet))]
pub struct PacketConfig {
    #[builder(default)]
    pub seq_on_a: Sequence,
    #[builder(default = PortId::transfer())]
    pub port_id_on_a: PortId,
    #[builder(default)]
    pub chan_id_on_a: ChannelId,
    #[builder(default = PortId::transfer())]
    pub port_id_on_b: PortId,
    #[builder(default)]
    pub chan_id_on_b: ChannelId,
    #[builder(default)]
    pub data: Vec<u8>,
    #[builder(default)]
    pub timeout_height_on_b: TimeoutHeight,
    #[builder(default)]
    pub timeout_timestamp_on_b: Timestamp,
}

impl From<PacketConfig> for Packet {
    fn from(config: PacketConfig) -> Self {
        Packet {
            seq_on_a: config.seq_on_a,
            port_id_on_a: config.port_id_on_a,
            chan_id_on_a: config.chan_id_on_a,
            port_id_on_b: config.port_id_on_b,
            chan_id_on_b: config.chan_id_on_b,
            data: config.data,
            timeout_height_on_b: config.timeout_height_on_b,
            timeout_timestamp_on_b: config.timeout_timestamp_on_b,
        }
    }
}
