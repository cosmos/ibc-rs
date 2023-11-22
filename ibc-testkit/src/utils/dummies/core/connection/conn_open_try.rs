use ibc::core::client::types::proto::v1::Height as RawHeight;
use ibc::core::client::types::Height;
use ibc::core::connection::types::msgs::MsgConnectionOpenTry;
use ibc::core::connection::types::proto::v1::MsgConnectionOpenTry as RawMsgConnectionOpenTry;
use ibc::core::connection::types::version::get_compatible_versions;
use ibc::core::host::types::identifiers::{ClientId, ConnectionId};
use ibc::core::primitives::prelude::*;

use super::dummy_raw_counterparty_conn;
use crate::testapp::ibc::clients::mock::client_state::MockClientState;
use crate::testapp::ibc::clients::mock::header::MockHeader;
use crate::utils::dummies::core::channel::dummy_proof;
use crate::utils::dummies::core::signer::dummy_bech32_account;

/// Returns a dummy `MsgConnectionOpenTry` for testing purposes only!
pub fn dummy_msg_conn_open_try(proof_height: u64, consensus_height: u64) -> MsgConnectionOpenTry {
    MsgConnectionOpenTry::try_from(dummy_raw_msg_conn_open_try(proof_height, consensus_height))
        .expect("Never fails")
}
/// Setter for the `client_id`
pub fn msg_conn_open_try_with_client_id(
    msg: MsgConnectionOpenTry,
    client_id: ClientId,
) -> MsgConnectionOpenTry {
    MsgConnectionOpenTry {
        client_id_on_b: client_id,
        ..msg
    }
}

/// Returns a dummy `RawMsgConnectionOpenTry` with parametrized heights. The parameter
/// `proof_height` represents the height, on the source chain, at which this chain produced the
/// proof. Parameter `consensus_height` represents the height of destination chain which a
/// client on the source chain stores.
pub fn dummy_raw_msg_conn_open_try(
    proof_height: u64,
    consensus_height: u64,
) -> RawMsgConnectionOpenTry {
    let client_state_height = Height::new(0, consensus_height).expect("could not create height");

    #[allow(deprecated)]
    RawMsgConnectionOpenTry {
        client_id: ClientId::default().to_string(),
        previous_connection_id: ConnectionId::default().to_string(),
        client_state: Some(MockClientState::new(MockHeader::new(client_state_height)).into()),
        counterparty: Some(dummy_raw_counterparty_conn(Some(0))),
        delay_period: 0,
        counterparty_versions: get_compatible_versions()
            .iter()
            .map(|v| v.clone().into())
            .collect(),
        proof_init: dummy_proof(),
        proof_height: Some(RawHeight {
            revision_number: 0,
            revision_height: proof_height,
        }),
        proof_consensus: dummy_proof(),
        consensus_height: Some(RawHeight {
            revision_number: 0,
            revision_height: consensus_height,
        }),
        proof_client: dummy_proof(),
        signer: dummy_bech32_account(),
        host_consensus_state_proof: vec![],
    }
}
