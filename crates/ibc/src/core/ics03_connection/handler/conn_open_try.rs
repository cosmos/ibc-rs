//! Protocol logic specific to processing ICS3 messages of type `MsgConnectionOpenTry`.
use crate::prelude::*;

use crate::core::ics03_connection::connection::{ConnectionEnd, Counterparty, State};
use crate::core::ics03_connection::context::ConnectionReader;
use crate::core::ics03_connection::error::ConnectionError;
use crate::core::ics03_connection::events::OpenTry;
use crate::core::ics03_connection::handler::ConnectionResult;
use crate::core::ics03_connection::msgs::conn_open_try::MsgConnectionOpenTry;
use crate::core::ics24_host::identifier::ConnectionId;
use crate::events::IbcEvent;
use crate::handler::{HandlerOutput, HandlerResult};

use super::ConnectionIdState;

#[cfg(feature = "val_exec_ctx")]
use crate::core::context::ContextError;
#[cfg(feature = "val_exec_ctx")]
use crate::core::ics24_host::identifier::ClientId;
#[cfg(feature = "val_exec_ctx")]
use crate::core::ics24_host::path::{ClientConnectionsPath, ConnectionsPath};
#[cfg(feature = "val_exec_ctx")]
use crate::core::{ExecutionContext, ValidationContext};

#[cfg(feature = "val_exec_ctx")]
pub(crate) fn validate<Ctx>(ctx_b: &Ctx, msg: MsgConnectionOpenTry) -> Result<(), ContextError>
where
    Ctx: ValidationContext,
{
    let vars = LocalVars::new(ctx_b, &msg)?;
    validate_impl(ctx_b, &msg, &vars)
}

#[cfg(feature = "val_exec_ctx")]
fn validate_impl<Ctx>(
    ctx_b: &Ctx,
    msg: &MsgConnectionOpenTry,
    vars: &LocalVars,
) -> Result<(), ContextError>
where
    Ctx: ValidationContext,
{
    ctx_b.validate_self_client(msg.client_state_of_b_on_a.clone())?;

    let host_height = ctx_b.host_height().map_err(|_| ConnectionError::Other {
        description: "failed to get host height".to_string(),
    })?;
    if msg.consensus_height_of_b_on_a > host_height {
        // Fail if the consensus height is too advanced.
        return Err(ConnectionError::InvalidConsensusHeight {
            target_height: msg.consensus_height_of_b_on_a,
            current_height: host_height,
        }
        .into());
    }

    let client_id_on_a = msg.counterparty.client_id();

    // Verify proofs
    {
        let client_state_of_a_on_b =
            ctx_b
                .client_state(vars.conn_end_on_b.client_id())
                .map_err(|_| ConnectionError::Other {
                    description: "failed to fetch client state".to_string(),
                })?;
        let consensus_state_of_a_on_b = ctx_b
            .consensus_state(&msg.client_id_on_b, &msg.proofs_height_on_a)
            .map_err(|_| ConnectionError::Other {
                description: "failed to fetch client consensus state".to_string(),
            })?;

        let prefix_on_a = vars.conn_end_on_b.counterparty().prefix();
        let prefix_on_b = ctx_b.commitment_prefix();

        {
            let expected_conn_end_on_a = ConnectionEnd::new(
                State::Init,
                client_id_on_a.clone(),
                Counterparty::new(msg.client_id_on_b.clone(), None, prefix_on_b),
                msg.versions_on_a.clone(),
                msg.delay_period,
            );

            client_state_of_a_on_b
                .verify_connection_state(
                    msg.proofs_height_on_a,
                    prefix_on_a,
                    &msg.proof_conn_end_on_a,
                    consensus_state_of_a_on_b.root(),
                    &vars.conn_id_on_a,
                    &expected_conn_end_on_a,
                )
                .map_err(ConnectionError::VerifyConnectionState)?;
        }

        client_state_of_a_on_b
            .verify_client_full_state(
                msg.proofs_height_on_a,
                prefix_on_a,
                &msg.proof_client_state_of_b_on_a,
                consensus_state_of_a_on_b.root(),
                client_id_on_a,
                msg.client_state_of_b_on_a.clone(),
            )
            .map_err(|e| ConnectionError::ClientStateVerificationFailure {
                client_id: msg.client_id_on_b.clone(),
                client_error: e,
            })?;

        let expected_consensus_state_of_b_on_a = ctx_b
            .host_consensus_state(&msg.consensus_height_of_b_on_a)
            .map_err(|_| ConnectionError::Other {
                description: "failed to fetch host consensus state".to_string(),
            })?;
        client_state_of_a_on_b
            .verify_client_consensus_state(
                msg.proofs_height_on_a,
                prefix_on_a,
                &msg.proof_consensus_state_of_b_on_a,
                consensus_state_of_a_on_b.root(),
                client_id_on_a,
                msg.consensus_height_of_b_on_a,
                expected_consensus_state_of_b_on_a.as_ref(),
            )
            .map_err(|e| ConnectionError::ClientStateVerificationFailure {
                client_id: client_id_on_a.clone(),
                client_error: e,
            })?;
    }

    Ok(())
}

#[cfg(feature = "val_exec_ctx")]
pub(crate) fn execute<Ctx>(ctx_b: &mut Ctx, msg: MsgConnectionOpenTry) -> Result<(), ContextError>
where
    Ctx: ExecutionContext,
{
    let vars = LocalVars::new(ctx_b, &msg)?;
    execute_impl(ctx_b, msg, vars)
}

#[cfg(feature = "val_exec_ctx")]
fn execute_impl<Ctx>(
    ctx_b: &mut Ctx,
    msg: MsgConnectionOpenTry,
    vars: LocalVars,
) -> Result<(), ContextError>
where
    Ctx: ExecutionContext,
{
    let conn_id_on_a = vars
        .conn_end_on_b
        .counterparty()
        .connection_id()
        .ok_or(ConnectionError::InvalidCounterparty)?;
    ctx_b.emit_ibc_event(IbcEvent::OpenTryConnection(OpenTry::new(
        vars.conn_id_on_b.clone(),
        msg.client_id_on_b.clone(),
        conn_id_on_a.clone(),
        vars.client_id_on_a.clone(),
    )));
    ctx_b.log_message("success: conn_open_try verification passed".to_string());

    ctx_b.increase_connection_counter();
    ctx_b.store_connection_to_client(
        ClientConnectionsPath(msg.client_id_on_b),
        vars.conn_id_on_b.clone(),
    )?;
    ctx_b.store_connection(ConnectionsPath(vars.conn_id_on_b), vars.conn_end_on_b)?;

    Ok(())
}

#[cfg(feature = "val_exec_ctx")]
struct LocalVars {
    conn_id_on_b: ConnectionId,
    conn_end_on_b: ConnectionEnd,
    client_id_on_a: ClientId,
    conn_id_on_a: ConnectionId,
}

#[cfg(feature = "val_exec_ctx")]
impl LocalVars {
    fn new<Ctx>(ctx_b: &Ctx, msg: &MsgConnectionOpenTry) -> Result<Self, ContextError>
    where
        Ctx: ValidationContext,
    {
        let version_on_b =
            ctx_b.pick_version(&ctx_b.get_compatible_versions(), &msg.versions_on_a)?;

        Ok(Self {
            conn_id_on_b: ConnectionId::new(ctx_b.connection_counter()?),
            conn_end_on_b: ConnectionEnd::new(
                State::TryOpen,
                msg.client_id_on_b.clone(),
                msg.counterparty.clone(),
                vec![version_on_b],
                msg.delay_period,
            ),
            client_id_on_a: msg.counterparty.client_id().clone(),
            conn_id_on_a: msg
                .counterparty
                .connection_id()
                .ok_or(ConnectionError::InvalidCounterparty)?
                .clone(),
        })
    }
}

/// Per our convention, this message is processed on chain B.
pub(crate) fn process(
    ctx_b: &dyn ConnectionReader,
    msg: MsgConnectionOpenTry,
) -> HandlerResult<ConnectionResult, ConnectionError> {
    let mut output = HandlerOutput::builder();

    let conn_id_on_b = ConnectionId::new(ctx_b.connection_counter()?);

    ctx_b.validate_self_client(msg.client_state_of_b_on_a.clone())?;

    if msg.consensus_height_of_b_on_a > ctx_b.host_current_height()? {
        // Fail if the consensus height is too advanced.
        return Err(ConnectionError::InvalidConsensusHeight {
            target_height: msg.consensus_height_of_b_on_a,
            current_height: ctx_b.host_current_height()?,
        });
    }

    let version_on_b = ctx_b.pick_version(&ctx_b.get_compatible_versions(), &msg.versions_on_a)?;

    let conn_end_on_b = ConnectionEnd::new(
        State::TryOpen,
        msg.client_id_on_b.clone(),
        msg.counterparty.clone(),
        vec![version_on_b],
        msg.delay_period,
    );

    let client_id_on_a = msg.counterparty.client_id();
    let conn_id_on_a = conn_end_on_b
        .counterparty()
        .connection_id()
        .ok_or(ConnectionError::InvalidCounterparty)?;

    // Verify proofs
    {
        let client_state_of_a_on_b = ctx_b.client_state(conn_end_on_b.client_id())?;
        let consensus_state_of_a_on_b =
            ctx_b.client_consensus_state(&msg.client_id_on_b, &msg.proofs_height_on_a)?;

        let prefix_on_a = conn_end_on_b.counterparty().prefix();
        let prefix_on_b = ctx_b.commitment_prefix();

        {
            let versions_on_a = msg.versions_on_a;
            let expected_conn_end_on_a = ConnectionEnd::new(
                State::Init,
                client_id_on_a.clone(),
                Counterparty::new(msg.client_id_on_b.clone(), None, prefix_on_b),
                versions_on_a,
                msg.delay_period,
            );

            client_state_of_a_on_b
                .verify_connection_state(
                    msg.proofs_height_on_a,
                    prefix_on_a,
                    &msg.proof_conn_end_on_a,
                    consensus_state_of_a_on_b.root(),
                    conn_id_on_a,
                    &expected_conn_end_on_a,
                )
                .map_err(ConnectionError::VerifyConnectionState)?;
        }

        client_state_of_a_on_b
            .verify_client_full_state(
                msg.proofs_height_on_a,
                prefix_on_a,
                &msg.proof_client_state_of_b_on_a,
                consensus_state_of_a_on_b.root(),
                client_id_on_a,
                msg.client_state_of_b_on_a,
            )
            .map_err(|e| ConnectionError::ClientStateVerificationFailure {
                client_id: conn_end_on_b.client_id().clone(),
                client_error: e,
            })?;

        let expected_consensus_state_of_b_on_a =
            ctx_b.host_consensus_state(&msg.consensus_height_of_b_on_a)?;
        client_state_of_a_on_b
            .verify_client_consensus_state(
                msg.proofs_height_on_a,
                prefix_on_a,
                &msg.proof_consensus_state_of_b_on_a,
                consensus_state_of_a_on_b.root(),
                client_id_on_a,
                msg.consensus_height_of_b_on_a,
                expected_consensus_state_of_b_on_a.as_ref(),
            )
            .map_err(|e| ConnectionError::ConsensusStateVerificationFailure {
                height: msg.proofs_height_on_a,
                client_error: e,
            })?;
    }

    // Success
    output.emit(IbcEvent::OpenTryConnection(OpenTry::new(
        conn_id_on_b.clone(),
        msg.client_id_on_b,
        conn_id_on_a.clone(),
        client_id_on_a.clone(),
    )));
    output.log("success: conn_open_try verification passed");

    let result = ConnectionResult {
        connection_id: conn_id_on_b,
        connection_end: conn_end_on_b,
        connection_id_state: ConnectionIdState::Generated,
    };

    Ok(output.with_result(result))
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    use test_log::test;

    use crate::core::ics03_connection::connection::State;
    use crate::core::ics03_connection::handler::{dispatch, ConnectionResult};
    use crate::core::ics03_connection::msgs::conn_open_try::test_util::get_dummy_raw_msg_conn_open_try;
    use crate::core::ics03_connection::msgs::conn_open_try::MsgConnectionOpenTry;
    use crate::core::ics03_connection::msgs::ConnectionMsg;
    use crate::core::ics24_host::identifier::ChainId;
    use crate::events::IbcEvent;
    use crate::mock::context::MockContext;
    use crate::mock::host::HostType;
    use crate::Height;

    #[cfg(feature = "val_exec_ctx")]
    use crate::core::ics26_routing::msgs::MsgEnvelope;
    #[cfg(feature = "val_exec_ctx")]
    use crate::core::ValidationContext;

    #[test]
    fn conn_open_try_msg_processing() {
        struct Test {
            name: String,
            ctx: MockContext,
            msg: ConnectionMsg,
            want_pass: bool,
        }

        let host_chain_height = Height::new(0, 35).unwrap();
        let max_history_size = 5;
        let context = MockContext::new(
            ChainId::new("mockgaia".to_string(), 0),
            HostType::Mock,
            max_history_size,
            host_chain_height,
        );
        let client_consensus_state_height = 10;

        let msg_conn_try = MsgConnectionOpenTry::try_from(get_dummy_raw_msg_conn_open_try(
            client_consensus_state_height,
            host_chain_height.revision_height(),
        ))
        .unwrap();

        // The proof targets a height that does not exist (i.e., too advanced) on destination chain.
        let msg_height_advanced = MsgConnectionOpenTry::try_from(get_dummy_raw_msg_conn_open_try(
            client_consensus_state_height,
            host_chain_height.increment().revision_height(),
        ))
        .unwrap();
        let pruned_height = host_chain_height
            .sub(max_history_size as u64 + 1)
            .unwrap()
            .revision_height();
        // The consensus proof targets a missing height (pruned) on destination chain.
        let msg_height_old = MsgConnectionOpenTry::try_from(get_dummy_raw_msg_conn_open_try(
            client_consensus_state_height,
            pruned_height,
        ))
        .unwrap();

        // The proofs in this message are created at a height which the client on destination chain does not have.
        let msg_proof_height_missing =
            MsgConnectionOpenTry::try_from(get_dummy_raw_msg_conn_open_try(
                client_consensus_state_height - 1,
                host_chain_height.revision_height(),
            ))
            .unwrap();

        let tests: Vec<Test> = vec![
            Test {
                name: "Processing fails because the height is too advanced".to_string(),
                ctx: context.clone(),
                msg: ConnectionMsg::Try(msg_height_advanced),
                want_pass: false,
            },
            Test {
                name: "Processing fails because the height is too old".to_string(),
                ctx: context.clone(),
                msg: ConnectionMsg::Try(msg_height_old),
                want_pass: false,
            },
            Test {
                name: "Processing fails because no client exists".to_string(),
                ctx: context.clone(),
                msg: ConnectionMsg::Try(msg_conn_try.clone()),
                want_pass: false,
            },
            Test {
                name: "Processing fails because the client misses the consensus state targeted by the proof".to_string(),
                ctx: context.clone().with_client(&msg_proof_height_missing.client_id_on_b, Height::new(0, client_consensus_state_height).unwrap()),
                msg: ConnectionMsg::Try(msg_proof_height_missing),
                want_pass: false,
            },
            Test {
                name: "Good parameters (no previous_connection_id)".to_string(),
                ctx: context.clone().with_client(&msg_conn_try.client_id_on_b, Height::new(0, client_consensus_state_height).unwrap()),
                msg: ConnectionMsg::Try(msg_conn_try.clone()),
                want_pass: true,
            },
            Test {
                name: "Good parameters".to_string(),
                ctx: context.with_client(&msg_conn_try.client_id_on_b, Height::new(0, client_consensus_state_height).unwrap()),
                msg: ConnectionMsg::Try(msg_conn_try),
                want_pass: true,
            },
        ]
        .into_iter()
        .collect();

        for test in tests {
            #[cfg(feature = "val_exec_ctx")]
            {
                let res = ValidationContext::validate(
                    &test.ctx,
                    MsgEnvelope::ConnectionMsg(test.msg.clone()),
                );

                match res {
                    Ok(_) => {
                        assert!(
                        test.want_pass,
                        "conn_open_try: test passed but was supposed to fail for test: {}, \nparams {:?} {:?}",
                        test.name,
                        test.msg.clone(),
                        test.ctx.clone()
                    )
                    }
                    Err(e) => {
                        assert!(
                            !test.want_pass,
                            "conn_open_try: did not pass test: {}, \nparams {:?} {:?} error: {:?}",
                            test.name,
                            test.msg,
                            test.ctx.clone(),
                            e,
                        );
                    }
                }
            }
            let res = dispatch(&test.ctx, test.msg.clone());
            // Additionally check the events and the output objects in the result.
            match res {
                Ok(proto_output) => {
                    assert!(
                        test.want_pass,
                        "conn_open_try: test passed but was supposed to fail for test: {}, \nparams {:?} {:?}",
                        test.name,
                        test.msg.clone(),
                        test.ctx.clone()
                    );

                    assert!(!proto_output.events.is_empty()); // Some events must exist.

                    // The object in the output is a ConnectionEnd, should have TryOpen state.
                    let res: ConnectionResult = proto_output.result;
                    assert_eq!(res.connection_end.state().clone(), State::TryOpen);

                    for e in proto_output.events.iter() {
                        assert!(matches!(e, &IbcEvent::OpenTryConnection(_)));
                    }
                }
                Err(e) => {
                    assert!(
                        !test.want_pass,
                        "conn_open_try: failed for test: {}, \nparams {:?} {:?} error: {:?}",
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
