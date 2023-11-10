use alloc::string::ToString;

use ibc::core::ics03_connection::msgs::conn_open_confirm::MsgConnectionOpenConfirm;
use ibc::proto::core::client::v1::Height;
use ibc::proto::core::connection::v1::MsgConnectionOpenConfirm as RawMsgConnectionOpenConfirm;

use crate::utils::dummies::core::signer::{dummy_bech32_account, dummy_proof};

/// Returns a new `MsgConnectionOpenConfirm` with dummy values.
pub fn dummy_conn_open_confirm() -> MsgConnectionOpenConfirm {
    MsgConnectionOpenConfirm::try_from(dummy_raw_msg_conn_open_confirm()).expect("Never fails")
}

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
