use ibc::core::connection::types::msgs::MsgConnectionOpenInit;
use ibc::core::connection::types::proto::v1::{
    MsgConnectionOpenInit as RawMsgConnectionOpenInit, Version as RawVersion,
};
use ibc::core::connection::types::version::Version;
use ibc::core::connection::types::Counterparty;
use ibc::core::host::types::identifiers::ClientId;
use ibc::core::primitives::prelude::*;

use super::dummy_raw_counterparty_conn;
use crate::utils::dummies::core::signer::dummy_bech32_account;

pub fn raw_version_from_identifier(identifier: &str) -> Option<RawVersion> {
    if identifier.is_empty() {
        return None;
    }

    Some(RawVersion {
        identifier: identifier.to_string(),
        features: vec![],
    })
}

/// Returns a dummy `MsgConnectionOpenInit` for testing purposes only!
pub fn dummy_msg_conn_open_init() -> MsgConnectionOpenInit {
    MsgConnectionOpenInit::try_from(dummy_raw_msg_conn_open_init()).expect("Never fails")
}

/// Setter for `client_id`. Amenable to chaining, since it consumes the input message.
pub fn dummy_msg_conn_open_init_with_client_id(
    msg: MsgConnectionOpenInit,
    client_id: ClientId,
) -> MsgConnectionOpenInit {
    MsgConnectionOpenInit {
        client_id_on_a: client_id,
        ..msg
    }
}

/// Setter for `counterparty`. Amenable to chaining, since it consumes the input message.
pub fn msg_conn_open_init_with_counterparty_conn_id(
    msg: MsgConnectionOpenInit,
    counterparty_conn_id: u64,
) -> MsgConnectionOpenInit {
    let counterparty =
        Counterparty::try_from(dummy_raw_counterparty_conn(Some(counterparty_conn_id)))
            .expect("Never fails");
    MsgConnectionOpenInit {
        counterparty,
        ..msg
    }
}

/// Setter for the connection `version`
pub fn msg_conn_open_with_version(
    msg: MsgConnectionOpenInit,
    identifier: Option<&str>,
) -> MsgConnectionOpenInit {
    let version = match identifier {
        Some(v) => Version::try_from(RawVersion {
            identifier: v.to_string(),
            features: vec![],
        })
        .expect("could not create version from identifier")
        .into(),
        None => None,
    };
    MsgConnectionOpenInit { version, ..msg }
}

/// Returns a dummy `RawMsgConnectionOpenInit`, for testing purposes only!
pub fn dummy_raw_msg_conn_open_init() -> RawMsgConnectionOpenInit {
    RawMsgConnectionOpenInit {
        client_id: ClientId::default().to_string(),
        counterparty: Some(dummy_raw_counterparty_conn(None)),
        version: Some(Version::default().into()),
        delay_period: 0,
        signer: dummy_bech32_account(),
    }
}
