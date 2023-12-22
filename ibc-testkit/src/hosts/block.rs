//! Host chain types and methods, used by context mock.

use core::str::FromStr;

use ibc::clients::tendermint::consensus_state::ConsensusState as TmConsensusState;
use ibc::clients::tendermint::types::proto::v1::Header as RawHeader;
use ibc::clients::tendermint::types::{Header, TENDERMINT_HEADER_TYPE_URL};
use ibc::core::client::types::error::ClientError;
use ibc::core::client::types::Height;
use ibc::core::host::types::identifiers::ChainId;
use ibc::core::primitives::prelude::*;
use ibc::core::primitives::Timestamp;
use ibc::primitives::proto::{Any, Protobuf};
use ibc::primitives::ToVec;
use tendermint::block::Header as TmHeader;
use tendermint::validator::Set as ValidatorSet;
use tendermint_testgen::light_block::TmLightBlock;
use tendermint_testgen::{
    Generator, Header as TestgenHeader, LightBlock as TestgenLightBlock,
    Validator as TestgenValidator,
};

use crate::testapp::ibc::clients::mock::consensus_state::MockConsensusState;
use crate::testapp::ibc::clients::mock::header::MockHeader;
use crate::testapp::ibc::clients::AnyConsensusState;

/// Defines the different types of host chains that a mock context can emulate.
/// The variants are as follows:
/// - `Mock` defines that the context history consists of `MockHeader` blocks.
/// - `SyntheticTendermint`: the context has synthetically-generated Tendermint (light) blocks.
/// See also the `HostBlock` enum to get more insights into the underlying block type.
#[derive(Clone, Debug, Copy)]
pub enum HostType {
    Mock,
    SyntheticTendermint,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SyntheticTmBlock {
    pub trusted_height: Height,
    pub trusted_next_validators: ValidatorSet,
    pub light_block: TmLightBlock,
}

impl SyntheticTmBlock {
    pub fn header(&self) -> &TmHeader {
        &self.light_block.signed_header.header
    }
}

impl From<SyntheticTmBlock> for Header {
    fn from(light_block: SyntheticTmBlock) -> Self {
        let SyntheticTmBlock {
            trusted_height,
            trusted_next_validators,
            light_block,
        } = light_block;
        Self {
            signed_header: light_block.signed_header,
            validator_set: light_block.validators,
            trusted_height,
            trusted_next_validator_set: trusted_next_validators,
        }
    }
}

/// Depending on `HostType` (the type of host chain underlying a context mock), this enum defines
/// the type of blocks composing the history of the host chain.
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HostBlock {
    Mock(Box<MockHeader>),
    SyntheticTendermint(Box<SyntheticTmBlock>),
}

impl HostBlock {
    /// Returns the height of a block.
    pub fn height(&self) -> Height {
        match self {
            HostBlock::Mock(header) => header.height(),
            HostBlock::SyntheticTendermint(light_block) => Height::new(
                ChainId::from_str(light_block.header().chain_id.as_str())
                    .expect("Never fails")
                    .revision_number(),
                light_block.header().height.value(),
            )
            .expect("Never fails"),
        }
    }

    pub fn set_trusted_height(&mut self, height: Height) {
        match self {
            HostBlock::Mock(_) => {}
            HostBlock::SyntheticTendermint(light_block) => light_block.trusted_height = height,
        }
    }

    pub fn set_trusted_next_validators_set(&mut self, trusted_next_validators: ValidatorSet) {
        match self {
            HostBlock::Mock(_) => {}
            HostBlock::SyntheticTendermint(light_block) => {
                light_block.trusted_next_validators = trusted_next_validators
            }
        }
    }

    /// Returns the timestamp of a block.
    pub fn timestamp(&self) -> Timestamp {
        match self {
            HostBlock::Mock(header) => header.timestamp,
            HostBlock::SyntheticTendermint(light_block) => light_block.header().time.into(),
        }
    }

    /// Generates a new block at `height` for the given chain identifier and chain type.
    pub fn generate_block(
        chain_id: ChainId,
        chain_type: HostType,
        height: u64,
        timestamp: Timestamp,
    ) -> HostBlock {
        match chain_type {
            HostType::Mock => HostBlock::Mock(Box::new(MockHeader {
                height: Height::new(chain_id.revision_number(), height).expect("Never fails"),
                timestamp,
            })),
            HostType::SyntheticTendermint => HostBlock::SyntheticTendermint(Box::new(
                Self::generate_tm_block(chain_id, height, timestamp),
            )),
        }
    }

    /// Generates a new block at `height` for the given chain identifier, chain type and validator sets.
    pub fn generate_block_with_validators(
        chain_id: ChainId,
        chain_type: HostType,
        height: u64,
        timestamp: Timestamp,
        validators: &[TestgenValidator],
        next_validators: &[TestgenValidator],
    ) -> HostBlock {
        match chain_type {
            HostType::Mock => HostBlock::Mock(Box::new(MockHeader {
                height: Height::new(chain_id.revision_number(), height).expect("Never fails"),
                timestamp,
            })),
            HostType::SyntheticTendermint => {
                let light_block = TestgenLightBlock::new_default_with_header(
                    TestgenHeader::new(validators)
                        .height(height)
                        .chain_id(chain_id.as_str())
                        .next_validators(next_validators)
                        .time(timestamp.into_tm_time().expect("Never fails")),
                )
                .validators(validators)
                .next_validators(next_validators)
                .generate()
                .expect("Never fails");

                HostBlock::SyntheticTendermint(Box::new(SyntheticTmBlock {
                    trusted_height: Height::new(chain_id.revision_number(), 1)
                        .expect("Never fails"),
                    trusted_next_validators: light_block.next_validators.clone(),
                    light_block,
                }))
            }
        }
    }

    pub fn generate_tm_block(
        chain_id: ChainId,
        height: u64,
        timestamp: Timestamp,
    ) -> SyntheticTmBlock {
        let validators = [
            TestgenValidator::new("1").voting_power(50),
            TestgenValidator::new("2").voting_power(50),
        ];

        let header = TestgenHeader::new(&validators)
            .height(height)
            .chain_id(chain_id.as_str())
            .next_validators(&validators)
            .time(timestamp.into_tm_time().expect("Never fails"));

        let light_block = TestgenLightBlock::new_default_with_header(header)
            .generate()
            .expect("Never fails");

        SyntheticTmBlock {
            trusted_height: Height::new(chain_id.revision_number(), 1).expect("Never fails"),
            trusted_next_validators: light_block.next_validators.clone(),
            light_block,
        }
    }

    pub fn try_into_tm_block(self) -> Option<SyntheticTmBlock> {
        match self {
            HostBlock::Mock(_) => None,
            HostBlock::SyntheticTendermint(tm_block) => Some(*tm_block),
        }
    }
}

impl From<SyntheticTmBlock> for AnyConsensusState {
    fn from(light_block: SyntheticTmBlock) -> Self {
        let cs = TmConsensusState::from(light_block.header().clone());
        cs.into()
    }
}

impl From<HostBlock> for AnyConsensusState {
    fn from(any_block: HostBlock) -> Self {
        match any_block {
            HostBlock::Mock(mock_header) => MockConsensusState::new(*mock_header).into(),
            HostBlock::SyntheticTendermint(light_block) => {
                TmConsensusState::from(light_block.header().clone()).into()
            }
        }
    }
}

impl Protobuf<Any> for HostBlock {}

impl TryFrom<Any> for HostBlock {
    type Error = ClientError;

    fn try_from(_raw: Any) -> Result<Self, Self::Error> {
        todo!()
    }
}

impl From<HostBlock> for Any {
    fn from(value: HostBlock) -> Self {
        fn encode_light_block(light_block: SyntheticTmBlock) -> Vec<u8> {
            let SyntheticTmBlock {
                trusted_height,
                trusted_next_validators,
                light_block,
            } = light_block;

            RawHeader {
                signed_header: Some(light_block.signed_header.into()),
                validator_set: Some(light_block.validators.into()),
                trusted_height: Some(trusted_height.into()),
                trusted_validators: Some(trusted_next_validators.into()),
            }
            .to_vec()
        }

        match value {
            HostBlock::Mock(mock_header) => (*mock_header).into(),
            HostBlock::SyntheticTendermint(light_block) => Self {
                type_url: TENDERMINT_HEADER_TYPE_URL.to_string(),
                value: encode_light_block(*light_block),
            },
        }
    }
}
