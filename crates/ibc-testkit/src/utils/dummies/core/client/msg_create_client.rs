use ibc::clients::ics07_tendermint::consensus_state::ConsensusState as TmConsensusState;
use ibc::proto::core::client::v1::MsgCreateClient;
use ibc::proto::Any;

use crate::utils::dummies::clients::tendermint::{
    dummy_tendermint_header, dummy_tm_client_state_from_header,
};
use crate::utils::dummies::core::signer::dummy_bech32_account;

/// Returns a dummy `RawMsgCreateClient`, for testing purposes only!
pub fn dummy_raw_msg_create_client() -> MsgCreateClient {
    let tm_header = dummy_tendermint_header();

    let tm_client_state = dummy_tm_client_state_from_header(tm_header.clone());

    MsgCreateClient {
        client_state: Some(Any::from(tm_client_state)),
        consensus_state: Some(Any::from(TmConsensusState::from(tm_header))),
        signer: dummy_bech32_account(),
    }
}
