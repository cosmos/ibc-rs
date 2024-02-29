use core::str::FromStr;

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

use super::{TestBlock, TestHeader, TestHost};
use crate::fixtures::clients::tendermint::ClientStateConfig;
use crate::testapp::ibc::core::types::MockClientConfig;

#[derive(Debug)]
pub struct TendermintHost(ChainId);

#[derive(Debug, Clone)]
pub struct TendermintBlock(TmLightBlock);

impl TendermintBlock {
    pub fn inner(&self) -> &TmLightBlock {
        &self.0
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TendermintHeader {
    pub trusted_height: Height,
    pub trusted_next_validators: ValidatorSet,
    pub light_block: TmLightBlock,
}

impl TendermintHeader {
    pub fn set_trusted_height(&mut self, trusted_height: Height) {
        self.trusted_height = trusted_height
    }

    pub fn set_trusted_next_validators_set(&mut self, trusted_next_validators: ValidatorSet) {
        self.trusted_next_validators = trusted_next_validators
    }

    pub fn header(&self) -> &TmHeader {
        &self.light_block.signed_header.header
    }
}

impl From<TendermintHeader> for Header {
    fn from(header: TendermintHeader) -> Self {
        Self {
            signed_header: header.light_block.signed_header,
            validator_set: header.light_block.validators,
            trusted_height: header.trusted_height,
            trusted_next_validator_set: header.trusted_next_validators,
        }
    }
}

#[derive(Debug)]
pub struct BlockParams {
    pub validators: Vec<TestgenValidator>,
    pub next_validators: Vec<TestgenValidator>,
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

impl TestHost for TendermintHost {
    type Block = TendermintBlock;
    type BlockParams = BlockParams;
    type LightClientParams = MockClientConfig;
    type ClientState = ClientState;

    fn new(chain_id: ChainId) -> Self {
        Self(chain_id)
    }

    fn chain_id(&self) -> &ChainId {
        &self.0
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
        latest_block: &Self::Block,
        params: &Self::LightClientParams,
    ) -> Self::ClientState {
        let client_state: ClientState = ClientStateConfig::builder()
            .chain_id(self.chain_id().clone())
            .latest_height(latest_block.height())
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

impl TestHeader for TendermintHeader {
    type ConsensusState = ConsensusState;

    fn height(&self) -> Height {
        Height::new(
            ChainId::from_str(self.light_block.signed_header.header.chain_id.as_str())
                .expect("Never fails")
                .revision_number(),
            self.light_block.signed_header.header.height.value(),
        )
        .expect("Never fails")
    }

    fn timestamp(&self) -> Timestamp {
        self.light_block.signed_header.header.time.into()
    }
}

impl From<TendermintHeader> for ConsensusState {
    fn from(header: TendermintHeader) -> Self {
        ConsensusState::from(header.light_block.signed_header.header)
    }
}

impl From<TendermintBlock> for TendermintHeader {
    fn from(block: TendermintBlock) -> Self {
        // let trusted_height = block.height().decrement().unwrap_or(block.height());
        // let trusted_next_validators = block.0.validators.clone();

        // from old impl

        let trusted_height = Height::new(block.height().revision_number(), 1).expect("Never fails");
        let trusted_next_validators = block.0.next_validators.clone();
        let light_block = block.0;

        // trust the previous block by default
        Self {
            trusted_height,
            trusted_next_validators,
            light_block,
        }
    }
}

impl From<TendermintHeader> for Any {
    fn from(value: TendermintHeader) -> Self {
        let value = RawHeader {
            signed_header: Some(value.light_block.signed_header.into()),
            validator_set: Some(value.light_block.validators.into()),
            trusted_height: Some(value.trusted_height.into()),
            trusted_validators: Some(value.trusted_next_validators.into()),
        }
        .to_vec();

        Self {
            type_url: TENDERMINT_HEADER_TYPE_URL.to_string(),
            value,
        }
    }
}
