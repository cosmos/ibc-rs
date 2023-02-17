use crate::core::ics23_commitment::commitment::CommitmentProofBytes;
use crate::{prelude::*, Height};

use ibc_proto::protobuf::Protobuf;

use ibc_proto::ibc::core::connection::v1::MsgConnectionOpenConfirm as RawMsgConnectionOpenConfirm;

use crate::core::ics03_connection::error::ConnectionError;
use crate::core::ics24_host::identifier::ConnectionId;
use crate::signer::Signer;
use crate::tx_msg::Msg;

pub const TYPE_URL: &str = "/ibc.core.connection.v1.MsgConnectionOpenConfirm";

/// Per our convention, this message is sent to chain B.
/// The handler will check proofs of chain A.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MsgConnectionOpenConfirm {
    /// ConnectionId that chain B has chosen for it's ConnectionEnd
    pub conn_id_on_b: ConnectionId,
    /// proof of ConnectionEnd stored on Chain A during ConnOpenInit
    pub proof_conn_end_on_a: CommitmentProofBytes,
    /// Height at which `proof_conn_end_on_a` in this message was taken
    pub proof_height_on_a: Height,
    pub signer: Signer,
}

impl Msg for MsgConnectionOpenConfirm {
    type Raw = RawMsgConnectionOpenConfirm;

    fn type_url(&self) -> String {
        TYPE_URL.to_string()
    }
}

impl Protobuf<RawMsgConnectionOpenConfirm> for MsgConnectionOpenConfirm {}

impl TryFrom<RawMsgConnectionOpenConfirm> for MsgConnectionOpenConfirm {
    type Error = ConnectionError;

    fn try_from(msg: RawMsgConnectionOpenConfirm) -> Result<Self, Self::Error> {
        Ok(Self {
            conn_id_on_b: msg
                .connection_id
                .parse()
                .map_err(ConnectionError::InvalidIdentifier)?,
            proof_conn_end_on_a: msg
                .proof_ack
                .try_into()
                .map_err(|_| ConnectionError::InvalidProof)?,
            proof_height_on_a: msg
                .proof_height
                .and_then(|raw_height| raw_height.try_into().ok())
                .ok_or(ConnectionError::MissingProofHeight)?,
            signer: msg.signer.parse().map_err(ConnectionError::Signer)?,
        })
    }
}

impl From<MsgConnectionOpenConfirm> for RawMsgConnectionOpenConfirm {
    fn from(msg: MsgConnectionOpenConfirm) -> Self {
        RawMsgConnectionOpenConfirm {
            connection_id: msg.conn_id_on_b.as_str().to_string(),
            proof_ack: msg.proof_conn_end_on_a.into(),
            proof_height: Some(msg.proof_height_on_a.into()),
            signer: msg.signer.to_string(),
        }
    }
}

#[cfg(test)]
pub mod test_util {
    use super::MsgConnectionOpenConfirm;
    use crate::prelude::*;
    use ibc_proto::ibc::core::client::v1::Height;
    use ibc_proto::ibc::core::connection::v1::MsgConnectionOpenConfirm as RawMsgConnectionOpenConfirm;

    use crate::test_utils::{get_dummy_bech32_account, get_dummy_proof};

    /// Testing-specific helper methods.
    impl MsgConnectionOpenConfirm {
        /// Returns a new `MsgConnectionOpenConfirm` with dummy values.
        pub fn new_dummy() -> Self {
            MsgConnectionOpenConfirm::try_from(get_dummy_raw_msg_conn_open_confirm()).unwrap()
        }
    }

    pub fn get_dummy_raw_msg_conn_open_confirm() -> RawMsgConnectionOpenConfirm {
        RawMsgConnectionOpenConfirm {
            connection_id: "srcconnection".to_string(),
            proof_ack: get_dummy_proof(),
            proof_height: Some(Height {
                revision_number: 0,
                revision_height: 10,
            }),
            signer: get_dummy_bech32_account(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    use test_log::test;

    use ibc_proto::ibc::core::client::v1::Height;
    use ibc_proto::ibc::core::connection::v1::MsgConnectionOpenConfirm as RawMsgConnectionOpenConfirm;

    use crate::core::ics03_connection::msgs::conn_open_confirm::test_util::get_dummy_raw_msg_conn_open_confirm;
    use crate::core::ics03_connection::msgs::conn_open_confirm::MsgConnectionOpenConfirm;

    #[test]
    fn parse_connection_open_confirm_msg() {
        #[derive(Clone, Debug, PartialEq)]
        struct Test {
            name: String,
            raw: RawMsgConnectionOpenConfirm,
            want_pass: bool,
        }

        let default_ack_msg = get_dummy_raw_msg_conn_open_confirm();
        let tests: Vec<Test> = vec![
            Test {
                name: "Good parameters".to_string(),
                raw: default_ack_msg.clone(),
                want_pass: true,
            },
            Test {
                name: "Bad connection id, non-alpha".to_string(),
                raw: RawMsgConnectionOpenConfirm {
                    connection_id: "con007".to_string(),
                    ..default_ack_msg.clone()
                },
                want_pass: false,
            },
            Test {
                name: "Bad proof height, height is 0".to_string(),
                raw: RawMsgConnectionOpenConfirm {
                    proof_height: Some(Height {
                        revision_number: 1,
                        revision_height: 0,
                    }),
                    ..default_ack_msg
                },
                want_pass: false,
            },
        ]
        .into_iter()
        .collect();

        for test in tests {
            let msg = MsgConnectionOpenConfirm::try_from(test.raw.clone());

            assert_eq!(
                test.want_pass,
                msg.is_ok(),
                "MsgConnOpenTry::new failed for test {}, \nmsg {:?} with error {:?}",
                test.name,
                test.raw,
                msg.err(),
            );
        }
    }

    #[test]
    fn to_and_from() {
        let raw = get_dummy_raw_msg_conn_open_confirm();
        let msg = MsgConnectionOpenConfirm::try_from(raw.clone()).unwrap();
        let raw_back = RawMsgConnectionOpenConfirm::from(msg.clone());
        let msg_back = MsgConnectionOpenConfirm::try_from(raw_back.clone()).unwrap();
        assert_eq!(raw, raw_back);
        assert_eq!(msg, msg_back);
    }
}
