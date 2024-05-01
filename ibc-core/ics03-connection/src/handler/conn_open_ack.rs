//! Protocol logic specific to processing ICS3 messages of type `MsgConnectionOpenAck`.

use ibc_core_client::context::prelude::*;
use ibc_core_client::types::error::ClientError;
use ibc_core_connection_types::error::ConnectionError;
use ibc_core_connection_types::events::OpenAck;
use ibc_core_connection_types::msgs::MsgConnectionOpenAck;
use ibc_core_connection_types::{ConnectionEnd, Counterparty, State};
use ibc_core_handler_types::error::ContextError;
use ibc_core_handler_types::events::{IbcEvent, MessageEvent};
use ibc_core_host::types::identifiers::ClientId;
use ibc_core_host::types::path::{ClientConsensusStatePath, ClientStatePath, ConnectionPath, Path};
use ibc_core_host::{ExecutionContext, ValidationContext};
use ibc_primitives::prelude::*;
use ibc_primitives::proto::{Any, Protobuf};
use ibc_primitives::ToVec;

pub fn validate<Ctx>(ctx_a: &Ctx, msg: MsgConnectionOpenAck) -> Result<(), ContextError>
where
    Ctx: ValidationContext,
    <Ctx::HostClientState as TryFrom<Any>>::Error: Into<ClientError>,
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
    <Ctx::HostClientState as TryFrom<Any>>::Error: Into<ClientError>,
{
    ctx_a.validate_message_signer(&msg.signer)?;

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

    let client_val_ctx_a = ctx_a.get_client_validation_context();

    let client_state_of_a_on_b =
        Ctx::HostClientState::try_from(msg.client_state_of_a_on_b.clone()).map_err(Into::into)?;

    ctx_a.validate_self_client(client_state_of_a_on_b)?;

    msg.version
        .verify_is_supported(vars.conn_end_on_a.versions())?;

    vars.conn_end_on_a.verify_state_matches(&State::Init)?;

    // Proof verification.
    {
        let client_state_of_b_on_a = client_val_ctx_a.client_state(vars.client_id_on_a())?;

        client_state_of_b_on_a
            .status(client_val_ctx_a, vars.client_id_on_a())?
            .verify_is_active()?;
        client_state_of_b_on_a.validate_proof_height(msg.proofs_height_on_b)?;

        let client_cons_state_path_on_a = ClientConsensusStatePath::new(
            vars.client_id_on_a().clone(),
            msg.proofs_height_on_b.revision_number(),
            msg.proofs_height_on_b.revision_height(),
        );

        let consensus_state_of_b_on_a =
            client_val_ctx_a.consensus_state(&client_cons_state_path_on_a)?;

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
            )?;

            client_state_of_b_on_a
                .verify_membership(
                    prefix_on_b,
                    &msg.proof_conn_end_on_b,
                    consensus_state_of_b_on_a.root(),
                    Path::Connection(ConnectionPath::new(&msg.conn_id_on_b)),
                    expected_conn_end_on_b.encode_vec(),
                )
                .map_err(ConnectionError::VerifyConnectionState)?;
        }

        client_state_of_b_on_a
            .verify_membership(
                prefix_on_b,
                &msg.proof_client_state_of_a_on_b,
                consensus_state_of_b_on_a.root(),
                Path::ClientState(ClientStatePath::new(vars.client_id_on_b().clone())),
                msg.client_state_of_a_on_b.to_vec(),
            )
            .map_err(|e| ConnectionError::ClientStateVerificationFailure {
                client_id: vars.client_id_on_b().clone(),
                client_error: e,
            })?;

        let expected_consensus_state_of_a_on_b =
            ctx_a.host_consensus_state(&msg.consensus_height_of_a_on_b)?;

        let client_cons_state_path_on_b = ClientConsensusStatePath::new(
            vars.client_id_on_b().clone(),
            msg.consensus_height_of_a_on_b.revision_number(),
            msg.consensus_height_of_a_on_b.revision_height(),
        );

        client_state_of_b_on_a
            .verify_membership(
                prefix_on_b,
                &msg.proof_consensus_state_of_a_on_b,
                consensus_state_of_b_on_a.root(),
                Path::ClientConsensusState(client_cons_state_path_on_b),
                expected_consensus_state_of_a_on_b.into().to_vec(),
            )
            .map_err(|e| ConnectionError::ConsensusStateVerificationFailure {
                height: msg.proofs_height_on_b,
                client_error: e,
            })?;
    }

    Ok(())
}

pub fn execute<Ctx>(ctx_a: &mut Ctx, msg: MsgConnectionOpenAck) -> Result<(), ContextError>
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
    let event = IbcEvent::OpenAckConnection(OpenAck::new(
        msg.conn_id_on_a.clone(),
        vars.client_id_on_a().clone(),
        msg.conn_id_on_b.clone(),
        vars.client_id_on_b().clone(),
    ));
    ctx_a.emit_ibc_event(IbcEvent::Message(MessageEvent::Connection))?;
    ctx_a.emit_ibc_event(event)?;

    ctx_a.log_message("success: conn_open_ack verification passed".to_string())?;

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
