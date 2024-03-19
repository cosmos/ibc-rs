use core::fmt::Debug;
use core::ops::Add;
use core::time::Duration;

use ibc::core::client::context::consensus_state::ConsensusState;
use ibc::core::client::types::Height;
use ibc::core::host::types::identifiers::ChainId;
use ibc::core::primitives::prelude::*;
use ibc::core::primitives::Timestamp;
use ibc::primitives::proto::Any;
use typed_builder::TypedBuilder;

use crate::testapp::ibc::clients::{AnyClientState, AnyConsensusState};
use crate::testapp::ibc::core::types::DEFAULT_BLOCK_TIME_SECS;
use crate::utils::year_2023;

pub mod mock;
pub mod tendermint;

pub use mock::MockHost;
pub use tendermint::TendermintHost;

#[derive(Debug, TypedBuilder)]
pub struct HostParams {
    #[builder(default = ChainId::new("mockgaia-0").expect("Never fails"))]
    pub chain_id: ChainId,
    #[builder(default = Duration::from_secs(DEFAULT_BLOCK_TIME_SECS))]
    pub block_time: Duration,
    #[builder(default = year_2023())]
    pub genesis_timestamp: Timestamp,
}

/// TestHost is a trait that defines the interface for a host blockchain.
pub trait TestHost: Debug + Sized {
    /// The type of block produced by the host.
    type Block: TestBlock;

    /// The type of client state produced by the host.
    type ClientState: Into<AnyClientState>;

    /// The type of block parameters to produce a block
    type BlockParams: Debug + Default;

    /// The type of light client parameters to produce a light client state
    type LightClientParams: Debug + Default;

    /// Create a new host with the given chain identifier.
    fn with_chain_id(chain_id: ChainId) -> Self;

    /// The chain identifier of the host.
    fn chain_id(&self) -> &ChainId;

    fn block_time(&self) -> Duration {
        Duration::from_secs(DEFAULT_BLOCK_TIME_SECS)
    }

    fn history(&self) -> Vec<Self::Block>;

    /// Accessor for a block of the local (host) chain. Returns `None` if the
    /// block at the requested height does not exist.
    fn get_block(&self, target_height: &Height) -> Option<Self::Block> {
        let target = target_height.revision_height();
        let latest = self.latest_height().revision_height();

        let history = self.history();

        // Check that the block is not too advanced, nor has it been pruned.
        if (target > latest) || (target <= latest - history.len() as u64) {
            None // Block for requested height does not exist in history.
        } else {
            let host_block = history[history.len() + target as usize - latest as usize - 1].clone();
            Some(host_block)
        }
    }

    fn latest_height(&self) -> Height {
        self.history()
            .last()
            .map(|block| block.height())
            .expect("Never fails")
    }

    /// Triggers the advancing of the host chain, by extending the history of blocks (or headers).
    fn advance_block(&mut self) {
        let history = self.history();

        let latest_block = history.last().expect("Never fails");

        let new_block = self.generate_block(
            latest_block.height().increment().revision_height(),
            latest_block
                .timestamp()
                .add(self.block_time())
                .expect("Never fails"),
            &Self::BlockParams::default(),
        );

        // History is not full yet.
        self.history().push(new_block);
    }

    fn advance_block_up_to(&mut self, target_height: Height) {
        let latest_height = self.latest_height();
        if target_height.revision_number() != latest_height.revision_number() {
            panic!("Cannot advance history of the chain to a different revision number!")
        } else if target_height.revision_height() < latest_height.revision_height() {
            panic!("Cannot rewind history of the chain to a smaller revision height!")
        } else {
            // Repeatedly advance the host chain height till we hit the desired height
            while self.latest_height().revision_height() < target_height.revision_height() {
                self.advance_block()
            }
        }
    }

    fn blocks_since(&self, old: Height) -> Option<u64> {
        let latest = self.latest_height();

        (latest.revision_number() == old.revision_number()
            && latest.revision_height() >= old.revision_height())
        .then(|| latest.revision_height() - old.revision_height())
    }

    /// Validates this context. Should be called after the context is mutated by a test.
    fn validate(&self) -> Result<(), String> {
        let history = self.history();

        // Check the content of the history.
        if !history.is_empty() {
            // Get the highest block.
            let lh = &history[history.len() - 1];
            // Check latest is properly updated with highest header height.
            if lh.height() != self.latest_height() {
                return Err("latest height is not updated".to_string());
            }
        }

        // Check that headers in the history are in sequential order.
        for i in 1..history.len() {
            let ph = &history[i - 1];
            let h = &history[i];
            if ph.height().increment() != h.height() {
                return Err("headers in history not sequential".to_string());
            }
        }
        Ok(())
    }

    /// Generate a block at the given height and timestamp, using the provided parameters.
    fn generate_block(
        &self,
        height: u64,
        timestamp: Timestamp,
        params: &Self::BlockParams,
    ) -> Self::Block;

    /// Generate a client state using the block at the given height and the provided parameters.
    fn generate_client_state(
        &self,
        latest_height: Height,
        params: &Self::LightClientParams,
    ) -> Self::ClientState;
}

/// TestBlock is a trait that defines the interface for a block produced by a host blockchain.
pub trait TestBlock: Clone + Debug {
    /// The type of header can be extracted from the block.
    type Header: TestHeader + From<Self>;

    /// Extract the header from the block.
    fn into_header(self) -> Self::Header {
        self.into()
    }

    /// The height of the block.
    fn height(&self) -> Height;

    /// The timestamp of the block.
    fn timestamp(&self) -> Timestamp;
}

/// TestHeader is a trait that defines the interface for a header produced by a host blockchain.
pub trait TestHeader: Clone + Debug + Into<Any> {
    /// The type of consensus state can be extracted from the header.
    type ConsensusState: ConsensusState + Into<AnyConsensusState> + From<Self>;

    /// The height of the block, as recorded in the header.
    fn height(&self) -> Height;

    /// The timestamp of the block, as recorded in the header.
    fn timestamp(&self) -> Timestamp;

    /// Extract the consensus state from the header.
    fn into_consensus_state(self) -> Self::ConsensusState {
        Self::ConsensusState::from(self)
    }
}
