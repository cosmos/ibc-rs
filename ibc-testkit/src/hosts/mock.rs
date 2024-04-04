use alloc::collections::VecDeque;
use alloc::vec::Vec;

use ibc::core::client::context::ClientValidationContext;
use ibc::core::client::types::Height;
use ibc::core::host::types::identifiers::{ChainId, ClientId};
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
    pub history: VecDeque<MockHeader>,
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

    fn history(&self) -> &VecDeque<Self::Block> {
        &self.history
    }

    fn push_block(&mut self, block: Self::Block) {
        self.history.push_back(block);
    }

    fn prune_block_till(&mut self, height: &Height) {
        while let Some(block) = self.history.front() {
            if &block.height() <= height {
                self.history.pop_front();
            } else {
                break;
            }
        }
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

    fn header_params<C: ClientValidationContext>(&self, _: &ClientId, _: &C) {}
}

impl TestBlock for MockHeader {
    type Header = Self;
    type HeaderParams = ();

    fn height(&self) -> Height {
        self.height
    }

    fn timestamp(&self) -> Timestamp {
        self.timestamp
    }

    fn into_header_with_params(self, _: &Self::HeaderParams) -> Self::Header {
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
