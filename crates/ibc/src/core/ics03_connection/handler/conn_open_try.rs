//! Protocol logic specific to processing ICS3 messages of type `MsgConnectionOpenTry`.
use crate::prelude::*;

use crate::core::ics03_connection::connection::{ConnectionEnd, Counterparty, State};
use crate::core::ics03_connection::error::ConnectionError;
use crate::core::ics03_connection::events::OpenTry;
use crate::core::ics03_connection::msgs::conn_open_try::MsgConnectionOpenTry;
use crate::core::ics24_host::identifier::ConnectionId;
use crate::events::IbcEvent;

use crate::core::context::ContextError;

use crate::core::ics24_host::identifier::ClientId;

use crate::core::ics24_host::path::{
    ClientConnectionPath, ClientConsensusStatePath, ClientStatePath, ConnectionPath,
};

use crate::core::{ExecutionContext, ValidationContext};

pub(crate) fn validate<Ctx>(ctx_b: &Ctx, msg: MsgConnectionOpenTry) -> Result<(), ContextError>
where
    Ctx: ValidationContext,
{
    let vars = LocalVars::new(ctx_b, &msg)?;
    validate_impl(ctx_b, &msg, &vars)
}

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
        let client_cons_state_path_on_b =
            ClientConsensusStatePath::new(&msg.client_id_on_b, &msg.proofs_height_on_a);
        let consensus_state_of_a_on_b = ctx_b
            .consensus_state(&client_cons_state_path_on_b)
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
                    &ConnectionPath::new(&vars.conn_id_on_a),
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
                &ClientStatePath::new(client_id_on_a),
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

        let client_cons_state_path_on_a =
            ClientConsensusStatePath::new(&msg.client_id_on_b, &msg.consensus_height_of_b_on_a);
        client_state_of_a_on_b
            .verify_client_consensus_state(
                msg.proofs_height_on_a,
                prefix_on_a,
                &msg.proof_consensus_state_of_b_on_a,
                consensus_state_of_a_on_b.root(),
                &client_cons_state_path_on_a,
                expected_consensus_state_of_b_on_a.as_ref(),
            )
            .map_err(|e| ConnectionError::ClientStateVerificationFailure {
                client_id: client_id_on_a.clone(),
                client_error: e,
            })?;
    }

    Ok(())
}

pub(crate) fn execute<Ctx>(ctx_b: &mut Ctx, msg: MsgConnectionOpenTry) -> Result<(), ContextError>
where
    Ctx: ExecutionContext,
{
    let vars = LocalVars::new(ctx_b, &msg)?;
    execute_impl(ctx_b, msg, vars)
}

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
        &ClientConnectionPath::new(&msg.client_id_on_b),
        vars.conn_id_on_b.clone(),
    )?;
    ctx_b.store_connection(&ConnectionPath::new(&vars.conn_id_on_b), vars.conn_end_on_b)?;

    Ok(())
}

struct LocalVars {
    conn_id_on_b: ConnectionId,
    conn_end_on_b: ConnectionEnd,
    client_id_on_a: ClientId,
    conn_id_on_a: ConnectionId,
}

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

#[cfg(test)]
mod tests {
    use super::*;

    use test_log::test;

    use crate::core::ics03_connection::connection::State;
    use crate::core::ics03_connection::handler::test_util::{Expect, Fixture};
    use crate::core::ics03_connection::msgs::conn_open_try::MsgConnectionOpenTry;
    use crate::core::ics24_host::identifier::ChainId;
    use crate::core::ValidationContext;
    use crate::events::IbcEvent;
    use crate::mock::context::MockContext;
    use crate::mock::host::HostType;
    use crate::Height;

    enum Ctx {
        Default,
        WithClient,
    }

    enum Msg {
        Default,
        HeightAdvanced,
        HeightOld,
        ProofHeightMissing,
    }

    fn conn_open_try_fixture(ctx_variant: Ctx, msg_variant: Msg) -> Fixture<MsgConnectionOpenTry> {
        let max_history_size = 5;
        let client_cons_state_height = 10;
        let host_chain_height = Height::new(0, 35).unwrap();
        let pruned_height = host_chain_height
            .sub(max_history_size as u64 + 1)
            .unwrap()
            .revision_height();

        let msg = match msg_variant {
            Msg::Default => MsgConnectionOpenTry::new_dummy(
                client_cons_state_height,
                host_chain_height.revision_height(),
            ),
            Msg::HeightAdvanced => MsgConnectionOpenTry::new_dummy(
                client_cons_state_height,
                host_chain_height.increment().revision_height(),
            ),
            Msg::HeightOld => {
                MsgConnectionOpenTry::new_dummy(client_cons_state_height, pruned_height)
            }
            Msg::ProofHeightMissing => MsgConnectionOpenTry::new_dummy(
                client_cons_state_height - 1,
                host_chain_height.revision_height(),
            ),
        };

        let ctx_new = MockContext::new(
            ChainId::new("mockgaia".to_string(), 0),
            HostType::Mock,
            max_history_size,
            host_chain_height,
        );
        let ctx = match ctx_variant {
            Ctx::Default => MockContext::default(),
            Ctx::WithClient => ctx_new.with_client(
                &msg.client_id_on_b,
                Height::new(0, client_cons_state_height).unwrap(),
            ),
        };
        Fixture { ctx, msg }
    }

    fn conn_open_try_validate(fxt: &Fixture<MsgConnectionOpenTry>, expect: Expect) {
        let res = validate(&fxt.ctx, fxt.msg.clone());
        let err_msg = fxt.generate_error_msg(&expect, "validation", &res);
        match expect {
            Expect::Failure(_) => {
                assert!(res.is_err(), "{err_msg}")
            }
            Expect::Success => {
                assert!(res.is_ok(), "{err_msg}");
            }
        }
    }

    fn conn_open_try_execute(fxt: &mut Fixture<MsgConnectionOpenTry>, expect: Expect) {
        let res = execute(&mut fxt.ctx, fxt.msg.clone());
        let err_msg = fxt.generate_error_msg(&expect, "execution", &res);
        match expect {
            Expect::Failure(_) => {
                assert!(res.is_err(), "{err_msg}")
            }
            Expect::Success => {
                assert!(res.is_ok(), "{err_msg}");
                assert_eq!(fxt.ctx.events.len(), 1);

                let event = fxt.ctx.events.first().unwrap();
                assert!(matches!(event, &IbcEvent::OpenTryConnection(_)));

                let conn_open_try_event = match event {
                    IbcEvent::OpenTryConnection(e) => e,
                    _ => unreachable!(),
                };
                let conn_end = <MockContext as ValidationContext>::connection_end(
                    &fxt.ctx,
                    conn_open_try_event.connection_id(),
                )
                .unwrap();
                assert_eq!(conn_end.state().clone(), State::TryOpen);
            }
        }
    }

    #[test]
    fn conn_open_try_healthy() {
        let mut fxt = conn_open_try_fixture(Ctx::WithClient, Msg::Default);
        conn_open_try_validate(&fxt, Expect::Success);
        conn_open_try_execute(&mut fxt, Expect::Success);
    }

    #[test]
    fn conn_open_try_height_advanced() {
        let fxt = conn_open_try_fixture(Ctx::WithClient, Msg::HeightAdvanced);
        conn_open_try_validate(&fxt, Expect::Failure(None));
    }

    #[test]
    fn conn_open_try_height_old() {
        let fxt = conn_open_try_fixture(Ctx::WithClient, Msg::HeightOld);
        conn_open_try_validate(&fxt, Expect::Failure(None));
    }

    #[test]
    fn conn_open_try_proof_height_missing() {
        let fxt = conn_open_try_fixture(Ctx::WithClient, Msg::ProofHeightMissing);
        conn_open_try_validate(&fxt, Expect::Failure(None));
    }

    #[test]
    fn conn_open_try_no_client() {
        let fxt = conn_open_try_fixture(Ctx::Default, Msg::Default);
        conn_open_try_validate(&fxt, Expect::Failure(None));
    }
}
