use ibc::core::client::types::proto::v1::MsgUpdateClient as RawMsgUpdateClient;
use ibc::primitives::proto::Any;

use crate::fixtures::clients::tendermint::dummy_ics07_header;
use crate::fixtures::core::signer::dummy_bech32_account;

/// Returns a dummy `RawMsgUpdateClient`, for testing purposes only!
pub fn dummy_raw_msg_update_client() -> RawMsgUpdateClient {
    let client_id = "07-tendermint-0".parse().expect("no error");

    let tm_header = dummy_ics07_header();

    RawMsgUpdateClient {
        client_id,
        client_message: Some(Any::from(tm_header)),
        signer: dummy_bech32_account(),
    }
}

#[cfg(test)]
mod tests {
    use ibc::core::client::types::msgs::MsgUpdateClient;

    use super::*;

    #[test]
    fn msg_update_client_serialization() {
        let raw = dummy_raw_msg_update_client();
        let msg = MsgUpdateClient::try_from(raw.clone()).unwrap();
        let raw_back = RawMsgUpdateClient::from(msg.clone());
        let msg_back = MsgUpdateClient::try_from(raw_back.clone()).unwrap();
        assert_eq!(msg, msg_back);
        assert_eq!(raw, raw_back);
    }
}
