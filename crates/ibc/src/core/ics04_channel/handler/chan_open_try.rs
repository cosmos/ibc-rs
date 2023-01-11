//! Protocol logic specific to ICS4 messages of type `MsgChannelOpenTry`.

use crate::core::ics03_connection::connection::State as ConnectionState;
use crate::core::ics04_channel::channel::{ChannelEnd, Counterparty, State};
use crate::core::ics04_channel::context::ChannelReader;
use crate::core::ics04_channel::error::ChannelError;
use crate::core::ics04_channel::handler::{ChannelIdState, ChannelResult};
use crate::core::ics04_channel::msgs::chan_open_try::MsgChannelOpenTry;
use crate::core::ics04_channel::Version;
use crate::core::ics24_host::identifier::ChannelId;
use crate::handler::{HandlerOutput, HandlerResult};
use crate::prelude::*;

#[cfg(feature = "val_exec_ctx")]
pub(crate) use val_exec_ctx::*;
#[cfg(feature = "val_exec_ctx")]
pub(crate) mod val_exec_ctx {
    use super::*;
    use crate::core::{ContextError, ExecutionContext, ValidationContext};

    pub fn validate<Ctx>(ctx_b: &Ctx, msg: &MsgChannelOpenTry) -> Result<ChannelId, ContextError>
    where
        Ctx: ValidationContext,
    {
        // An IBC connection running on the local (host) chain should exist.
        if msg.connection_hops_on_b.len() != 1 {
            return Err(ChannelError::InvalidConnectionHopsLength {
                expected: 1,
                actual: msg.connection_hops_on_b.len(),
            })
            .map_err(ContextError::ChannelError);
        }

        let conn_end_on_b = ctx_b.connection_end(&msg.connection_hops_on_b[0])?;
        if !conn_end_on_b.state_matches(&ConnectionState::Open) {
            return Err(ChannelError::ConnectionNotOpen {
                connection_id: msg.connection_hops_on_b[0].clone(),
            })
            .map_err(ContextError::ChannelError);
        }

        let conn_version = match conn_end_on_b.versions() {
            [version] => version,
            _ => {
                return Err(ChannelError::InvalidVersionLengthConnection)
                    .map_err(ContextError::ChannelError)
            }
        };

        let channel_feature = msg.ordering.to_string();
        if !conn_version.is_supported_feature(channel_feature) {
            return Err(ChannelError::ChannelFeatureNotSuportedByConnection)
                .map_err(ContextError::ChannelError);
        }

        // Verify proofs
        {
            let client_id_on_b = conn_end_on_b.client_id();
            let client_state_of_a_on_b = ctx_b.client_state(client_id_on_b)?;
            let consensus_state_of_a_on_b =
                ctx_b.consensus_state(client_id_on_b, &msg.proof_height_on_a)?;
            let prefix_on_a = conn_end_on_b.counterparty().prefix();
            let port_id_on_a = &&msg.port_id_on_a;
            let chan_id_on_a = msg.chan_id_on_a.clone();
            let conn_id_on_a = conn_end_on_b.counterparty().connection_id().ok_or(
                ChannelError::UndefinedConnectionCounterparty {
                    connection_id: msg.connection_hops_on_b[0].clone(),
                },
            )?;

            // The client must not be frozen.
            if client_state_of_a_on_b.is_frozen() {
                return Err(ChannelError::FrozenClient {
                    client_id: client_id_on_b.clone(),
                })
                .map_err(ContextError::ChannelError);
            }

            let expected_chan_end_on_a = ChannelEnd::new(
                State::Init,
                msg.ordering,
                Counterparty::new(msg.port_id_on_b.clone(), None),
                vec![conn_id_on_a.clone()],
                msg.version_supported_on_a.clone(),
            );

            // Verify the proof for the channel state against the expected channel end.
            // A counterparty channel id of None in not possible, and is checked by validate_basic in msg.
            client_state_of_a_on_b
                .verify_channel_state(
                    msg.proof_height_on_a,
                    prefix_on_a,
                    &msg.proof_chan_end_on_a,
                    consensus_state_of_a_on_b.root(),
                    port_id_on_a,
                    &chan_id_on_a,
                    &expected_chan_end_on_a,
                )
                .map_err(ChannelError::VerifyChannelFailed)?;
        }

        let chan_id_on_b = ChannelId::new(ctx_b.channel_counter()?);

        Ok(chan_id_on_b)
    }

    pub fn execute<Ctx>(ctx_b: &mut Ctx) -> Result<ChannelId, ContextError>
    where
        Ctx: ExecutionContext,
    {
        let chan_id_on_b = ChannelId::new(ctx_b.channel_counter()?);

        ctx_b.log_message(format!(
            "success: channel open try with channel identifier: {chan_id_on_b}"
        ));

        Ok(chan_id_on_b)
    }
}
/// Per our convention, this message is processed on chain B.
pub(crate) fn process<Ctx: ChannelReader>(
    ctx_b: &Ctx,
    msg: &MsgChannelOpenTry,
) -> HandlerResult<ChannelResult, ChannelError> {
    let mut output = HandlerOutput::builder();

    // An IBC connection running on the local (host) chain should exist.
    if msg.connection_hops_on_b.len() != 1 {
        return Err(ChannelError::InvalidConnectionHopsLength {
            expected: 1,
            actual: msg.connection_hops_on_b.len(),
        });
    }

    let conn_end_on_b = ctx_b.connection_end(&msg.connection_hops_on_b[0])?;
    if !conn_end_on_b.state_matches(&ConnectionState::Open) {
        return Err(ChannelError::ConnectionNotOpen {
            connection_id: msg.connection_hops_on_b[0].clone(),
        });
    }

    let conn_version = match conn_end_on_b.versions() {
        [version] => version,
        _ => return Err(ChannelError::InvalidVersionLengthConnection),
    };

    let channel_feature = msg.ordering.to_string();
    if !conn_version.is_supported_feature(channel_feature) {
        return Err(ChannelError::ChannelFeatureNotSuportedByConnection);
    }

    // Verify proofs
    {
        let client_id_on_b = conn_end_on_b.client_id();
        let client_state_of_a_on_b = ctx_b.client_state(client_id_on_b)?;
        let consensus_state_of_a_on_b =
            ctx_b.client_consensus_state(client_id_on_b, &msg.proof_height_on_a)?;
        let prefix_on_a = conn_end_on_b.counterparty().prefix();
        let port_id_on_a = &&msg.port_id_on_a;
        let chan_id_on_a = msg.chan_id_on_a.clone();
        let conn_id_on_a = conn_end_on_b.counterparty().connection_id().ok_or(
            ChannelError::UndefinedConnectionCounterparty {
                connection_id: msg.connection_hops_on_b[0].clone(),
            },
        )?;

        // The client must not be frozen.
        if client_state_of_a_on_b.is_frozen() {
            return Err(ChannelError::FrozenClient {
                client_id: client_id_on_b.clone(),
            });
        }

        let expected_chan_end_on_a = ChannelEnd::new(
            State::Init,
            msg.ordering,
            Counterparty::new(msg.port_id_on_b.clone(), None),
            vec![conn_id_on_a.clone()],
            msg.version_supported_on_a.clone(),
        );

        // Verify the proof for the channel state against the expected channel end.
        // A counterparty channel id of None in not possible, and is checked by validate_basic in msg.
        client_state_of_a_on_b
            .verify_channel_state(
                msg.proof_height_on_a,
                prefix_on_a,
                &msg.proof_chan_end_on_a,
                consensus_state_of_a_on_b.root(),
                port_id_on_a,
                &chan_id_on_a,
                &expected_chan_end_on_a,
            )
            .map_err(ChannelError::VerifyChannelFailed)?;
    }

    let chan_end_on_b = ChannelEnd::new(
        State::TryOpen,
        msg.ordering,
        Counterparty::new(msg.port_id_on_a.clone(), Some(msg.chan_id_on_a.clone())),
        msg.connection_hops_on_b.clone(),
        // Note: This will be rewritten by the module callback
        Version::empty(),
    );

    let chan_id_on_b = ChannelId::new(ctx_b.channel_counter()?);

    output.log(format!(
        "success: channel open try with channel identifier: {chan_id_on_b}"
    ));

    let result = ChannelResult {
        port_id: msg.port_id_on_b.clone(),
        channel_id: chan_id_on_b,
        channel_end: chan_end_on_b,
        channel_id_state: ChannelIdState::Generated,
    };

    Ok(output.with_result(result))
}

#[cfg(test)]
mod tests {
    use crate::core::ics04_channel::handler::chan_open_try;
    use crate::downcast;
    use crate::prelude::*;

    use test_log::test;

    use crate::core::ics02_client::error as ics02_error;
    use crate::core::ics03_connection::connection::ConnectionEnd;
    use crate::core::ics03_connection::connection::Counterparty as ConnectionCounterparty;
    use crate::core::ics03_connection::connection::State as ConnectionState;
    use crate::core::ics03_connection::error as ics03_error;
    use crate::core::ics03_connection::msgs::test_util::get_dummy_raw_counterparty;
    use crate::core::ics03_connection::version::get_compatible_versions;
    use crate::core::ics04_channel::channel::{ChannelEnd, Counterparty, State};
    use crate::core::ics04_channel::error;
    use crate::core::ics04_channel::msgs::chan_open_try::test_util::get_dummy_raw_msg_chan_open_try;
    use crate::core::ics04_channel::msgs::chan_open_try::MsgChannelOpenTry;
    use crate::core::ics04_channel::msgs::ChannelMsg;
    use crate::core::ics04_channel::Version;
    use crate::core::ics24_host::identifier::{ChannelId, ClientId, ConnectionId};
    use crate::mock::client_state::client_type as mock_client_type;
    use crate::mock::context::MockContext;
    use crate::timestamp::ZERO_DURATION;
    use crate::Height;

    #[test]
    fn chan_open_try_msg_processing() {
        struct Test {
            name: String,
            ctx: MockContext,
            msg: ChannelMsg,
            want_pass: bool,
            match_error: Box<dyn FnOnce(error::ChannelError)>,
        }

        // Some general-purpose variable to parametrize the messages and the context.
        let proof_height = 10;
        let conn_id = ConnectionId::new(2);
        let client_id = ClientId::new(mock_client_type(), 45).unwrap();

        // The context. We'll reuse this same one across all tests.
        let context = MockContext::default();

        // This is the connection underlying the channel we're trying to open.
        let conn_end = ConnectionEnd::new(
            ConnectionState::Open,
            client_id.clone(),
            ConnectionCounterparty::try_from(get_dummy_raw_counterparty()).unwrap(),
            get_compatible_versions(),
            ZERO_DURATION,
        );

        // We're going to test message processing against this message.
        let mut msg =
            MsgChannelOpenTry::try_from(get_dummy_raw_msg_chan_open_try(proof_height)).unwrap();

        let chan_id = ChannelId::new(24);
        let hops = vec![conn_id.clone()];
        msg.connection_hops_on_b = hops;

        // A preloaded channel end that resides in the context. This is constructed so as to be
        // consistent with the incoming ChanOpenTry message `msg`.
        let correct_chan_end = ChannelEnd::new(
            State::Init,
            msg.ordering,
            Counterparty::new(msg.port_id_on_a.clone(), Some(msg.chan_id_on_a.clone())),
            msg.connection_hops_on_b.clone(),
            Version::empty(),
        );

        let tests: Vec<Test> = vec![
            Test {
                name: "Processing fails because no connection exists in the context".to_string(),
                ctx: context.clone(),
                msg: ChannelMsg::OpenTry(msg.clone()),
                want_pass: false,
                match_error: {
                    let connection_id = msg.connection_hops_on_b[0].clone();
                    Box::new(move |e| match e {
                        error::ChannelError::Connection(e) => {
                            assert_eq!(
                                e.to_string(),
                                ics03_error::ConnectionError::ConnectionNotFound { connection_id }
                                    .to_string()
                            );
                        }
                        _ => {
                            panic!("Expected MissingConnection, instead got {}", e)
                        }
                    })
                },
            },
            Test {
                name: "Processing fails b/c the context has no client state".to_string(),
                ctx: context
                    .clone()
                    .with_connection(conn_id.clone(), conn_end.clone())
                    .with_channel(
                        msg.port_id_on_b.clone(),
                        chan_id.clone(),
                        correct_chan_end.clone(),
                    ),
                msg: ChannelMsg::OpenTry(msg.clone()),
                want_pass: false,
                match_error: Box::new(|e| match e {
                    error::ChannelError::Connection(e) => {
                        assert_eq!(
                            e.to_string(),
                            ics03_error::ConnectionError::Client(
                                ics02_error::ClientError::ClientNotFound {
                                    client_id: ClientId::new(mock_client_type(), 45).unwrap()
                                }
                            )
                            .to_string()
                        );
                    }
                    _ => {
                        panic!("Expected MissingClientState, instead got {}", e)
                    }
                }),
            },
            Test {
                name: "Processing is successful".to_string(),
                ctx: context
                    .clone()
                    .with_client(&client_id, Height::new(0, proof_height).unwrap())
                    .with_connection(conn_id.clone(), conn_end.clone())
                    .with_channel(msg.port_id_on_b.clone(), chan_id, correct_chan_end),
                msg: ChannelMsg::OpenTry(msg.clone()),
                want_pass: true,
                match_error: Box::new(|_| {}),
            },
            Test {
                name: "Processing is successful against an empty context (no preexisting channel)"
                    .to_string(),
                ctx: context
                    .with_client(&client_id, Height::new(0, proof_height).unwrap())
                    .with_connection(conn_id, conn_end),
                msg: ChannelMsg::OpenTry(msg),
                want_pass: true,
                match_error: Box::new(|_| {}),
            },
        ]
        .into_iter()
        .collect();

        for test in tests {
            let test_msg = downcast!(test.msg => ChannelMsg::OpenTry).unwrap();
            let res = chan_open_try::process(&test.ctx, &test_msg);
            // Additionally check the events and the output objects in the result.
            match res {
                Ok(proto_output) => {
                    assert!(
                        test.want_pass,
                        "chan_open_ack: test passed but was supposed to fail for test: {}, \nparams {:?} {:?}",
                        test.name,
                        test_msg,
                        test.ctx.clone()
                    );

                    // The object in the output is a channel end, should have TryOpen state.
                    assert_eq!(
                        proto_output.result.channel_end.state().clone(),
                        State::TryOpen
                    );
                }
                Err(e) => {
                    assert!(
                        !test.want_pass,
                        "chan_open_try: did not pass test: {}, \nparams:\n\tmsg={:?}\n\tcontext={:?}\nerror: {:?}",
                        test.name,
                        test_msg,
                        test.ctx.clone(),
                        e,
                    );

                    (test.match_error)(e);
                }
            }
        }
    }

    /// Addresses [issue 219](https://github.com/cosmos/ibc-rs/issues/219)
    #[test]
    fn chan_open_try_invalid_counterparty_channel_id() {
        let proof_height = 10;
        let conn_id = ConnectionId::new(2);
        let client_id = ClientId::new(mock_client_type(), 45).unwrap();

        // This is the connection underlying the channel we're trying to open.
        let conn_end = ConnectionEnd::new(
            ConnectionState::Open,
            client_id.clone(),
            ConnectionCounterparty::try_from(get_dummy_raw_counterparty()).unwrap(),
            get_compatible_versions(),
            ZERO_DURATION,
        );

        // We're going to test message processing against this message.
        // Note: we make the counterparty's channel_id `None`.
        let mut msg =
            MsgChannelOpenTry::try_from(get_dummy_raw_msg_chan_open_try(proof_height)).unwrap();

        let chan_id = ChannelId::new(24);
        let hops = vec![conn_id.clone()];
        msg.connection_hops_on_b = hops;

        let chan_end = ChannelEnd::new(
            State::Init,
            msg.ordering,
            Counterparty::new(msg.port_id_on_a.clone(), None),
            msg.connection_hops_on_b.clone(),
            Version::empty(),
        );
        let context = MockContext::default()
            .with_client(&client_id, Height::new(0, proof_height).unwrap())
            .with_connection(conn_id, conn_end)
            .with_channel(msg.port_id_on_b.clone(), chan_id, chan_end);

        // Makes sure we don't crash
        let _ = chan_open_try::process(&context, &msg);
    }
}
