use alloc::sync::Arc;
use core::str::FromStr;
use core::time::Duration;
use std::sync::Mutex;

use ibc::clients::tendermint::client_state::ClientState;
use ibc::clients::tendermint::consensus_state::ConsensusState;
use ibc::clients::tendermint::types::proto::v1::Header as RawHeader;
use ibc::clients::tendermint::types::{Header, TENDERMINT_HEADER_TYPE_URL};
use ibc::core::client::types::Height;
use ibc::core::host::types::identifiers::ChainId;
use ibc::core::primitives::prelude::*;
use ibc::core::primitives::Timestamp;
use ibc::primitives::proto::Any;
use ibc::primitives::ToVec;
use tendermint::block::Header as TmHeader;
use tendermint::validator::Set as ValidatorSet;
use tendermint_testgen::light_block::TmLightBlock;
use tendermint_testgen::{
    Generator, Header as TestgenHeader, LightBlock as TestgenLightBlock,
    Validator as TestgenValidator,
};

use crate::context::MockClientConfig;
use crate::fixtures::clients::tendermint::ClientStateConfig;
use crate::hosts::{HostParams, TestBlock, TestHeader, TestHost};

#[derive(Debug)]
pub struct TendermintHost {
    pub chain_id: ChainId,
    pub block_time: Duration,
    pub genesis_timestamp: Timestamp,

    /// The chain of blocks underlying this context.
    pub history: Arc<Mutex<Vec<TendermintBlock>>>,
}

impl TestHost for TendermintHost {
    type Block = TendermintBlock;
    type BlockParams = BlockParams;
    type LightClientParams = MockClientConfig;
    type ClientState = ClientState;

    fn build(params: HostParams) -> Self {
        let HostParams {
            chain_id,
            block_time,
            genesis_timestamp,
        } = params;

        Self {
            chain_id,
            block_time,
            genesis_timestamp,
            history: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn chain_id(&self) -> &ChainId {
        &self.chain_id
    }

    fn is_empty(&self) -> bool {
        self.history.lock().expect("lock").is_empty()
    }

    fn genesis_timestamp(&self) -> Timestamp {
        self.genesis_timestamp
    }

    fn latest_block(&self) -> Self::Block {
        self.history
            .lock()
            .expect("lock")
            .last()
            .cloned()
            .expect("Never fails")
    }

    fn get_block(&self, target_height: &Height) -> Option<Self::Block> {
        self.history
            .lock()
            .expect("lock")
            .get(target_height.revision_height() as usize - 1)
            .cloned() // indexed from 1
    }

    fn push_block(&self, block: Self::Block) {
        self.history.lock().expect("lock").push(block);
    }

    fn generate_block(
        &self,
        height: u64,
        timestamp: Timestamp,
        params: &Self::BlockParams,
    ) -> Self::Block {
        TendermintBlock(
            TestgenLightBlock::new_default_with_header(
                TestgenHeader::new(&params.validators)
                    .height(height)
                    .chain_id(self.chain_id().as_str())
                    .next_validators(&params.next_validators)
                    .time(timestamp.into_tm_time().expect("Never fails")),
            )
            .validators(&params.validators)
            .next_validators(&params.next_validators)
            .generate()
            .expect("Never fails"),
        )
    }

    fn generate_client_state(
        &self,
        latest_height: Height,
        params: &Self::LightClientParams,
    ) -> Self::ClientState {
        let client_state: ClientState = ClientStateConfig::builder()
            .chain_id(self.chain_id().clone())
            .latest_height(
                self.get_block(&latest_height)
                    .expect("block exists")
                    .height(),
            )
            .trusting_period(params.trusting_period)
            .max_clock_drift(params.max_clock_drift)
            .unbonding_period(params.unbonding_period)
            .build()
            .try_into()
            .expect("never fails");

        client_state.inner().validate().expect("never fails");

        client_state
    }
}

#[derive(Debug, Clone)]
pub struct TendermintBlock(TmLightBlock);

impl TendermintBlock {
    pub fn inner(&self) -> &TmLightBlock {
        &self.0
    }
}

impl TestBlock for TendermintBlock {
    type Header = TendermintHeader;

    fn height(&self) -> Height {
        Height::new(
            ChainId::from_str(self.0.signed_header.header.chain_id.as_str())
                .expect("Never fails")
                .revision_number(),
            self.0.signed_header.header.height.value(),
        )
        .expect("Never fails")
    }

    fn timestamp(&self) -> Timestamp {
        self.0.signed_header.header.time.into()
    }
}

#[derive(Debug)]
pub struct BlockParams {
    pub validators: Vec<TestgenValidator>,
    pub next_validators: Vec<TestgenValidator>,
}

impl BlockParams {
    pub fn from_validator_history(validator_history: Vec<Vec<TestgenValidator>>) -> Vec<Self> {
        validator_history
            .windows(2)
            .map(|vals| Self {
                validators: vals[0].clone(),
                next_validators: vals[1].clone(),
            })
            .collect()
    }
}

impl Default for BlockParams {
    fn default() -> Self {
        let validators = vec![
            TestgenValidator::new("1").voting_power(50),
            TestgenValidator::new("2").voting_power(50),
        ];

        Self {
            validators: validators.clone(),
            next_validators: validators,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TendermintHeader(Header);

impl TendermintHeader {
    pub fn set_trusted_height(&mut self, trusted_height: Height) {
        self.0.trusted_height = trusted_height
    }

    pub fn set_trusted_next_validators_set(&mut self, trusted_next_validator_set: ValidatorSet) {
        self.0.trusted_next_validator_set = trusted_next_validator_set
    }

    pub fn header(&self) -> &TmHeader {
        &self.0.signed_header.header
    }
}

impl TestHeader for TendermintHeader {
    type ConsensusState = ConsensusState;

    fn height(&self) -> Height {
        Height::new(
            ChainId::from_str(self.0.signed_header.header.chain_id.as_str())
                .expect("Never fails")
                .revision_number(),
            self.0.signed_header.header.height.value(),
        )
        .expect("Never fails")
    }

    fn timestamp(&self) -> Timestamp {
        self.0.signed_header.header.time.into()
    }
}

impl From<TendermintHeader> for Header {
    fn from(header: TendermintHeader) -> Self {
        header.0
    }
}

impl From<TendermintHeader> for ConsensusState {
    fn from(header: TendermintHeader) -> Self {
        ConsensusState::from(header.0.signed_header.header)
    }
}

impl From<TendermintBlock> for TendermintHeader {
    fn from(block: TendermintBlock) -> Self {
        let trusted_height = block.height();

        let TmLightBlock {
            signed_header,
            validators: validator_set,
            ..
        } = block.0;

        let trusted_next_validator_set = validator_set.clone();

        // by default trust the current height and validators
        Self(Header {
            signed_header,
            validator_set,
            trusted_height,
            trusted_next_validator_set,
        })
    }
}

impl From<TendermintHeader> for Any {
    fn from(value: TendermintHeader) -> Self {
        Self {
            type_url: TENDERMINT_HEADER_TYPE_URL.to_string(),
            value: RawHeader::from(value.0).to_vec(),
        }
    }
}
