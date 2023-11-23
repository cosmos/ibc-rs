use ibc::core::channel::types::proto::v1::MsgChannelOpenInit as RawMsgChannelOpenInit;
use ibc::core::host::types::identifiers::PortId;
use ibc::core::primitives::prelude::*;

use super::dummy_raw_channel_end;
use crate::fixtures::core::signer::dummy_bech32_account;

/// Returns a dummy `RawMsgChannelOpenInit`, for testing purposes only!
pub fn dummy_raw_msg_chan_open_init(counterparty_channel_id: Option<u64>) -> RawMsgChannelOpenInit {
    RawMsgChannelOpenInit {
        port_id: PortId::transfer().to_string(),
        channel: Some(dummy_raw_channel_end(1, counterparty_channel_id)),
        signer: dummy_bech32_account(),
    }
}

#[cfg(test)]
mod tests {
    use ibc::core::channel::types::msgs::MsgChannelOpenInit;

    use super::*;

    #[test]
    fn channel_open_init_from_raw() {
        struct Test {
            name: String,
            raw: RawMsgChannelOpenInit,
            want_pass: bool,
        }

        let default_raw_init_msg = dummy_raw_msg_chan_open_init(None);

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
        let raw = dummy_raw_msg_chan_open_init(None);
        let msg = MsgChannelOpenInit::try_from(raw.clone()).unwrap();
        let raw_back = RawMsgChannelOpenInit::from(msg.clone());
        let msg_back = MsgChannelOpenInit::try_from(raw_back.clone()).unwrap();
        assert_eq!(raw, raw_back);
        assert_eq!(msg, msg_back);

        // Check if handler sets counterparty channel id to `None`
        // in case relayer passes `MsgChannelOpenInit` message with it set to `Some(_)`
        let raw_with_counterparty_chan_id_some = dummy_raw_msg_chan_open_init(None);
        let msg_with_counterparty_chan_id_some =
            MsgChannelOpenInit::try_from(raw_with_counterparty_chan_id_some).unwrap();
        let raw_with_counterparty_chan_id_some_back =
            RawMsgChannelOpenInit::from(msg_with_counterparty_chan_id_some.clone());
        let msg_with_counterparty_chan_id_some_back =
            MsgChannelOpenInit::try_from(raw_with_counterparty_chan_id_some_back.clone()).unwrap();
        assert_eq!(
            raw_with_counterparty_chan_id_some_back
                .channel
                .unwrap()
                .counterparty
                .unwrap()
                .channel_id,
            "".to_string()
        );
        assert_eq!(
            msg_with_counterparty_chan_id_some,
            msg_with_counterparty_chan_id_some_back
        );
    }
}
