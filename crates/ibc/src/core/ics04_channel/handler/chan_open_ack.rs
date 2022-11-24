//! Protocol logic specific to ICS4 messages of type `MsgChannelOpenAck`.
use crate::core::ics03_connection::connection::State as ConnectionState;
use crate::core::ics04_channel::channel::{ChannelEnd, Counterparty, State};
use crate::core::ics04_channel::context::ChannelReader;
use crate::core::ics04_channel::error::ChannelError;
use crate::core::ics04_channel::handler::{ChannelIdState, ChannelResult};
use crate::core::ics04_channel::msgs::chan_open_ack::MsgChannelOpenAck;
use crate::handler::{HandlerOutput, HandlerResult};
use crate::prelude::*;

/// Per our convention, this message is processed on chain A.
pub(crate) fn process<Ctx: ChannelReader>(
    ctx_a: &Ctx,
    msg: &MsgChannelOpenAck,
) -> HandlerResult<ChannelResult, ChannelError> {
    let mut output = HandlerOutput::builder();

    // Unwrap the old channel end and validate it against the message.
    let chan_end_on_a = ctx_a.channel_end(&msg.port_id_on_a, &msg.chan_id_on_a)?;

    // Validate that the channel end is in a state where it can be ack.
    if !chan_end_on_a.state_matches(&State::Init) {
        return Err(ChannelError::InvalidChannelState {
            channel_id: msg.chan_id_on_a.clone(),
            state: chan_end_on_a.state,
        });
    }

    // An OPEN IBC connection running on the local (host) chain should exist.

    if chan_end_on_a.connection_hops().len() != 1 {
        return Err(ChannelError::InvalidConnectionHopsLength {
            expected: 1,
            actual: chan_end_on_a.connection_hops().len(),
        });
    }

    let conn_end_on_a = ctx_a.connection_end(&chan_end_on_a.connection_hops()[0])?;

    if !conn_end_on_a.state_matches(&ConnectionState::Open) {
        return Err(ChannelError::ConnectionNotOpen {
            connection_id: chan_end_on_a.connection_hops()[0].clone(),
        });
    }

    // Verify proofs
    {
        let client_id_on_a = conn_end_on_a.client_id().clone();
        let client_state_of_b_on_a = ctx_a.client_state(&client_id_on_a)?;
        let consensus_state_of_b_on_a =
            ctx_a.client_consensus_state(&client_id_on_a, msg.proof_height_on_b)?;
        let prefix_on_b = conn_end_on_a.counterparty().prefix();
        let port_id_on_b = &chan_end_on_a.counterparty().port_id;
        let conn_id_on_b = conn_end_on_a.counterparty().connection_id().ok_or(
            ChannelError::UndefinedConnectionCounterparty {
                connection_id: chan_end_on_a.connection_hops()[0].clone(),
            },
        )?;

        // The client must not be frozen.
        if client_state_of_b_on_a.is_frozen() {
            return Err(ChannelError::FrozenClient {
                client_id: client_id_on_a,
            });
        }

        let expected_chan_end_on_b = ChannelEnd::new(
            State::TryOpen,
            // Note: Both ends of a channel must have the same ordering, so it's
            // fine to use A's ordering here
            *chan_end_on_a.ordering(),
            Counterparty::new(msg.port_id_on_a.clone(), Some(msg.chan_id_on_a.clone())),
            vec![conn_id_on_b.clone()],
            msg.version_on_b.clone(),
        );

        // Verify the proof for the channel state against the expected channel end.
        // A counterparty channel id of None in not possible, and is checked by validate_basic in msg.
        client_state_of_b_on_a
            .verify_channel_state(
                msg.proof_height_on_b,
                prefix_on_b,
                &msg.proof_chan_end_on_b,
                consensus_state_of_b_on_a.root(),
                port_id_on_b,
                &msg.chan_id_on_b,
                &expected_chan_end_on_b,
            )
            .map_err(ChannelError::VerifyChannelFailed)?;
    }

    output.log("success: channel open ack");

    // Transition the channel end to the new state & pick a version.
    let new_chan_end_on_a = {
        let mut chan_end_on_a = chan_end_on_a;

        chan_end_on_a.set_state(State::Open);
        chan_end_on_a.set_version(msg.version_on_b.clone());
        chan_end_on_a.set_counterparty_channel_id(msg.chan_id_on_b.clone());

        chan_end_on_a
    };

    let result = ChannelResult {
        port_id: msg.port_id_on_a.clone(),
        channel_id: msg.chan_id_on_a.clone(),
        channel_id_state: ChannelIdState::Reused,
        channel_end: new_chan_end_on_a,
    };

    Ok(output.with_result(result))
}

#[cfg(test)]
mod tests {
    use core::str::FromStr;

    use test_log::test;

    use crate::core::ics03_connection::connection::ConnectionEnd;
    use crate::core::ics03_connection::connection::Counterparty as ConnectionCounterparty;
    use crate::core::ics03_connection::connection::State as ConnectionState;
    use crate::core::ics03_connection::msgs::conn_open_init::test_util::get_dummy_raw_msg_conn_open_init;
    use crate::core::ics03_connection::msgs::conn_open_init::MsgConnectionOpenInit;
    use crate::core::ics03_connection::msgs::conn_open_try::test_util::get_dummy_raw_msg_conn_open_try;
    use crate::core::ics03_connection::msgs::conn_open_try::MsgConnectionOpenTry;
    use crate::core::ics03_connection::version::get_compatible_versions;
    use crate::core::ics04_channel::channel::{ChannelEnd, Counterparty, State};
    use crate::core::ics04_channel::handler::channel_dispatch;
    use crate::core::ics04_channel::msgs::chan_open_ack::test_util::get_dummy_raw_msg_chan_open_ack;
    use crate::core::ics04_channel::msgs::chan_open_ack::MsgChannelOpenAck;
    use crate::core::ics04_channel::msgs::chan_open_try::test_util::get_dummy_raw_msg_chan_open_try;
    use crate::core::ics04_channel::msgs::chan_open_try::MsgChannelOpenTry;
    use crate::core::ics04_channel::msgs::ChannelMsg;
    use crate::core::ics24_host::identifier::ConnectionId;
    use crate::mock::context::MockContext;
    use crate::prelude::*;
    use crate::Height;

    // TODO: The tests here are very fragile and complex.
    //  Should be adapted to use the same structure as `handler::chan_open_try::tests`.
    #[test]
    fn chan_open_ack_msg_processing() {
        struct Test {
            name: String,
            ctx: MockContext,
            msg: ChannelMsg,
            want_pass: bool,
        }
        let proof_height = 10;
        let client_consensus_state_height = 10;
        let host_chain_height = Height::new(0, 35).unwrap();

        let context = MockContext::default();

        let msg_conn_init =
            MsgConnectionOpenInit::try_from(get_dummy_raw_msg_conn_open_init()).unwrap();

        let conn_end = ConnectionEnd::new(
            ConnectionState::Open,
            msg_conn_init.client_id_on_a.clone(),
            ConnectionCounterparty::new(
                msg_conn_init.counterparty.client_id().clone(),
                Some(ConnectionId::from_str("defaultConnection-1").unwrap()),
                msg_conn_init.counterparty.prefix().clone(),
            ),
            get_compatible_versions(),
            msg_conn_init.delay_period,
        );

        let ccid = <ConnectionId as FromStr>::from_str("defaultConnection-0");
        let cid = match ccid {
            Ok(v) => v,
            Err(_e) => ConnectionId::default(),
        };

        let mut connection_vec0 = Vec::new();
        connection_vec0.insert(
            0,
            match <ConnectionId as FromStr>::from_str("defaultConnection-0") {
                Ok(a) => a,
                _ => unreachable!(),
            },
        );

        let msg_conn_try = MsgConnectionOpenTry::try_from(get_dummy_raw_msg_conn_open_try(
            client_consensus_state_height,
            host_chain_height.revision_height(),
        ))
        .unwrap();

        let msg_chan_ack =
            MsgChannelOpenAck::try_from(get_dummy_raw_msg_chan_open_ack(proof_height)).unwrap();

        let msg_chan_try =
            MsgChannelOpenTry::try_from(get_dummy_raw_msg_chan_open_try(proof_height)).unwrap();

        let chan_end = ChannelEnd::new(
            State::Init,
            *msg_chan_try.chan_end_on_b.ordering(),
            Counterparty::new(
                msg_chan_ack.port_id_on_a.clone(),
                Some(msg_chan_ack.chan_id_on_a.clone()),
            ),
            connection_vec0.clone(),
            msg_chan_try.chan_end_on_b.version().clone(),
        );

        let failed_chan_end = ChannelEnd::new(
            State::Open,
            *msg_chan_try.chan_end_on_b.ordering(),
            Counterparty::new(
                msg_chan_ack.port_id_on_a.clone(),
                Some(msg_chan_ack.chan_id_on_a.clone()),
            ),
            connection_vec0,
            msg_chan_try.chan_end_on_b.version().clone(),
        );

        let tests: Vec<Test> = vec![
            Test {
                name: "Processing fails because no channel exists in the context".to_string(),
                ctx: context.clone(),
                msg: ChannelMsg::ChannelOpenAck(msg_chan_ack.clone()),
                want_pass: false,
            },
            Test {
                name: "Processing fails because the channel is in the wrong state".to_string(),
                ctx: context
                    .clone()
                    .with_client(
                        &msg_conn_try.client_id_on_b,
                        Height::new(0, client_consensus_state_height).unwrap(),
                    )
                    .with_channel(
                        msg_chan_ack.port_id_on_a.clone(),
                        msg_chan_ack.chan_id_on_a.clone(),
                        failed_chan_end,
                    ),
                msg: ChannelMsg::ChannelOpenAck(msg_chan_ack.clone()),
                want_pass: false,
            },
            Test {
                name: "Processing fails because a connection does exist".to_string(),
                ctx: context
                    .clone()
                    .with_client(
                        &msg_conn_try.client_id_on_b,
                        Height::new(0, client_consensus_state_height).unwrap(),
                    )
                    .with_channel(
                        msg_chan_ack.port_id_on_a.clone(),
                        msg_chan_ack.chan_id_on_a.clone(),
                        chan_end.clone(),
                    ),
                msg: ChannelMsg::ChannelOpenAck(msg_chan_ack.clone()),
                want_pass: false,
            },
            Test {
                name: "Processing fails due to missing client state ".to_string(),
                ctx: context
                    .clone()
                    .with_connection(cid.clone(), conn_end.clone())
                    .with_channel(
                        msg_chan_ack.port_id_on_a.clone(),
                        msg_chan_ack.chan_id_on_a.clone(),
                        chan_end.clone(),
                    ),
                msg: ChannelMsg::ChannelOpenAck(msg_chan_ack.clone()),
                want_pass: false,
            },
            Test {
                name: "Good parameters".to_string(),
                ctx: context //  .clone()
                    .with_client(
                        &msg_conn_try.client_id_on_b,
                        Height::new(0, client_consensus_state_height).unwrap(),
                    )
                    .with_connection(cid, conn_end)
                    .with_channel(
                        msg_chan_ack.port_id_on_a.clone(),
                        msg_chan_ack.chan_id_on_a.clone(),
                        chan_end,
                    ),
                msg: ChannelMsg::ChannelOpenAck(msg_chan_ack),
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
                            "chan_open_ack: test passed but was supposed to fail for test: {}, \nparams {:?} {:?}",
                            test.name,
                            test.msg,
                            test.ctx.clone()
                        );

                    // The object in the output is a ConnectionEnd, should have init state.
                    //assert_eq!(res.channel_id, msg_chan_init.channel_id().clone());
                    assert_eq!(res.channel_end.state().clone(), State::Open);
                }
                Err(e) => {
                    assert!(
                        !test.want_pass,
                        "chan_open_ack: did not pass test: {}, \nparams {:?} {:?} error: {:?}",
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
