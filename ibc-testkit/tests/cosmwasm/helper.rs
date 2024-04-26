use std::str::FromStr;

use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::{coins, Env, MessageInfo, Timestamp as CwTimestamp};
use ibc::clients::tendermint::types::ConsensusState;
use ibc::core::primitives::Timestamp as IbcTimestamp;
use tendermint::Hash;

pub fn dummy_msg_info() -> MessageInfo {
    mock_info("creator", &coins(1000, "ibc"))
}

pub fn dummy_checksum() -> Vec<u8> {
    hex::decode("2469f43c3ca20d476442bd3d98cbd97a180776ab37332aa7b02cae5a620acfc6")
        .expect("Never fails")
}

pub fn dummy_sov_consensus_state(timestamp: IbcTimestamp) -> ConsensusState {
    ConsensusState::new(
        vec![0].into(),
        timestamp.into_tm_time().expect("Time exists"),
        // Hash of default validator set
        Hash::from_str("D6B93922C33AAEBEC9043566CB4B1B48365B1358B67C7DEF986D9EE1861BC143")
            .expect("Never fails"),
    )
}

/// Returns a mock environment with the current timestamp. This is define to use
/// for testing client expiry and other time-sensitive operations.
pub fn mock_env_with_timestamp_now() -> Env {
    let mut env = mock_env();
    let now_nanos = IbcTimestamp::now().nanoseconds();
    env.block.time = CwTimestamp::from_nanos(now_nanos);
    env
}
