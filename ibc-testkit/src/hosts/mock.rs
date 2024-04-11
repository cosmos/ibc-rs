use alloc::vec::Vec;

use ibc::core::client::types::Height;
use ibc::core::host::types::identifiers::ChainId;
use ibc::core::primitives::Timestamp;
use typed_builder::TypedBuilder;

use super::{TestBlock, TestHeader, TestHost};
use crate::testapp::ibc::clients::mock::client_state::MockClientState;
use crate::testapp::ibc::clients::mock::consensus_state::MockConsensusState;
use crate::testapp::ibc::clients::mock::header::MockHeader;

#[derive(TypedBuilder, Debug)]
pub struct MockHost {
    /// Unique identifier for the chain.
    #[builder(default = ChainId::new("mock-0").expect("Never fails"))]
    pub chain_id: ChainId,
    /// The chain of blocks underlying this context.
    #[builder(default)]
    pub history: Vec<MockHeader>,
}

impl Default for MockHost {
    fn default() -> Self {
        Self::builder().build()
    }
}

impl TestHost for MockHost {
    type Block = MockHeader;
    type ClientState = MockClientState;
    type BlockParams = ();
    type LightClientParams = ();

    fn history(&self) -> &Vec<Self::Block> {
        &self.history
    }

    fn push_block(&mut self, block: Self::Block) {
        self.history.push(block);
    }

    fn generate_block(
        &self,
        _: Vec<u8>,
        height: u64,
        timestamp: Timestamp,
        _: &Self::BlockParams,
    ) -> Self::Block {
        MockHeader {
            height: Height::new(self.chain_id.revision_number(), height).expect("Never fails"),
            timestamp,
        }
    }

    fn generate_client_state(
        &self,
        latest_height: &Height,
        _: &Self::LightClientParams,
    ) -> Self::ClientState {
        MockClientState::new(self.get_block(latest_height).expect("height exists"))
    }
}

impl TestBlock for MockHeader {
    type Header = Self;

    fn height(&self) -> Height {
        self.height
    }

    fn timestamp(&self) -> Timestamp {
        self.timestamp
    }

    fn into_header_with_trusted(self, _: &Self) -> Self::Header {
        self
    }
}

impl From<MockHeader> for MockConsensusState {
    fn from(block: MockHeader) -> Self {
        Self::new(block)
    }
}

impl TestHeader for MockHeader {
    type ConsensusState = MockConsensusState;

    fn height(&self) -> Height {
        self.height
    }

    fn timestamp(&self) -> Timestamp {
        self.timestamp
    }
}
