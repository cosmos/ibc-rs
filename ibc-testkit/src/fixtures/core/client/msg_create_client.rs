use ibc::clients::tendermint::types::ConsensusState as TmConsensusState;
use ibc::core::client::types::proto::v1::MsgCreateClient as RawMsgCreateClient;
use ibc::primitives::proto::Any;

use crate::fixtures::clients::tendermint::{
    dummy_tendermint_header, dummy_tm_client_state_from_header,
};
use crate::fixtures::core::signer::dummy_bech32_account;

/// Returns a dummy `RawMsgCreateClient`, for testing purposes only!
pub fn dummy_raw_msg_create_client() -> RawMsgCreateClient {
    let tm_header = dummy_tendermint_header();

    let tm_client_state = dummy_tm_client_state_from_header(tm_header.clone());

    RawMsgCreateClient {
        client_state: Some(Any::from(tm_client_state)),
        consensus_state: Some(Any::from(TmConsensusState::from(tm_header))),
        signer: dummy_bech32_account(),
    }
}

#[cfg(test)]
mod tests {
    use ibc::core::client::types::msgs::MsgCreateClient;

    use super::*;

    #[test]
    fn msg_create_client_serialization() {
        let raw = dummy_raw_msg_create_client();
        let msg = MsgCreateClient::try_from(raw.clone()).unwrap();
        let raw_back = RawMsgCreateClient::from(msg.clone());
        let msg_back = MsgCreateClient::try_from(raw_back.clone()).unwrap();
        assert_eq!(msg, msg_back);
        assert_eq!(raw, raw_back);
    }
}
