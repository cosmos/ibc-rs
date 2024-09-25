mod conn_open_ack;
mod conn_open_confirm;
mod conn_open_init;
mod conn_open_try;

use ibc::core::commitment_types::proto::v1::MerklePrefix;
use ibc::core::connection::types::proto::v1::Counterparty as RawCounterparty;
use ibc::core::host::types::identifiers::ConnectionId;
use ibc::core::primitives::prelude::*;

pub use self::conn_open_ack::*;
pub use self::conn_open_confirm::*;
pub use self::conn_open_init::*;
pub use self::conn_open_try::*;

pub fn dummy_raw_counterparty_conn(conn_id: Option<u64>) -> RawCounterparty {
    let connection_id = match conn_id {
        Some(id) => ConnectionId::new(id).to_string(),
        None => "".to_string(),
    };
    RawCounterparty {
        client_id: "07-tendermint-0".into(),
        connection_id,
        prefix: Some(MerklePrefix {
            key_prefix: b"ibc".to_vec(),
        }),
    }
}
