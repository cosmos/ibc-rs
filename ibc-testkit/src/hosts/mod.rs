use core::fmt::Debug;

use ibc::core::client::types::Height;
use ibc::core::host::types::identifiers::ChainId;
use ibc::core::primitives::prelude::*;
use ibc::core::primitives::Timestamp;
use ibc::primitives::proto::Any;

use crate::testapp::ibc::clients::{AnyClientState, AnyConsensusState};

pub mod mock;
pub mod tendermint;

pub use mock::Host as MockHost;
pub use tendermint::Host as TendermintHost;

/// TestHost is a trait that defines the interface for a host blockchain.
pub trait TestHost: Debug {
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
        latest_block: &Self::Block,
        params: &Self::LightClientParams,
    ) -> Self::ClientState;
}

/// TestBlock is a trait that defines the interface for a block produced by a host blockchain.
pub trait TestBlock: Clone + Debug {
    /// The type of header can be extracted from the block.
    type Header: TestHeader + From<Self>;

    /// The height of the block.
    fn height(&self) -> Height;

    /// The timestamp of the block.
    fn timestamp(&self) -> Timestamp;

    /// Extract the header from the block.
    fn into_header(self) -> Self::Header {
        self.into()
    }
}

/// TestHeader is a trait that defines the interface for a header produced by a host blockchain.
pub trait TestHeader: Clone + Debug + Into<Any> {
    /// The type of consensus state can be extracted from the header.
    type ConsensusState: Into<AnyConsensusState> + From<Self>;

    /// The height of the block, as recorded in the header.
    fn height(&self) -> Height;

    /// The timestamp of the block, as recorded in the header.
    fn timestamp(&self) -> Timestamp;

    /// Extract the consensus state from the header.
    fn into_consensus_state(self) -> Self::ConsensusState {
        Self::ConsensusState::from(self)
    }
}
