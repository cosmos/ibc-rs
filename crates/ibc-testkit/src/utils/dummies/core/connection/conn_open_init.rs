use ibc::core::ics03_connection::msgs::conn_open_init::MsgConnectionOpenInit;
use ibc::core::ics03_connection::version::Version;
use ibc::core::ics24_host::identifier::ClientId;
use ibc::prelude::*;
use ibc::proto::core::connection::v1::{
    MsgConnectionOpenInit as RawMsgConnectionOpenInit, Version as RawVersion,
};

use super::dummy_raw_counterparty;
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

use ibc::core::ics03_connection::connection::Counterparty;

/// Returns a new `MsgConnectionOpenInit` with dummy values.
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

/// Setter for `counterparty`. Amenable to chaining, since it consumes the input message.\
pub fn msg_conn_open_init_with_counterparty_conn_id(
    msg: MsgConnectionOpenInit,
    counterparty_conn_id: u64,
) -> MsgConnectionOpenInit {
    let counterparty = Counterparty::try_from(dummy_raw_counterparty(Some(counterparty_conn_id)))
        .expect("Never fails");
    MsgConnectionOpenInit {
        counterparty,
        ..msg
    }
}

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

/// Returns a dummy message, for testing only.
/// Other unit tests may import this if they depend on a MsgConnectionOpenInit.
pub fn dummy_raw_msg_conn_open_init() -> RawMsgConnectionOpenInit {
    RawMsgConnectionOpenInit {
        client_id: ClientId::default().to_string(),
        counterparty: Some(dummy_raw_counterparty(None)),
        version: Some(Version::default().into()),
        delay_period: 0,
        signer: dummy_bech32_account(),
    }
}
