use ibc::proto::core::client::v1::MsgUpdateClient;
use ibc::proto::Any;

use crate::utils::dummies::clients::tendermint::dummy_ics07_header;
use crate::utils::dummies::core::signer::dummy_bech32_account;

pub fn dummy_raw_msg_update_client() -> MsgUpdateClient {
    let client_id = "tendermint".parse().unwrap();
    let tm_header = dummy_ics07_header();

    MsgUpdateClient {
        client_id,
        client_message: Some(Any::from(tm_header)),
        signer: dummy_bech32_account(),
    }
}
