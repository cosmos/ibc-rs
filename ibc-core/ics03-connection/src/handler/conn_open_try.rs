//! Protocol logic specific to processing ICS3 messages of type `MsgConnectionOpenTry`.;
use ibc_core_client::context::prelude::*;
use ibc_core_client::types::error::ClientError;
use ibc_core_connection_types::error::ConnectionError;
use ibc_core_connection_types::events::OpenTry;
use ibc_core_connection_types::msgs::MsgConnectionOpenTry;
use ibc_core_connection_types::{ConnectionEnd, Counterparty, State};
use ibc_core_handler_types::error::ContextError;
use ibc_core_handler_types::events::{IbcEvent, MessageEvent};
use ibc_core_host::types::identifiers::{ClientId, ConnectionId};
use ibc_core_host::types::path::{
    ClientConnectionPath, ClientConsensusStatePath, ClientStatePath, ConnectionPath, Path,
};
use ibc_core_host::{ExecutionContext, ValidationContext};
use ibc_primitives::prelude::*;
use ibc_primitives::proto::{Any, Protobuf};
use ibc_primitives::ToVec;

pub fn validate<Ctx>(ctx_b: &Ctx, msg: MsgConnectionOpenTry) -> Result<(), ContextError>
where
    Ctx: ValidationContext,
    <Ctx::HostClientState as TryFrom<Any>>::Error: Into<ClientError>,
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
    <Ctx::HostClientState as TryFrom<Any>>::Error: Into<ClientError>,
{
    ctx_b.validate_message_signer(&msg.signer)?;

    let client_val_ctx_b = ctx_b.get_client_validation_context();

    let client_state_of_b_on_a =
        Ctx::HostClientState::try_from(msg.client_state_of_b_on_a.clone()).map_err(Into::into)?;

    ctx_b.validate_self_client(client_state_of_b_on_a)?;

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
            client_val_ctx_b.client_state(vars.conn_end_on_b.client_id())?;

        client_state_of_a_on_b
            .status(client_val_ctx_b, &msg.client_id_on_b)?
            .verify_is_active()?;
        client_state_of_a_on_b.validate_proof_height(msg.proofs_height_on_a)?;

        let client_cons_state_path_on_b = ClientConsensusStatePath::new(
            msg.client_id_on_b.clone(),
            msg.proofs_height_on_a.revision_number(),
            msg.proofs_height_on_a.revision_height(),
        );

        let consensus_state_of_a_on_b =
            client_val_ctx_b.consensus_state(&client_cons_state_path_on_b)?;

        let prefix_on_a = vars.conn_end_on_b.counterparty().prefix();
        let prefix_on_b = ctx_b.commitment_prefix();

        {
            let expected_conn_end_on_a = ConnectionEnd::new(
                State::Init,
                client_id_on_a.clone(),
                Counterparty::new(msg.client_id_on_b.clone(), None, prefix_on_b),
                msg.versions_on_a.clone(),
                msg.delay_period,
            )?;

            client_state_of_a_on_b
                .verify_membership(
                    prefix_on_a,
                    &msg.proof_conn_end_on_a,
                    consensus_state_of_a_on_b.root(),
                    Path::Connection(ConnectionPath::new(&vars.conn_id_on_a)),
                    expected_conn_end_on_a.encode_vec(),
                )
                .map_err(ConnectionError::VerifyConnectionState)?;
        }

        client_state_of_a_on_b
            .verify_membership(
                prefix_on_a,
                &msg.proof_client_state_of_b_on_a,
                consensus_state_of_a_on_b.root(),
                Path::ClientState(ClientStatePath::new(client_id_on_a.clone())),
                msg.client_state_of_b_on_a.to_vec(),
            )
            .map_err(|e| ConnectionError::ClientStateVerificationFailure {
                client_id: msg.client_id_on_b.clone(),
                client_error: e,
            })?;

        let expected_consensus_state_of_b_on_a =
            ctx_b.host_consensus_state(&msg.consensus_height_of_b_on_a)?;

        let client_cons_state_path_on_a = ClientConsensusStatePath::new(
            client_id_on_a.clone(),
            msg.consensus_height_of_b_on_a.revision_number(),
            msg.consensus_height_of_b_on_a.revision_height(),
        );

        client_state_of_a_on_b
            .verify_membership(
                prefix_on_a,
                &msg.proof_consensus_state_of_b_on_a,
                consensus_state_of_a_on_b.root(),
                Path::ClientConsensusState(client_cons_state_path_on_a),
                expected_consensus_state_of_b_on_a.into().to_vec(),
            )
            .map_err(|e| ConnectionError::ConsensusStateVerificationFailure {
                height: msg.proofs_height_on_a,
                client_error: e,
            })?;
    }

    Ok(())
}

pub fn execute<Ctx>(ctx_b: &mut Ctx, msg: MsgConnectionOpenTry) -> Result<(), ContextError>
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
    let event = IbcEvent::OpenTryConnection(OpenTry::new(
        vars.conn_id_on_b.clone(),
        msg.client_id_on_b.clone(),
        conn_id_on_a.clone(),
        vars.client_id_on_a.clone(),
    ));
    ctx_b.emit_ibc_event(IbcEvent::Message(MessageEvent::Connection))?;
    ctx_b.emit_ibc_event(event)?;
    ctx_b.log_message("success: conn_open_try verification passed".to_string())?;

    ctx_b.increase_connection_counter()?;
    ctx_b.store_connection_to_client(
        &ClientConnectionPath::new(msg.client_id_on_b),
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
        let version_on_b = ctx_b.pick_version(&msg.versions_on_a)?;

        Ok(Self {
            conn_id_on_b: ConnectionId::new(ctx_b.connection_counter()?),
            conn_end_on_b: ConnectionEnd::new(
                State::TryOpen,
                msg.client_id_on_b.clone(),
                msg.counterparty.clone(),
                vec![version_on_b],
                msg.delay_period,
            )?,
            client_id_on_a: msg.counterparty.client_id().clone(),
            conn_id_on_a: msg
                .counterparty
                .connection_id()
                .ok_or(ConnectionError::InvalidCounterparty)?
                .clone(),
        })
    }
}
