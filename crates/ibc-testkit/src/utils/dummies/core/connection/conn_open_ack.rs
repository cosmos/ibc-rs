use ibc::core::ics02_client::height::Height;
use ibc::core::ics03_connection::msgs::conn_open_ack::MsgConnectionOpenAck;
use ibc::core::ics03_connection::version::Version;
use ibc::core::ics24_host::identifier::ConnectionId;
use ibc::prelude::*;
use ibc::proto::core::client::v1::Height as RawHeight;
use ibc::proto::core::connection::v1::MsgConnectionOpenAck as RawMsgConnectionOpenAck;

use crate::testapp::ibc::clients::mock::client_state::MockClientState;
use crate::testapp::ibc::clients::mock::header::MockHeader;
use crate::utils::dummies::core::signer::{dummy_bech32_account, dummy_proof};

/// Returns a new `MsgConnectionOpenAck` with dummy values.
pub fn dummy_msg_conn_open_ack(proof_height: u64, consensus_height: u64) -> MsgConnectionOpenAck {
    MsgConnectionOpenAck::try_from(dummy_raw_msg_conn_open_ack(proof_height, consensus_height))
        .expect("Never fails")
}

pub fn dummy_raw_msg_conn_open_ack(
    proof_height: u64,
    consensus_height: u64,
) -> RawMsgConnectionOpenAck {
    let client_state_height = Height::new(0, consensus_height).expect("invalid height");
    RawMsgConnectionOpenAck {
        connection_id: ConnectionId::new(0).to_string(),
        counterparty_connection_id: ConnectionId::new(1).to_string(),
        proof_try: dummy_proof(),
        proof_height: Some(RawHeight {
            revision_number: 0,
            revision_height: proof_height,
        }),
        proof_consensus: dummy_proof(),
        consensus_height: Some(RawHeight {
            revision_number: 0,
            revision_height: consensus_height,
        }),
        client_state: Some(MockClientState::new(MockHeader::new(client_state_height)).into()),
        proof_client: dummy_proof(),
        version: Some(Version::default().into()),
        signer: dummy_bech32_account(),
        host_consensus_state_proof: vec![],
    }
}
