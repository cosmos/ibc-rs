//! Protocol logic specific to ICS4 messages of type `MsgChannelOpenConfirm`.
use crate::core::ics03_connection::connection::State as ConnectionState;
use crate::core::ics04_channel::channel::{ChannelEnd, Counterparty, State};
use crate::core::ics04_channel::context::ChannelReader;
use crate::core::ics04_channel::error::ChannelError;
use crate::core::ics04_channel::handler::{ChannelIdState, ChannelResult};
use crate::core::ics04_channel::msgs::chan_open_confirm::MsgChannelOpenConfirm;
use crate::handler::{HandlerOutput, HandlerResult};
use crate::prelude::*;

#[cfg(feature = "val_exec_ctx")]
pub(crate) use val_exec_ctx::*;
#[cfg(feature = "val_exec_ctx")]
pub(crate) mod val_exec_ctx {
    use super::*;
    use crate::core::{ContextError, ValidationContext};

    pub fn validate<Ctx>(ctx_b: &Ctx, msg: &MsgChannelOpenConfirm) -> Result<(), ContextError>
    where
        Ctx: ValidationContext,
    {
        // Unwrap the old channel end and validate it against the message.
        let chan_end_on_b =
            ctx_b.channel_end(&(msg.port_id_on_b.clone(), msg.chan_id_on_b.clone()))?;

        // Validate that the channel end is in a state where it can be confirmed.
        if !chan_end_on_b.state_matches(&State::TryOpen) {
            return Err(ChannelError::InvalidChannelState {
                channel_id: msg.chan_id_on_b.clone(),
                state: chan_end_on_b.state,
            }
            .into());
        }

        // An OPEN IBC connection running on the local (host) chain should exist.
        if chan_end_on_b.connection_hops().len() != 1 {
            return Err(ChannelError::InvalidConnectionHopsLength {
                expected: 1,
                actual: chan_end_on_b.connection_hops().len(),
            }
            .into());
        }

        let conn_end_on_b = ctx_b.connection_end(&chan_end_on_b.connection_hops()[0])?;

        if !conn_end_on_b.state_matches(&ConnectionState::Open) {
            return Err(ChannelError::ConnectionNotOpen {
                connection_id: chan_end_on_b.connection_hops()[0].clone(),
            }
            .into());
        }

        // Verify proofs
        {
            let client_id_on_b = conn_end_on_b.client_id();
            let client_state_of_a_on_b = ctx_b.client_state(client_id_on_b)?;
            let consensus_state_of_a_on_b =
                ctx_b.consensus_state(client_id_on_b, &msg.proof_height_on_a)?;
            let prefix_on_a = conn_end_on_b.counterparty().prefix();
            let port_id_on_a = &chan_end_on_b.counterparty().port_id;
            let chan_id_on_a = chan_end_on_b
                .counterparty()
                .channel_id()
                .ok_or(ChannelError::InvalidCounterpartyChannelId)?;
            let conn_id_on_a = conn_end_on_b.counterparty().connection_id().ok_or(
                ChannelError::UndefinedConnectionCounterparty {
                    connection_id: chan_end_on_b.connection_hops()[0].clone(),
                },
            )?;

            // The client must not be frozen.
            if client_state_of_a_on_b.is_frozen() {
                return Err(ChannelError::FrozenClient {
                    client_id: client_id_on_b.clone(),
                }
                .into());
            }

            let expected_chan_end_on_a = ChannelEnd::new(
                State::Open,
                *chan_end_on_b.ordering(),
                Counterparty::new(msg.port_id_on_b.clone(), Some(msg.chan_id_on_b.clone())),
                vec![conn_id_on_a.clone()],
                chan_end_on_b.version.clone(),
            );

            // Verify the proof for the channel state against the expected channel end.
            // A counterparty channel id of None in not possible, and is checked in msg.
            client_state_of_a_on_b
                .verify_channel_state(
                    msg.proof_height_on_a,
                    prefix_on_a,
                    &msg.proof_chan_end_on_a,
                    consensus_state_of_a_on_b.root(),
                    port_id_on_a,
                    chan_id_on_a,
                    &expected_chan_end_on_a,
                )
                .map_err(ChannelError::VerifyChannelFailed)?;
        }

        Ok(())
    }
}

/// Per our convention, this message is processed on chain B.
pub(crate) fn process<Ctx: ChannelReader>(
    ctx_b: &Ctx,
    msg: &MsgChannelOpenConfirm,
) -> HandlerResult<ChannelResult, ChannelError> {
    let mut output = HandlerOutput::builder();

    // Unwrap the old channel end and validate it against the message.
    let mut chan_end_on_b = ctx_b.channel_end(&msg.port_id_on_b, &msg.chan_id_on_b)?;

    // Validate that the channel end is in a state where it can be confirmed.
    if !chan_end_on_b.state_matches(&State::TryOpen) {
        return Err(ChannelError::InvalidChannelState {
            channel_id: msg.chan_id_on_b.clone(),
            state: chan_end_on_b.state,
        });
    }

    // An OPEN IBC connection running on the local (host) chain should exist.
    if chan_end_on_b.connection_hops().len() != 1 {
        return Err(ChannelError::InvalidConnectionHopsLength {
            expected: 1,
            actual: chan_end_on_b.connection_hops().len(),
        });
    }

    let conn_end_on_b = ctx_b.connection_end(&chan_end_on_b.connection_hops()[0])?;

    if !conn_end_on_b.state_matches(&ConnectionState::Open) {
        return Err(ChannelError::ConnectionNotOpen {
            connection_id: chan_end_on_b.connection_hops()[0].clone(),
        });
    }

    // Verify proofs
    {
        let client_id_on_b = conn_end_on_b.client_id();
        let client_state_of_a_on_b = ctx_b.client_state(client_id_on_b)?;
        let consensus_state_of_a_on_b =
            ctx_b.client_consensus_state(client_id_on_b, &msg.proof_height_on_a)?;
        let prefix_on_a = conn_end_on_b.counterparty().prefix();
        let port_id_on_a = &chan_end_on_b.counterparty().port_id;
        let chan_id_on_a = chan_end_on_b
            .counterparty()
            .channel_id()
            .ok_or(ChannelError::InvalidCounterpartyChannelId)?;
        let conn_id_on_a = conn_end_on_b.counterparty().connection_id().ok_or(
            ChannelError::UndefinedConnectionCounterparty {
                connection_id: chan_end_on_b.connection_hops()[0].clone(),
            },
        )?;

        // The client must not be frozen.
        if client_state_of_a_on_b.is_frozen() {
            return Err(ChannelError::FrozenClient {
                client_id: client_id_on_b.clone(),
            });
        }

        let expected_chan_end_on_a = ChannelEnd::new(
            State::Open,
            *chan_end_on_b.ordering(),
            Counterparty::new(msg.port_id_on_b.clone(), Some(msg.chan_id_on_b.clone())),
            vec![conn_id_on_a.clone()],
            chan_end_on_b.version.clone(),
        );

        // Verify the proof for the channel state against the expected channel end.
        // A counterparty channel id of None in not possible, and is checked in msg.
        client_state_of_a_on_b
            .verify_channel_state(
                msg.proof_height_on_a,
                prefix_on_a,
                &msg.proof_chan_end_on_a,
                consensus_state_of_a_on_b.root(),
                port_id_on_a,
                chan_id_on_a,
                &expected_chan_end_on_a,
            )
            .map_err(ChannelError::VerifyChannelFailed)?;
    }

    output.log("success: channel open confirm ");

    // Transition the channel end to the new state.
    chan_end_on_b.set_state(State::Open);

    let result = ChannelResult {
        port_id: msg.port_id_on_b.clone(),
        channel_id: msg.chan_id_on_b.clone(),
        channel_id_state: ChannelIdState::Reused,
        channel_end: chan_end_on_b,
    };

    Ok(output.with_result(result))
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    use test_log::test;

    use crate::core::ics03_connection::connection::ConnectionEnd;
    use crate::core::ics03_connection::connection::Counterparty as ConnectionCounterparty;
    use crate::core::ics03_connection::connection::State as ConnectionState;
    use crate::core::ics03_connection::context::ConnectionReader;
    use crate::core::ics03_connection::msgs::test_util::get_dummy_raw_counterparty;
    use crate::core::ics03_connection::version::get_compatible_versions;
    use crate::core::ics04_channel::channel::{ChannelEnd, Counterparty, Order, State};
    use crate::core::ics04_channel::handler::channel_dispatch;
    use crate::core::ics04_channel::msgs::chan_open_confirm::test_util::get_dummy_raw_msg_chan_open_confirm;
    use crate::core::ics04_channel::msgs::chan_open_confirm::MsgChannelOpenConfirm;
    use crate::core::ics04_channel::msgs::ChannelMsg;
    use crate::core::ics04_channel::Version;
    use crate::core::ics24_host::identifier::{ClientId, ConnectionId};
    use crate::mock::client_state::client_type as mock_client_type;
    use crate::mock::context::MockContext;
    use crate::timestamp::ZERO_DURATION;
    use crate::Height;

    // TODO: The tests here should use the same structure as `handler::chan_open_try::tests`.
    #[test]
    fn chan_open_confirm_msg_processing() {
        struct Test {
            name: String,
            ctx: MockContext,
            msg: ChannelMsg,
            want_pass: bool,
        }
        let client_id = ClientId::new(mock_client_type(), 24).unwrap();
        let conn_id = ConnectionId::new(2);
        let context = MockContext::default();
        let client_consensus_state_height =
            context.host_current_height().unwrap().revision_height();

        // The connection underlying the channel we're trying to open.
        let conn_end = ConnectionEnd::new(
            ConnectionState::Open,
            client_id.clone(),
            ConnectionCounterparty::try_from(get_dummy_raw_counterparty(Some(0))).unwrap(),
            get_compatible_versions(),
            ZERO_DURATION,
        );

        let msg_chan_confirm = MsgChannelOpenConfirm::try_from(
            get_dummy_raw_msg_chan_open_confirm(client_consensus_state_height),
        )
        .unwrap();

        let chan_end = ChannelEnd::new(
            State::TryOpen,
            Order::default(),
            Counterparty::new(
                msg_chan_confirm.port_id_on_b.clone(),
                Some(msg_chan_confirm.chan_id_on_b.clone()),
            ),
            vec![conn_id.clone()],
            Version::default(),
        );

        let tests: Vec<Test> = vec![Test {
            name: "Good parameters".to_string(),
            ctx: context
                .with_client(
                    &client_id,
                    Height::new(0, client_consensus_state_height).unwrap(),
                )
                .with_connection(conn_id, conn_end)
                .with_channel(
                    msg_chan_confirm.port_id_on_b.clone(),
                    msg_chan_confirm.chan_id_on_b.clone(),
                    chan_end,
                ),
            msg: ChannelMsg::OpenConfirm(msg_chan_confirm),
            want_pass: true,
        }]
        .into_iter()
        .collect();

        for test in tests {
            let res = channel_dispatch(&test.ctx, &test.msg);
            // Additionally check the events and the output objects in the result.
            match res {
                Ok((_, res)) => {
                    assert!(
                            test.want_pass,
                            "chan_open_confirm: test passed but was supposed to fail for test: {}, \nparams {:?} {:?}",
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
                        "chan_open_ack: did not pass test: {}, \nparams {:?} {:?}\nerror: {:?}",
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
