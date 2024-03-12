mod conn_open_ack;
mod conn_open_confirm;
mod conn_open_init;
mod conn_open_try;

pub use conn_open_ack::*;
pub use conn_open_confirm::*;
pub use conn_open_init::*;
pub use conn_open_try::*;
use ibc::core::commitment_types::proto::v1::MerklePrefix;
use ibc::core::connection::types::proto::v1::Counterparty as RawCounterparty;
use ibc::core::host::types::identifiers::ConnectionId;
use ibc::core::primitives::prelude::*;
use typed_builder::TypedBuilder;

#[derive(TypedBuilder, Debug)]
#[builder(build_method(into = RawCounterparty))]
pub struct CounterpartyConfig {
    #[builder(default = "07-tendermint-0")]
    client_id: &'static str,
    #[builder(default = "connection-0")]
    connection_id: &'static str,
    #[builder(default = Some(MerklePrefix {
        key_prefix: b"ibc".to_vec()
    }))]
    prefix: Option<MerklePrefix>,
}

impl From<CounterpartyConfig> for RawCounterparty {
    fn from(config: CounterpartyConfig) -> Self {
        Self {
            client_id: config.client_id.to_string(),
            connection_id: config.connection_id.to_string(),
            prefix: config.prefix,
        }
    }
}

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
