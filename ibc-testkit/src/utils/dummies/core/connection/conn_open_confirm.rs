use alloc::string::ToString;

use ibc::core::client::types::proto::v1::Height;
use ibc::core::connection::types::msgs::MsgConnectionOpenConfirm;
use ibc::core::connection::types::proto::v1::MsgConnectionOpenConfirm as RawMsgConnectionOpenConfirm;

use crate::utils::dummies::core::channel::dummy_proof;
use crate::utils::dummies::core::signer::dummy_bech32_account;

/// Returns a dummy `MsgConnectionOpenConfirm` for testing purposes only!
pub fn dummy_conn_open_confirm() -> MsgConnectionOpenConfirm {
    MsgConnectionOpenConfirm::try_from(dummy_raw_msg_conn_open_confirm()).expect("Never fails")
}

/// Returns a dummy `RawMsgConnectionOpenConfirm` for testing purposes only!
pub fn dummy_raw_msg_conn_open_confirm() -> RawMsgConnectionOpenConfirm {
    RawMsgConnectionOpenConfirm {
        connection_id: "srcconnection".to_string(),
        proof_ack: dummy_proof(),
        proof_height: Some(Height {
            revision_number: 0,
            revision_height: 10,
        }),
        signer: dummy_bech32_account(),
    }
}
