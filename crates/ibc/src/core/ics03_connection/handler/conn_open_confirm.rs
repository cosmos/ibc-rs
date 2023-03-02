//! Protocol logic specific to processing ICS3 messages of type `MsgConnectionOpenConfirm`.
use crate::prelude::*;

use crate::core::ics03_connection::connection::{ConnectionEnd, Counterparty, State};
use crate::core::ics03_connection::error::ConnectionError;
use crate::core::ics03_connection::events::OpenConfirm;
use crate::core::ics03_connection::msgs::conn_open_confirm::MsgConnectionOpenConfirm;
use crate::events::IbcEvent;

use crate::core::context::ContextError;

use crate::core::ics24_host::identifier::{ClientId, ConnectionId};

use crate::core::ics24_host::path::{ClientConsensusStatePath, ConnectionPath};

use crate::core::{ExecutionContext, ValidationContext};

pub(crate) fn validate<Ctx>(ctx_b: &Ctx, msg: &MsgConnectionOpenConfirm) -> Result<(), ContextError>
where
    Ctx: ValidationContext,
{
    let vars = LocalVars::new(ctx_b, msg)?;
    validate_impl(ctx_b, msg, &vars)
}

fn validate_impl<Ctx>(
    ctx_b: &Ctx,
    msg: &MsgConnectionOpenConfirm,
    vars: &LocalVars,
) -> Result<(), ContextError>
where
    Ctx: ValidationContext,
{
    let conn_end_on_b = vars.conn_end_on_b();
    if !conn_end_on_b.state_matches(&State::TryOpen) {
        return Err(ConnectionError::ConnectionMismatch {
            connection_id: msg.conn_id_on_b.clone(),
        }
        .into());
    }

    let client_id_on_a = vars.client_id_on_a();
    let client_id_on_b = vars.client_id_on_b();
    let conn_id_on_a = vars.conn_id_on_a()?;

    // Verify proofs
    {
        let client_state_of_a_on_b =
            ctx_b
                .client_state(client_id_on_b)
                .map_err(|_| ConnectionError::Other {
                    description: "failed to fetch client state".to_string(),
                })?;
        let client_cons_state_path_on_b =
            ClientConsensusStatePath::new(client_id_on_b, &msg.proof_height_on_a);
        let consensus_state_of_a_on_b = ctx_b
            .consensus_state(&client_cons_state_path_on_b)
            .map_err(|_| ConnectionError::Other {
                description: "failed to fetch client consensus state".to_string(),
            })?;

        let prefix_on_a = conn_end_on_b.counterparty().prefix();
        let prefix_on_b = ctx_b.commitment_prefix();

        let expected_conn_end_on_a = ConnectionEnd::new(
            State::Open,
            client_id_on_a.clone(),
            Counterparty::new(
                client_id_on_b.clone(),
                Some(msg.conn_id_on_b.clone()),
                prefix_on_b,
            ),
            conn_end_on_b.versions().to_vec(),
            conn_end_on_b.delay_period(),
        );

        client_state_of_a_on_b
            .verify_connection_state(
                msg.proof_height_on_a,
                prefix_on_a,
                &msg.proof_conn_end_on_a,
                consensus_state_of_a_on_b.root(),
                &ConnectionPath::new(conn_id_on_a),
                &expected_conn_end_on_a,
            )
            .map_err(ConnectionError::VerifyConnectionState)?;
    }

    Ok(())
}

pub(crate) fn execute<Ctx>(
    ctx_b: &mut Ctx,
    msg: &MsgConnectionOpenConfirm,
) -> Result<(), ContextError>
where
    Ctx: ExecutionContext,
{
    let vars = LocalVars::new(ctx_b, msg)?;
    execute_impl(ctx_b, msg, vars)
}

fn execute_impl<Ctx>(
    ctx_b: &mut Ctx,
    msg: &MsgConnectionOpenConfirm,
    vars: LocalVars,
) -> Result<(), ContextError>
where
    Ctx: ExecutionContext,
{
    let client_id_on_a = vars.client_id_on_a();
    let client_id_on_b = vars.client_id_on_b();
    let conn_id_on_a = vars.conn_id_on_a()?;

    ctx_b.emit_ibc_event(IbcEvent::OpenConfirmConnection(OpenConfirm::new(
        msg.conn_id_on_b.clone(),
        client_id_on_b.clone(),
        conn_id_on_a.clone(),
        client_id_on_a.clone(),
    )));
    ctx_b.log_message("success: conn_open_confirm verification passed".to_string());

    {
        let new_conn_end_on_b = {
            let mut new_conn_end_on_b = vars.conn_end_on_b;

            new_conn_end_on_b.set_state(State::Open);
            new_conn_end_on_b
        };

        ctx_b.store_connection(&ConnectionPath(msg.conn_id_on_b.clone()), new_conn_end_on_b)?;
    }

    Ok(())
}

struct LocalVars {
    conn_end_on_b: ConnectionEnd,
}

impl LocalVars {
    fn new<Ctx>(ctx_b: &Ctx, msg: &MsgConnectionOpenConfirm) -> Result<Self, ContextError>
    where
        Ctx: ValidationContext,
    {
        Ok(Self {
            conn_end_on_b: ctx_b.connection_end(&msg.conn_id_on_b)?,
        })
    }

    fn conn_end_on_b(&self) -> &ConnectionEnd {
        &self.conn_end_on_b
    }

    fn client_id_on_a(&self) -> &ClientId {
        self.conn_end_on_b.counterparty().client_id()
    }

    fn client_id_on_b(&self) -> &ClientId {
        self.conn_end_on_b.client_id()
    }

    fn conn_id_on_a(&self) -> Result<&ConnectionId, ConnectionError> {
        self.conn_end_on_b
            .counterparty()
            .connection_id()
            .ok_or(ConnectionError::InvalidCounterparty)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use core::str::FromStr;
    use test_log::test;

    use crate::core::ics03_connection::connection::{ConnectionEnd, Counterparty, State};
    use crate::core::ics03_connection::handler::test_util::{Expect, Fixture};
    use crate::core::ics03_connection::msgs::conn_open_confirm::MsgConnectionOpenConfirm;
    use crate::core::ics23_commitment::commitment::CommitmentPrefix;
    use crate::core::ics24_host::identifier::ClientId;
    use crate::events::IbcEvent;
    use crate::mock::context::MockContext;
    use crate::timestamp::ZERO_DURATION;
    use crate::Height;

    use crate::core::ValidationContext;

    enum Ctx {
        Default,
        CorrectConnection,
        IncorrectConnection,
    }

    fn conn_open_confirm_fixture(ctx: Ctx) -> Fixture<MsgConnectionOpenConfirm> {
        let client_id = ClientId::from_str("mock_clientid").unwrap();
        let msg = MsgConnectionOpenConfirm::new_dummy();
        let counterparty = Counterparty::new(
            client_id.clone(),
            Some(msg.conn_id_on_b.clone()),
            CommitmentPrefix::try_from(b"ibc".to_vec()).unwrap(),
        );

        let ctx_default = MockContext::default();

        let incorrect_conn_end_state = ConnectionEnd::new(
            State::Init,
            client_id.clone(),
            counterparty,
            ValidationContext::get_compatible_versions(&ctx_default),
            ZERO_DURATION,
        );

        let mut correct_conn_end = incorrect_conn_end_state.clone();
        correct_conn_end.set_state(State::TryOpen);

        let ctx = match ctx {
            Ctx::Default => ctx_default,
            Ctx::IncorrectConnection => ctx_default
                .with_client(&client_id, Height::new(0, 10).unwrap())
                .with_connection(msg.conn_id_on_b.clone(), incorrect_conn_end_state),
            Ctx::CorrectConnection => ctx_default
                .with_client(&client_id, Height::new(0, 10).unwrap())
                .with_connection(msg.conn_id_on_b.clone(), correct_conn_end),
        };

        Fixture { ctx, msg }
    }

    fn conn_open_confirm_validate(fxt: &Fixture<MsgConnectionOpenConfirm>, expect: Expect) {
        let res = validate(&fxt.ctx, &fxt.msg);
        let err_msg = fxt.generate_error_msg(&expect, "validation", &res);
        match expect {
            Expect::Failure(_) => {
                assert!(res.is_err(), "{err_msg}");
            }
            Expect::Success => {
                assert!(res.is_ok(), "{err_msg}");
            }
        };
    }

    fn conn_open_confirm_execute(fxt: &mut Fixture<MsgConnectionOpenConfirm>, expect: Expect) {
        let res = execute(&mut fxt.ctx, &fxt.msg);
        let err_msg = fxt.generate_error_msg(&expect, "execution", &res);
        match expect {
            Expect::Failure(_) => {
                assert!(res.is_err(), "{err_msg}");
            }
            Expect::Success => {
                assert!(res.is_ok(), "{err_msg}");
                assert_eq!(fxt.ctx.events.len(), 1);

                let event = fxt.ctx.events.first().unwrap();
                assert!(matches!(event, &IbcEvent::OpenConfirmConnection(_)));

                let conn_open_try_event = match event {
                    IbcEvent::OpenConfirmConnection(e) => e,
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
    fn conn_open_confirm_healthy() {
        let mut fxt = conn_open_confirm_fixture(Ctx::CorrectConnection);
        conn_open_confirm_validate(&fxt, Expect::Success);
        conn_open_confirm_execute(&mut fxt, Expect::Success);
    }

    #[test]
    fn conn_open_confirm_no_connection() {
        let fxt = conn_open_confirm_fixture(Ctx::Default);
        conn_open_confirm_validate(&fxt, Expect::Failure(None));
    }

    #[test]
    fn conn_open_confirm_connection_mismatch() {
        let fxt = conn_open_confirm_fixture(Ctx::IncorrectConnection);
        conn_open_confirm_validate(&fxt, Expect::Failure(None));
    }
}
