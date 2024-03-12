use core::time::Duration;

use ibc_core_host_types::identifiers::ClientId;
use ibc_primitives::prelude::*;
use ibc_primitives::Signer;
use ibc_proto::ibc::core::connection::v1::MsgConnectionOpenInit as RawMsgConnectionOpenInit;
use ibc_proto::Protobuf;

use crate::connection::Counterparty;
use crate::error::ConnectionError;
use crate::version::Version;

pub const CONN_OPEN_INIT_TYPE_URL: &str = "/ibc.core.connection.v1.MsgConnectionOpenInit";

/// Per our convention, this message is sent to chain A.
/// The handler will check proofs of chain B.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct MsgConnectionOpenInit {
    /// ClientId on chain A that the connection is being opened for
    pub client_id_on_a: ClientId,
    pub counterparty: Counterparty,
    pub version: Option<Version>,
    pub delay_period: Duration,
    pub signer: Signer,
}

/// This module encapsulates the workarounds we need to do to implement
/// `BorshSerialize` and `BorshDeserialize` on `MsgConnectionOpenInit`
#[cfg(feature = "borsh")]
mod borsh_impls {
    use borsh::maybestd::io::{self, Read};
    use borsh::{BorshDeserialize, BorshSerialize};

    use super::*;

    #[derive(BorshSerialize, BorshDeserialize)]
    pub struct InnerMsgConnectionOpenInit {
        /// ClientId on chain A that the connection is being opened for
        pub client_id_on_a: ClientId,
        pub counterparty: Counterparty,
        pub version: Option<Version>,
        pub delay_period_nanos: u64,
        pub signer: Signer,
    }

    impl BorshSerialize for MsgConnectionOpenInit {
        fn serialize<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
            let delay_period_nanos: u64 =
                self.delay_period.as_nanos().try_into().map_err(|_| {
                    io::Error::new(
                        io::ErrorKind::Other,
                        format!("Duration too long: {} nanos", self.delay_period.as_nanos()),
                    )
                })?;

            let inner = InnerMsgConnectionOpenInit {
                client_id_on_a: self.client_id_on_a.clone(),
                counterparty: self.counterparty.clone(),
                version: self.version.clone(),
                delay_period_nanos,
                signer: self.signer.clone(),
            };

            inner.serialize(writer)
        }
    }

    impl BorshDeserialize for MsgConnectionOpenInit {
        fn deserialize_reader<R: Read>(reader: &mut R) -> io::Result<Self> {
            let inner = InnerMsgConnectionOpenInit::deserialize_reader(reader)?;

            Ok(MsgConnectionOpenInit {
                client_id_on_a: inner.client_id_on_a,
                counterparty: inner.counterparty,
                version: inner.version,
                delay_period: Duration::from_nanos(inner.delay_period_nanos),
                signer: inner.signer,
            })
        }
    }
}

impl Protobuf<RawMsgConnectionOpenInit> for MsgConnectionOpenInit {}

impl TryFrom<RawMsgConnectionOpenInit> for MsgConnectionOpenInit {
    type Error = ConnectionError;

    fn try_from(msg: RawMsgConnectionOpenInit) -> Result<Self, Self::Error> {
        let counterparty: Counterparty = msg
            .counterparty
            .ok_or(ConnectionError::MissingCounterparty)?
            .try_into()?;

        counterparty.verify_empty_connection_id()?;

        Ok(Self {
            client_id_on_a: msg
                .client_id
                .parse()
                .map_err(ConnectionError::InvalidIdentifier)?,
            counterparty,
            version: msg.version.map(TryInto::try_into).transpose()?,
            delay_period: Duration::from_nanos(msg.delay_period),
            signer: msg.signer.into(),
        })
    }
}

impl From<MsgConnectionOpenInit> for RawMsgConnectionOpenInit {
    fn from(ics_msg: MsgConnectionOpenInit) -> Self {
        RawMsgConnectionOpenInit {
            client_id: ics_msg.client_id_on_a.as_str().to_string(),
            counterparty: Some(ics_msg.counterparty.into()),
            version: ics_msg.version.map(Into::into),
            delay_period: ics_msg.delay_period.as_nanos() as u64,
            signer: ics_msg.signer.to_string(),
        }
    }
}
