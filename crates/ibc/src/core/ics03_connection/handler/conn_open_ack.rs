//! Protocol logic specific to processing ICS3 messages of type `MsgConnectionOpenAck`.
use crate::prelude::*;

use crate::core::ics03_connection::connection::{ConnectionEnd, Counterparty, State};
use crate::core::ics03_connection::error::ConnectionError;
use crate::core::ics03_connection::events::OpenAck;
use crate::core::ics03_connection::msgs::conn_open_ack::MsgConnectionOpenAck;
use crate::events::IbcEvent;

use crate::core::context::ContextError;

use crate::core::ics24_host::identifier::ClientId;

use crate::core::ics24_host::path::{ClientConsensusStatePath, ClientStatePath, ConnectionPath};

use crate::core::{ExecutionContext, ValidationContext};

pub(crate) fn validate<Ctx>(ctx_a: &Ctx, msg: MsgConnectionOpenAck) -> Result<(), ContextError>
where
    Ctx: ValidationContext,
{
    let vars = LocalVars::new(ctx_a, &msg)?;
    validate_impl(ctx_a, &msg, &vars)
}

fn validate_impl<Ctx>(
    ctx_a: &Ctx,
    msg: &MsgConnectionOpenAck,
    vars: &LocalVars,
) -> Result<(), ContextError>
where
    Ctx: ValidationContext,
{
    let host_height = ctx_a.host_height().map_err(|_| ConnectionError::Other {
        description: "failed to get host height".to_string(),
    })?;
    if msg.consensus_height_of_a_on_b > host_height {
        return Err(ConnectionError::InvalidConsensusHeight {
            target_height: msg.consensus_height_of_a_on_b,
            current_height: host_height,
        }
        .into());
    }

    ctx_a.validate_self_client(msg.client_state_of_a_on_b.clone())?;

    if !(vars.conn_end_on_a.state_matches(&State::Init)
        && vars.conn_end_on_a.versions().contains(&msg.version))
    {
        return Err(ConnectionError::ConnectionMismatch {
            connection_id: msg.conn_id_on_a.clone(),
        }
        .into());
    }

    // Proof verification.
    {
        let client_state_of_b_on_a =
            ctx_a
                .client_state(vars.client_id_on_a())
                .map_err(|_| ConnectionError::Other {
                    description: "failed to fetch client state".to_string(),
                })?;
        let client_cons_state_path_on_a =
            ClientConsensusStatePath::new(vars.client_id_on_a(), &msg.proofs_height_on_b);
        let consensus_state_of_b_on_a = ctx_a
            .consensus_state(&client_cons_state_path_on_a)
            .map_err(|_| ConnectionError::Other {
                description: "failed to fetch client consensus state".to_string(),
            })?;

        let prefix_on_a = ctx_a.commitment_prefix();
        let prefix_on_b = vars.conn_end_on_a.counterparty().prefix();

        {
            let expected_conn_end_on_b = ConnectionEnd::new(
                State::TryOpen,
                vars.client_id_on_b().clone(),
                Counterparty::new(
                    vars.client_id_on_a().clone(),
                    Some(msg.conn_id_on_a.clone()),
                    prefix_on_a,
                ),
                vec![msg.version.clone()],
                vars.conn_end_on_a.delay_period(),
            );

            client_state_of_b_on_a
                .verify_connection_state(
                    msg.proofs_height_on_b,
                    prefix_on_b,
                    &msg.proof_conn_end_on_b,
                    consensus_state_of_b_on_a.root(),
                    &ConnectionPath::new(&msg.conn_id_on_b),
                    &expected_conn_end_on_b,
                )
                .map_err(ConnectionError::VerifyConnectionState)?;
        }

        client_state_of_b_on_a
            .verify_client_full_state(
                msg.proofs_height_on_b,
                prefix_on_b,
                &msg.proof_client_state_of_a_on_b,
                consensus_state_of_b_on_a.root(),
                &ClientStatePath::new(vars.client_id_on_a()),
                msg.client_state_of_a_on_b.clone(),
            )
            .map_err(|e| ConnectionError::ClientStateVerificationFailure {
                client_id: vars.client_id_on_a().clone(),
                client_error: e,
            })?;

        let expected_consensus_state_of_a_on_b = ctx_a
            .host_consensus_state(&msg.consensus_height_of_a_on_b)
            .map_err(|_| ConnectionError::Other {
                description: "failed to fetch host consensus state".to_string(),
            })?;

        let client_cons_state_path_on_b =
            ClientConsensusStatePath::new(vars.client_id_on_b(), &msg.consensus_height_of_a_on_b);

        client_state_of_b_on_a
            .verify_client_consensus_state(
                msg.proofs_height_on_b,
                prefix_on_b,
                &msg.proof_consensus_state_of_a_on_b,
                consensus_state_of_b_on_a.root(),
                &client_cons_state_path_on_b,
                expected_consensus_state_of_a_on_b.as_ref(),
            )
            .map_err(|e| ConnectionError::ConsensusStateVerificationFailure {
                height: msg.proofs_height_on_b,
                client_error: e,
            })?;
    }

    Ok(())
}

pub(crate) fn execute<Ctx>(ctx_a: &mut Ctx, msg: MsgConnectionOpenAck) -> Result<(), ContextError>
where
    Ctx: ExecutionContext,
{
    let vars = LocalVars::new(ctx_a, &msg)?;
    execute_impl(ctx_a, msg, vars)
}

fn execute_impl<Ctx>(
    ctx_a: &mut Ctx,
    msg: MsgConnectionOpenAck,
    vars: LocalVars,
) -> Result<(), ContextError>
where
    Ctx: ExecutionContext,
{
    ctx_a.emit_ibc_event(IbcEvent::OpenAckConnection(OpenAck::new(
        msg.conn_id_on_a.clone(),
        vars.client_id_on_a().clone(),
        msg.conn_id_on_b.clone(),
        vars.client_id_on_b().clone(),
    )));

    ctx_a.log_message("success: conn_open_ack verification passed".to_string());

    {
        let new_conn_end_on_a = {
            let mut counterparty = vars.conn_end_on_a.counterparty().clone();
            counterparty.connection_id = Some(msg.conn_id_on_b.clone());

            let mut new_conn_end_on_a = vars.conn_end_on_a;
            new_conn_end_on_a.set_state(State::Open);
            new_conn_end_on_a.set_version(msg.version.clone());
            new_conn_end_on_a.set_counterparty(counterparty);
            new_conn_end_on_a
        };

        ctx_a.store_connection(&ConnectionPath::new(&msg.conn_id_on_a), new_conn_end_on_a)?;
    }

    Ok(())
}

struct LocalVars {
    conn_end_on_a: ConnectionEnd,
}

impl LocalVars {
    fn new<Ctx>(ctx_a: &Ctx, msg: &MsgConnectionOpenAck) -> Result<Self, ContextError>
    where
        Ctx: ValidationContext,
    {
        Ok(LocalVars {
            conn_end_on_a: ctx_a.connection_end(&msg.conn_id_on_a)?,
        })
    }

    fn client_id_on_a(&self) -> &ClientId {
        self.conn_end_on_a.client_id()
    }

    fn client_id_on_b(&self) -> &ClientId {
        self.conn_end_on_a.counterparty().client_id()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use core::str::FromStr;
    use test_log::test;

    use crate::core::ics02_client::height::Height;
    use crate::core::ics03_connection::connection::{ConnectionEnd, Counterparty, State};
    use crate::core::ics03_connection::handler::test_util::{Expect, Fixture};
    use crate::core::ics03_connection::msgs::conn_open_ack::MsgConnectionOpenAck;
    use crate::core::ics23_commitment::commitment::CommitmentPrefix;
    use crate::core::ics24_host::identifier::{ChainId, ClientId};
    use crate::core::ValidationContext;

    use crate::events::IbcEvent;
    use crate::mock::context::MockContext;
    use crate::mock::host::HostType;
    use crate::timestamp::ZERO_DURATION;

    enum Ctx {
        New,
        NewWithConnection,
        NewWithConnectionEndOpen,
        DefaultWithConnection,
    }

    fn conn_open_ack_fixture(ctx: Ctx) -> Fixture<MsgConnectionOpenAck> {
        let msg = MsgConnectionOpenAck::new_dummy(10, 10);

        // Client parameters -- identifier and correct height (matching the proof height)
        let client_id = ClientId::from_str("mock_clientid").unwrap();
        let proof_height = msg.proofs_height_on_b;
        let conn_id = msg.conn_id_on_a.clone();

        // Parametrize the host chain to have a height at least as recent as the
        // the height of the proofs in the Ack msg.
        let latest_height = proof_height.increment();
        let max_history_size = 5;

        // A connection end that will exercise the successful path.
        let default_conn_end = ConnectionEnd::new(
            State::Init,
            client_id.clone(),
            Counterparty::new(
                client_id.clone(),
                Some(msg.conn_id_on_b.clone()),
                CommitmentPrefix::try_from(b"ibc".to_vec()).unwrap(),
            ),
            vec![msg.version.clone()],
            ZERO_DURATION,
        );

        // A connection end with incorrect state `Open`; will be part of the context.
        let mut conn_end_open = default_conn_end.clone();
        conn_end_open.set_state(State::Open); // incorrect field

        let ctx_default = MockContext::default();
        let ctx_new = MockContext::new(
            ChainId::new("mockgaia".to_string(), latest_height.revision_number()),
            HostType::Mock,
            max_history_size,
            latest_height,
        );
        let ctx = match ctx {
            Ctx::New => ctx_new,
            Ctx::NewWithConnection => ctx_new
                .with_client(&client_id, proof_height)
                .with_connection(conn_id, default_conn_end),
            Ctx::DefaultWithConnection => ctx_default
                .with_client(&client_id, proof_height)
                .with_connection(conn_id, default_conn_end),
            Ctx::NewWithConnectionEndOpen => ctx_new
                .with_client(&client_id, proof_height)
                .with_connection(conn_id, conn_end_open),
        };

        Fixture { ctx, msg }
    }

    fn conn_open_ack_validate(fxt: &Fixture<MsgConnectionOpenAck>, expect: Expect) {
        let res = validate(&fxt.ctx, fxt.msg.clone());
        let err_msg = fxt.generate_error_msg(&expect, "validation", &res);
        match expect {
            Expect::Failure(err) => {
                assert!(res.is_err(), "{err_msg}");
                assert_eq!(
                    core::mem::discriminant(res.as_ref().unwrap_err()),
                    core::mem::discriminant(&err.unwrap())
                );
            }
            Expect::Success => {
                assert!(res.is_ok(), "{err_msg}");
                return;
            }
        };
        let right_connection_id = fxt.msg.conn_id_on_a.clone();
        let cons_state_height = fxt.msg.consensus_height_of_a_on_b;
        match res.unwrap_err() {
            ContextError::ConnectionError(ConnectionError::ConnectionNotFound {
                connection_id,
            }) => {
                assert_eq!(connection_id, right_connection_id)
            }
            ContextError::ConnectionError(ConnectionError::InvalidConsensusHeight {
                target_height,
                current_height: _,
            }) => {
                assert_eq!(cons_state_height, target_height);
            }
            ContextError::ConnectionError(ConnectionError::ConnectionMismatch {
                connection_id,
            }) => {
                assert_eq!(connection_id, right_connection_id)
            }
            _ => unreachable!(),
        }
    }

    fn conn_open_ack_execute(fxt: &mut Fixture<MsgConnectionOpenAck>, expect: Expect) {
        let res = execute(&mut fxt.ctx, fxt.msg.clone());
        let err_msg = fxt.generate_error_msg(&expect, "execution", &res);
        match expect {
            Expect::Failure(_) => {
                assert!(res.is_err(), "{err_msg}");
            }
            Expect::Success => {
                assert!(res.is_ok(), "{err_msg}");
                assert_eq!(fxt.ctx.events.len(), 1);

                let event = fxt.ctx.events.first().unwrap();
                assert!(matches!(event, &IbcEvent::OpenAckConnection(_)));

                let conn_open_try_event = match event {
                    IbcEvent::OpenAckConnection(e) => e,
                    _ => unreachable!(),
                };
                let conn_end = <MockContext as ValidationContext>::connection_end(
                    &fxt.ctx,
                    conn_open_try_event.connection_id(),
                )
                .unwrap();
                assert_eq!(conn_end.state().clone(), State::Open);
            }
        }
    }

    #[test]
    fn conn_open_ack_healthy() {
        let mut fxt = conn_open_ack_fixture(Ctx::NewWithConnection);
        conn_open_ack_validate(&fxt, Expect::Success);
        conn_open_ack_execute(&mut fxt, Expect::Success);
    }

    #[test]
    fn conn_open_ack_no_connection() {
        let fxt = conn_open_ack_fixture(Ctx::New);
        let expected_err = ContextError::ConnectionError(ConnectionError::ConnectionNotFound {
            connection_id: fxt.msg.conn_id_on_a.clone(),
        });
        conn_open_ack_validate(&fxt, Expect::Failure(Some(expected_err)));
    }

    #[test]
    fn conn_open_ack_invalid_consensus_height() {
        let fxt = conn_open_ack_fixture(Ctx::DefaultWithConnection);
        let expected_err = ContextError::ConnectionError(ConnectionError::InvalidConsensusHeight {
            target_height: fxt.msg.consensus_height_of_a_on_b,
            current_height: Height::new(0, 10).unwrap(),
        });
        conn_open_ack_validate(&fxt, Expect::Failure(Some(expected_err)));
    }

    #[test]
    fn conn_open_ack_connection_mismatch() {
        let fxt = conn_open_ack_fixture(Ctx::NewWithConnectionEndOpen);
        let expected_err = ContextError::ConnectionError(ConnectionError::ConnectionMismatch {
            connection_id: fxt.msg.conn_id_on_a.clone(),
        });
        conn_open_ack_validate(&fxt, Expect::Failure(Some(expected_err)));
    }
}
