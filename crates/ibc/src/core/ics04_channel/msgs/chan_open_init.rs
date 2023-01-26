use crate::core::ics04_channel::channel::ChannelEnd;
use crate::core::ics04_channel::channel::Counterparty;
use crate::core::ics04_channel::channel::{Order, State};
use crate::core::ics04_channel::error::ChannelError;
use crate::core::ics04_channel::Version;
use crate::core::ics24_host::identifier::{ConnectionId, PortId};
use crate::prelude::*;
use crate::signer::Signer;
use crate::tx_msg::Msg;

use ibc_proto::ibc::core::channel::v1::MsgChannelOpenInit as RawMsgChannelOpenInit;
use ibc_proto::protobuf::Protobuf;

pub const TYPE_URL: &str = "/ibc.core.channel.v1.MsgChannelOpenInit";

///
/// Message definition for the first step in the channel open handshake (`ChanOpenInit` datagram).
/// Per our convention, this message is sent to chain A.
///
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MsgChannelOpenInit {
    pub port_id_on_a: PortId,
    pub connection_hops_on_a: Vec<ConnectionId>,
    pub port_id_on_b: PortId,
    pub ordering: Order,
    pub signer: Signer,
    /// Allow a relayer to specify a particular version by providing a non-empty version string
    pub version_proposal: Version,
}

impl Msg for MsgChannelOpenInit {
    type Raw = RawMsgChannelOpenInit;

    fn type_url(&self) -> String {
        TYPE_URL.to_string()
    }
}

impl Protobuf<RawMsgChannelOpenInit> for MsgChannelOpenInit {}

impl TryFrom<RawMsgChannelOpenInit> for MsgChannelOpenInit {
    type Error = ChannelError;

    fn try_from(raw_msg: RawMsgChannelOpenInit) -> Result<Self, Self::Error> {
        let chan_end_on_a: ChannelEnd = raw_msg
            .channel
            .ok_or(ChannelError::MissingChannel)?
            .try_into()?;
        Ok(MsgChannelOpenInit {
            port_id_on_a: raw_msg.port_id.parse().map_err(ChannelError::Identifier)?,
            connection_hops_on_a: chan_end_on_a.connection_hops,
            port_id_on_b: chan_end_on_a.remote.port_id,
            ordering: chan_end_on_a.ordering,
            signer: raw_msg.signer.parse().map_err(ChannelError::Signer)?,
            version_proposal: chan_end_on_a.version,
        })
    }
}
impl From<MsgChannelOpenInit> for RawMsgChannelOpenInit {
    fn from(domain_msg: MsgChannelOpenInit) -> Self {
        let chan_end_on_a = ChannelEnd::new(
            State::Init,
            domain_msg.ordering,
            Counterparty::new(domain_msg.port_id_on_b, None),
            domain_msg.connection_hops_on_a,
            domain_msg.version_proposal,
        );
        RawMsgChannelOpenInit {
            port_id: domain_msg.port_id_on_a.to_string(),
            channel: Some(chan_end_on_a.into()),
            signer: domain_msg.signer.to_string(),
        }
    }
}

#[cfg(test)]
pub mod test_util {
    use crate::prelude::*;
    use ibc_proto::ibc::core::channel::v1::MsgChannelOpenInit as RawMsgChannelOpenInit;

    use crate::core::ics04_channel::channel::test_util::get_dummy_raw_channel_end;
    use crate::core::ics24_host::identifier::PortId;
    use crate::test_utils::get_dummy_bech32_account;

    /// Returns a dummy `RawMsgChannelOpenInit`, for testing only!
    pub fn get_dummy_raw_msg_chan_open_init(
        counterparty_channel_id: Option<u64>,
    ) -> RawMsgChannelOpenInit {
        RawMsgChannelOpenInit {
            port_id: PortId::default().to_string(),
            channel: Some(get_dummy_raw_channel_end(counterparty_channel_id)),
            signer: get_dummy_bech32_account(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::core::ics04_channel::msgs::chan_open_init::test_util::get_dummy_raw_msg_chan_open_init;
    use crate::core::ics04_channel::msgs::chan_open_init::MsgChannelOpenInit;
    use crate::prelude::*;

    use ibc_proto::ibc::core::channel::v1::MsgChannelOpenInit as RawMsgChannelOpenInit;
    use test_log::test;

    #[test]
    fn channel_open_init_from_raw() {
        struct Test {
            name: String,
            raw: RawMsgChannelOpenInit,
            want_pass: bool,
        }

        let default_raw_init_msg = get_dummy_raw_msg_chan_open_init(None);

        let tests: Vec<Test> = vec![
            Test {
                name: "Good parameters".to_string(),
                raw: default_raw_init_msg.clone(),
                want_pass: true,
            },
            Test {
                name: "Incorrect port identifier, slash (separator) prohibited".to_string(),
                raw: RawMsgChannelOpenInit {
                    port_id: "p34/".to_string(),
                    ..default_raw_init_msg.clone()
                },
                want_pass: false,
            },
            Test {
                name: "Missing channel".to_string(),
                raw: RawMsgChannelOpenInit {
                    channel: None,
                    ..default_raw_init_msg
                },
                want_pass: false,
            },
        ]
        .into_iter()
        .collect();

        for test in tests {
            let res_msg = MsgChannelOpenInit::try_from(test.raw.clone());

            assert_eq!(
                test.want_pass,
                res_msg.is_ok(),
                "MsgChanOpenInit::try_from failed for test {}, \nraw msg {:?} with error {:?}",
                test.name,
                test.raw,
                res_msg.err(),
            );
        }
    }

    #[test]
    fn to_and_from() {
        // Check if raw and domain types are equal after conversions
        let raw = get_dummy_raw_msg_chan_open_init(None);
        let msg = MsgChannelOpenInit::try_from(raw.clone()).unwrap();
        let raw_back = RawMsgChannelOpenInit::from(msg.clone());
        let msg_back = MsgChannelOpenInit::try_from(raw_back.clone()).unwrap();
        assert_eq!(raw, raw_back);
        assert_eq!(msg, msg_back);

        // Check if handler sets counterparty channel id to `None`
        // in case relayer passes `MsgChannelOpenInit` message with it set to `Some(_)`
        let raw_with_counterpary_chan_id_some = get_dummy_raw_msg_chan_open_init(None);
        let msg_with_counterpary_chan_id_some =
            MsgChannelOpenInit::try_from(raw_with_counterpary_chan_id_some).unwrap();
        let raw_with_counterpary_chan_id_some_back =
            RawMsgChannelOpenInit::from(msg_with_counterpary_chan_id_some.clone());
        let msg_with_counterpary_chan_id_some_back =
            MsgChannelOpenInit::try_from(raw_with_counterpary_chan_id_some_back.clone()).unwrap();
        assert_eq!(
            raw_with_counterpary_chan_id_some_back
                .channel
                .unwrap()
                .counterparty
                .unwrap()
                .channel_id,
            "".to_string()
        );
        assert_eq!(
            msg_with_counterpary_chan_id_some,
            msg_with_counterpary_chan_id_some_back
        );
    }
}
