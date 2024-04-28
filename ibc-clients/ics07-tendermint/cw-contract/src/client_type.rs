use ibc_client_cw::api::ClientType;
use ibc_client_tendermint::client_state::ClientState;
use ibc_client_tendermint::consensus_state::ConsensusState;

/// A unit struct that represents the Tendermint client type.
#[derive(Clone, Debug)]
pub struct TendermintClient;

impl<'a> ClientType<'a> for TendermintClient {
    type ClientState = ClientState;
    type ConsensusState = ConsensusState;
}
