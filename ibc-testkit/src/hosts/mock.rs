use alloc::collections::VecDeque;
use alloc::sync::Arc;
use core::time::Duration;
use std::sync::Mutex;

use ibc::core::client::types::Height;
use ibc::core::host::types::identifiers::ChainId;
use ibc::core::primitives::Timestamp;

use super::{HostParams, TestBlock, TestHeader, TestHost};
use crate::testapp::ibc::clients::mock::client_state::MockClientState;
use crate::testapp::ibc::clients::mock::consensus_state::MockConsensusState;
use crate::testapp::ibc::clients::mock::header::MockHeader;

#[derive(Debug)]
pub struct MockHost {
    pub chain_id: ChainId,
    pub block_time: Duration,
    pub genesis_timestamp: Timestamp,

    /// The chain of blocks underlying this context.
    history: Arc<Mutex<VecDeque<MockHeader>>>,
}

impl TestHost for MockHost {
    type Block = MockHeader;
    type BlockParams = ();
    type LightClientParams = ();
    type ClientState = MockClientState;

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

            history: Arc::new(Mutex::new(VecDeque::new())),
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
            .back()
            .copied()
            .expect("Never fails")
    }

    fn get_block(&self, target_height: &Height) -> Option<Self::Block> {
        self.history
            .lock()
            .expect("lock")
            .get(target_height.revision_height() as usize - 1)
            .copied() // indexed from 1
    }

    fn push_block(&self, block: Self::Block) {
        self.history.lock().expect("lock").push_back(block);
    }

    fn prune_block_till(&self, height: &Height) {
        let mut history = self.history.lock().expect("lock");

        while let Some(block) = history.front() {
            if &block.height() <= height {
                history.pop_front();
            } else {
                break;
            }
        }
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
        latest_height: Height,
        _: &Self::LightClientParams,
    ) -> Self::ClientState {
        MockClientState::new(self.get_block(&latest_height).expect("height exists"))
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
