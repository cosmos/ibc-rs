use core::fmt::Debug;

use ibc::core::client::types::Height;
use ibc::core::host::types::identifiers::ChainId;
use ibc::core::primitives::prelude::*;
use ibc::core::primitives::Timestamp;
use ibc::primitives::proto::Any;

use crate::testapp::ibc::clients::{AnyClientState, AnyConsensusState};

pub mod mockhost;
pub mod tenderminthost;

pub trait TestHost: Debug {
    // produced block on chain
    type Block: TestBlock;

    type ClientState: Into<AnyClientState>;

    // parameters for block
    type BlockParams: Debug + Default;

    // parameters for light client
    type LightClientParams: Debug + Default;

    fn new(chain_id: ChainId) -> Self;

    fn chain_id(&self) -> &ChainId;

    fn generate_block(
        &self,
        height: u64,
        timestamp: Timestamp,
        params: &Self::BlockParams,
    ) -> Self::Block;

    fn generate_client_state(
        &self,
        latest_block: &Self::Block,
        params: &Self::LightClientParams,
    ) -> Self::ClientState;
}

pub trait TestBlock: Clone + Debug {
    type Header: TestHeader + From<Self>;

    fn height(&self) -> Height;
    fn timestamp(&self) -> Timestamp;

    fn into_header(self) -> Self::Header {
        self.into()
    }
}

// relayed by relayer
pub trait TestHeader: Clone + Debug + Into<Any> {
    type ConsensusState: Into<AnyConsensusState> + From<Self>;

    fn height(&self) -> Height;
    fn timestamp(&self) -> Timestamp;

    fn into_consensus_state(self) -> Self::ConsensusState {
        Self::ConsensusState::from(self)
    }
}
