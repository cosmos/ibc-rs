use ibc::core::client::types::Height;
use ibc::core::host::types::identifiers::ChainId;
use ibc::core::primitives::Timestamp;

use super::{TestBlock, TestHeader, TestHost};
use crate::testapp::ibc::clients::mock::client_state::MockClientState;
use crate::testapp::ibc::clients::mock::consensus_state::MockConsensusState;
use crate::testapp::ibc::clients::mock::header::MockHeader;

#[derive(Debug)]
pub struct MockHost(ChainId);

impl TestHost for MockHost {
    type Block = MockHeader;
    type BlockParams = ();
    type LightClientParams = ();
    type ClientState = MockClientState;

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
        _: &Self::BlockParams,
    ) -> Self::Block {
        MockHeader {
            height: Height::new(self.chain_id().revision_number(), height).expect("Never fails"),
            timestamp,
        }
    }

    fn generate_client_state(
        &self,
        latest_block: &Self::Block,
        _: &Self::LightClientParams,
    ) -> Self::ClientState {
        MockClientState::new(*latest_block)
    }
}
impl TestBlock for MockHeader {
    type Header = MockHeader;

    fn height(&self) -> Height {
        self.height
    }

    fn timestamp(&self) -> Timestamp {
        self.timestamp
    }
}

impl From<MockHeader> for MockConsensusState {
    fn from(block: MockHeader) -> Self {
        MockConsensusState::new(block)
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
