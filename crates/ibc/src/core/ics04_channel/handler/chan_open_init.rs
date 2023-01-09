//! Protocol logic specific to ICS4 messages of type `MsgChannelOpenInit`.

use crate::core::ics04_channel::channel::{ChannelEnd, State};
use crate::core::ics04_channel::context::ChannelReader;
use crate::core::ics04_channel::error::ChannelError;
use crate::core::ics04_channel::handler::{ChannelIdState, ChannelResult};
use crate::core::ics04_channel::msgs::chan_open_init::MsgChannelOpenInit;
use crate::core::ics24_host::identifier::ChannelId;
use crate::handler::{HandlerOutput, HandlerResult};
use crate::prelude::*;

/// Per our convention, this message is processed on chain A.
pub(crate) fn process<Ctx: ChannelReader>(
    ctx_a: &Ctx,
    msg: &MsgChannelOpenInit,
) -> HandlerResult<ChannelResult, ChannelError> {
    let mut output = HandlerOutput::builder();

    if msg.chan_end_on_a.connection_hops().len() != 1 {
        return Err(ChannelError::InvalidConnectionHopsLength {
            expected: 1,
            actual: msg.chan_end_on_a.connection_hops().len(),
        });
    }

    // An IBC connection running on the local (host) chain should exist.
    let conn_end_on_a = ctx_a.connection_end(&msg.chan_end_on_a.connection_hops()[0])?;

    let conn_version = match conn_end_on_a.versions() {
        [version] => version,
        _ => return Err(ChannelError::InvalidVersionLengthConnection),
    };

    let channel_feature = msg.chan_end_on_a.ordering().to_string();
    if !conn_version.is_supported_feature(channel_feature) {
        return Err(ChannelError::ChannelFeatureNotSuportedByConnection);
    }

    let chan_end_on_a = ChannelEnd::new(
        State::Init,
        *msg.chan_end_on_a.ordering(),
        msg.chan_end_on_a.counterparty().clone(),
        msg.chan_end_on_a.connection_hops().clone(),
        msg.chan_end_on_a.version().clone(),
    );

    let chan_id_on_a = ChannelId::new(ctx_a.channel_counter()?);

    output.log(format!(
        "success: channel open init with channel identifier: {chan_id_on_a}"
    ));

    let result = ChannelResult {
        port_id: msg.port_id_on_a.clone(),
        channel_id: chan_id_on_a,
        channel_end: chan_end_on_a,
        channel_id_state: ChannelIdState::Generated,
    };

    Ok(output.with_result(result))
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    use test_log::test;

    use crate::core::ics03_connection::connection::ConnectionEnd;
    use crate::core::ics03_connection::connection::State as ConnectionState;
    use crate::core::ics03_connection::msgs::conn_open_init::test_util::get_dummy_raw_msg_conn_open_init;
    use crate::core::ics03_connection::msgs::conn_open_init::MsgConnectionOpenInit;
    use crate::core::ics03_connection::version::get_compatible_versions;
    use crate::core::ics04_channel::channel::State;
    use crate::core::ics04_channel::handler::channel_dispatch;
    use crate::core::ics04_channel::msgs::chan_open_init::test_util::get_dummy_raw_msg_chan_open_init;
    use crate::core::ics04_channel::msgs::chan_open_init::MsgChannelOpenInit;
    use crate::core::ics04_channel::msgs::ChannelMsg;
    use crate::core::ics24_host::identifier::ConnectionId;
    use crate::mock::context::MockContext;

    #[test]
    fn chan_open_init_msg_processing() {
        struct Test {
            name: String,
            ctx: MockContext,
            msg: ChannelMsg,
            want_pass: bool,
        }

        let msg_chan_init =
            MsgChannelOpenInit::try_from(get_dummy_raw_msg_chan_open_init()).unwrap();

        let context = MockContext::default();

        let msg_conn_init =
            MsgConnectionOpenInit::try_from(get_dummy_raw_msg_conn_open_init()).unwrap();

        let init_conn_end = ConnectionEnd::new(
            ConnectionState::Init,
            msg_conn_init.client_id_on_a.clone(),
            msg_conn_init.counterparty.clone(),
            get_compatible_versions(),
            msg_conn_init.delay_period,
        );

        let cid = ConnectionId::default();

        let tests: Vec<Test> = vec![
            Test {
                name: "Processing fails because no connection exists in the context".to_string(),
                ctx: context.clone(),
                msg: ChannelMsg::OpenInit(msg_chan_init.clone()),
                want_pass: false,
            },
            Test {
                name: "Good parameters".to_string(),
                ctx: context.with_connection(cid, init_conn_end),
                msg: ChannelMsg::OpenInit(msg_chan_init),
                want_pass: true,
            },
        ]
        .into_iter()
        .collect();

        for test in tests {
            let res = channel_dispatch(&test.ctx, &test.msg);
            // Additionally check the events and the output objects in the result.
            match res {
                Ok((_, res)) => {
                    assert!(
                        test.want_pass,
                        "chan_open_init: test passed but was supposed to fail for test: {}, \nparams {:?} {:?}",
                        test.name,
                        test.msg,
                        test.ctx.clone()
                    );

                    // The object in the output is a ChannelEnd, should have init state.
                    assert_eq!(res.channel_end.state().clone(), State::Init);
                    let msg_init = test.msg;

                    if let ChannelMsg::OpenInit(msg_init) = msg_init {
                        assert_eq!(res.port_id.clone(), msg_init.port_id_on_a.clone());
                    }
                }
                Err(e) => {
                    assert!(
                        !test.want_pass,
                        "chan_open_init: did not pass test: {}, \nparams {:?} {:?} error: {:?}",
                        test.name,
                        test.msg,
                        test.ctx.clone(),
                        e,
                    );
                }
            }
        }
    }
}
